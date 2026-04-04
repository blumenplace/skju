use core::slice::from_raw_parts;
use core::sync::atomic::{AtomicU8, Ordering};

use embassy_executor::Spawner;
use embassy_nrf::uarte::Uarte;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Instant, Timer};
use futures::future::{Either, select};
use futures::pin_mut;
use heapless::Vec;
use nrf_softdevice::ble::advertisement_builder::AdvertisementDataType;
use nrf_softdevice::ble::central::{ConnectConfig, ScanConfig, scan};
use nrf_softdevice::ble::gatt_client::discover;
use nrf_softdevice::ble::{Address, central};
use nrf_softdevice::raw::{ble_gap_addr_t, ble_gap_evt_adv_report_t};
use nrf_softdevice::{Softdevice, ble};

use crate::ble_bridge::ble_central::{ReadingsServiceClient, ReadingsServiceClientEvent};
use crate::constants::{
    BLE_BATCH_SIZE, BLE_CENTRAL_SCAN_DURATION, BLE_CENTRAL_SCAN_INTERVAL, BLE_SENSOR_NAME, TIMESTAMP_BYTES,
    TOTAL_SENSORS,
};
use crate::mpu_sensor::readings::ReadingsChannel;

pub mod ble_central;

static CURR_CONNECTIONS: AtomicU8 = AtomicU8::new(0);

struct ConnectionGuard;

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        CURR_CONNECTIONS.fetch_sub(1, Ordering::Release);
    }
}

#[embassy_executor::task]
pub async fn process_sensor_readings(mut uart: Uarte<'static>, readings_channel: &'static ReadingsChannel) {
    let mut unix_timestamp_base = get_unix_timestamp_base(&mut uart).await;
    let mut last_timestamp_sync = Instant::now();

    loop {
        let mut readings = readings_channel.receiver().receive().await;
        let time_elapsed = Instant::elapsed(&last_timestamp_sync).as_secs() > 60;
        let _ = readings.adjust_timestamp(unix_timestamp_base as i64);
        let bytes: [u8; BLE_BATCH_SIZE] = readings.into();

        uart.write(&bytes).await.expect("Failed to write readings");

        if time_elapsed {
            last_timestamp_sync = Instant::now();
            unix_timestamp_base = get_unix_timestamp_base(&mut uart).await;
        }
    }
}

#[embassy_executor::task]
pub async fn scan_ble_devices(
    softdevice: &'static Softdevice,
    spawner: Spawner,
    readings_channel: &'static ReadingsChannel,
) {
    loop {
        while CURR_CONNECTIONS.load(Ordering::Acquire) != 0 {
            defmt::info!("Waiting for connections to finish...");
            Timer::after_millis(BLE_CENTRAL_SCAN_INTERVAL).await;
            continue;
        }

        defmt::info!("START SCANNING!");

        let timeout_future = Timer::after_millis(BLE_CENTRAL_SCAN_DURATION);
        let ble_scan_future = do_ble_scan(softdevice, spawner, readings_channel);

        pin_mut!(timeout_future);
        pin_mut!(ble_scan_future);

        let _ = select(timeout_future, ble_scan_future).await;

        Timer::after_millis(BLE_CENTRAL_SCAN_INTERVAL).await;
    }
}

async fn do_ble_scan(softdevice: &'static Softdevice, spawner: Spawner, readings_channel: &'static ReadingsChannel) {
    let mut connected_nodes = Vec::<ble_gap_addr_t, { TOTAL_SENSORS as usize }>::new();

    loop {
        let peer_addr = scan_available_nodes(softdevice).await;
        let is_connected = connected_nodes.iter().any(|addr| addr.addr == peer_addr.addr);

        if is_connected {
            defmt::info!("Already connected to a sensor node, skipping...");
            continue;
        }

        CURR_CONNECTIONS.fetch_add(1, Ordering::Release);

        connected_nodes.push(peer_addr).ok();
        spawner
            .spawn(process_ble_connection(softdevice, peer_addr, readings_channel))
            .expect("process_ble_connection task failed to spawn");

        if connected_nodes.is_full() {
            defmt::info!("All sensors connected, stopping BLE scan");
            break;
        }
    }
}

