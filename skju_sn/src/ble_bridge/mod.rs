use core::slice::from_raw_parts;

use defmt::warn;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Instant;
use futures::future::{Either, select};
use futures::pin_mut;
use heapless::Vec;
use nrf_softdevice::ble::advertisement_builder::AdvertisementDataType;
use nrf_softdevice::ble::central::{ConnectConfig, ScanConfig, scan};
use nrf_softdevice::ble::gatt_client::discover;
use nrf_softdevice::ble::{Address, Connection, central};
use nrf_softdevice::raw::{ble_gap_addr_t, ble_gap_evt_adv_report_t};
use nrf_softdevice::{Softdevice, ble};

use crate::ble_bridge::ble_central::{ReadingsServiceClient, ReadingsServiceClientEvent};
use crate::constants::{BLE_BATCH_SIZE, BLE_SENSOR_NAME, TIMESTAMP_BYTES, TOTAL_SENSORS};

pub mod ble_central;

static READINGS_CHANNEL: Channel<CriticalSectionRawMutex, [u8; BLE_BATCH_SIZE], 1> = Channel::new();

pub async fn scan_available_nodes(softdevice: &Softdevice) -> ble_gap_addr_t {
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

#[embassy_executor::task(pool_size = TOTAL_SENSORS as usize)]
pub async fn process_ble_connection(sd: &'static Softdevice, peer_addr: ble_gap_addr_t) {
    let addrs = &[&Address::from_raw(peer_addr)];
    let mut config = ConnectConfig::default();

    config.scan_config.whitelist = Some(addrs);

    let connection = central::connect(sd, &config).await.expect("Failed to connect");
    let client: ReadingsServiceClient = discover(&connection).await.expect("Failed to discover ReadingsService");

    client
        .readings_cccd_write(true)
        .await
        .expect("Failed to enable notifications");

    let gatt_client_future = ble::gatt_client::run(&connection, &client, |event| match event {
        ReadingsServiceClientEvent::ReadingsNotification(batch) => {
            READINGS_CHANNEL
                .sender()
                .try_send(batch)
                .expect("Unable to send readings batch to channel");
        }
    });

    let readings_process_future = async {
        let mut last_sync = Instant::now();

        loop {
            let batch = READINGS_CHANNEL.receiver().receive().await;
            let timestamp_bytes: [u8; TIMESTAMP_BYTES] =
                batch[..TIMESTAMP_BYTES].try_into().expect("Not enough bytes received");

            let timestamp = u64::from_be_bytes(timestamp_bytes);
            let readings = &batch[TIMESTAMP_BYTES..];
            let conn_interval_millis = (connection.conn_params().min_conn_interval * 5 / 4) as u64;
            let shared_central_timestamp = Instant::now().as_millis() + conn_interval_millis;

            defmt::info!("Timestamp: {}.\nReadings: {}", timestamp / 1000, readings);

            // Synchronize central timestamp every minute
            if (Instant::elapsed(&last_sync).as_secs() > 60) {
                last_sync = Instant::now();
                client
                    .central_timestamp_write(&shared_central_timestamp)
                    .await
                    .expect("Failed to write central timestamp");
            }
        }
    };

    pin_mut!(gatt_client_future);
    pin_mut!(readings_process_future);

    let _ = match select(gatt_client_future, readings_process_future).await {
        Either::Left(_) => defmt::info!("GATT Client error"),
        Either::Right(_) => defmt::info!("Unable to process readings"),
    };

    defmt::info!("connection to {} lost", peer_addr.addr);
}

pub fn is_skju_sensor_ad(params: &ble_gap_evt_adv_report_t) -> bool {
    unsafe {
        let mut buffer = from_raw_parts(params.data.p_data, params.data.len as usize);

        while buffer.len() != 0 {
            let data_len = buffer[0] as usize;

            if buffer.len() < data_len + 1 {
                warn!(
                    "Advertisement data truncated. Expected {} bytes, got {} bytes",
                    data_len + 1,
                    buffer.len()
                );
                return false;
            }

            if data_len < 1 {
                warn!("Advertisement data invalid. Expected at least 1 byte");
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