#[embassy_executor::task(pool_size = TOTAL_SENSORS as usize)]
async fn process_ble_connection(
    sd: &'static Softdevice,
    peer_addr: ble_gap_addr_t,
    readings_channel: &'static ReadingsChannel,
) {
    let _guard = ConnectionGuard;
    let addrs = &[&Address::from_raw(peer_addr)];
    let mut config = ConnectConfig::default();

    config.scan_config.whitelist = Some(addrs);
    config.scan_config.interval = 8;
    config.scan_config.window = 8;
    config.conn_params = nrf_softdevice::raw::ble_gap_conn_params_t {
        min_conn_interval: 16,
        max_conn_interval: 16,
        slave_latency: 0,
        conn_sup_timeout: 400,
    };

    let Ok(connection) = central::connect(sd, &config).await else {
        return;
    };

    let Ok(client): Result<ReadingsServiceClient, _> = discover(&connection).await else {
        return;
    };

    let conn_interval_millis = (connection.conn_params().min_conn_interval * 5 / 4) as u64;
    let shared_central_timestamp = Instant::now().as_millis() + conn_interval_millis;

    client
        .readings_cccd_write(true)
        .await
        .expect("Failed to enable notifications");

    // TODO: define a custom event to schedule the timestamp sync afterwards
    client
        .central_timestamp_write(&shared_central_timestamp)
        .await
        .expect("Failed to write central timestamp");

    // TODO: consider handling disconnects that are not of type BleDisconnectReason::ExchangeComplete
    ble::gatt_client::run(&connection, &client, |event| match event {
        ReadingsServiceClientEvent::ReadingsNotification(batch) => {
            readings_channel
                .sender()
                .try_send(batch.into())
                .expect("Failed to send readings");
        }
    })
    .await;

    defmt::info!("Disconnected from peer: {}", peer_addr.addr);
}

async fn scan_available_nodes(softdevice: &Softdevice) -> ble_gap_addr_t {
    let scan_config = ScanConfig::default();
    let peer_addr = scan(softdevice, &scan_config, |params| -> Option<ble_gap_addr_t> {
        if is_skju_sensor_ad(params) {
            return Some(params.peer_addr);
        }

        None
    })
    .await
    .expect("Failed to scan for BLE devices");

    peer_addr
}

fn is_skju_sensor_ad(params: &ble_gap_evt_adv_report_t) -> bool {
    unsafe {
        let mut buffer = from_raw_parts(params.data.p_data, params.data.len as usize);

        while buffer.len() != 0 {
            let data_len = buffer[0] as usize;

            if buffer.len() < data_len + 1 {
                defmt::warn!(
                    "Advertisement data truncated. Expected {} bytes, got {} bytes",
                    data_len + 1,
                    buffer.len()
                );
                return false;
            }

            if data_len < 1 {
                defmt::warn!("Advertisement data invalid. Expected at least 1 byte");
                return false;
            }

            let data_type = buffer[1];
            let data = &buffer[2..data_len + 1];
            let is_full_name = data_type == AdvertisementDataType::FULL_NAME.to_u8();

            if is_full_name {
                return BLE_SENSOR_NAME.as_bytes() == data;
            }

            buffer = &buffer[data_len + 1..];
        }

        false
    }
}

async fn get_unix_timestamp_base(uart: &mut Uarte<'static>) -> u64 {
    let mut timestamp_buffer = [0u8; TIMESTAMP_BYTES];

    uart.write(&[42u8]).await.expect("Failed to notify timestamp sync");
    uart.read(&mut timestamp_buffer)
        .await
        .expect("Failed to read timestamp");

    let current_unix_timestamp = u64::from_be_bytes(timestamp_buffer);
    let unix_timestamp_base = current_unix_timestamp - Instant::now().as_millis();

    unix_timestamp_base
}
