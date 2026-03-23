use core::slice::from_raw_parts;

use defmt::warn;
use heapless::Vec;
use nrf_softdevice::ble::advertisement_builder::AdvertisementDataType;
use nrf_softdevice::ble::central::{ConnectConfig, ScanConfig, scan};
use nrf_softdevice::ble::gatt_client::discover;
use nrf_softdevice::ble::{Address, central};
use nrf_softdevice::raw::{ble_gap_addr_t, ble_gap_evt_adv_report_t};
use nrf_softdevice::{Softdevice, ble};

use crate::ble_bridge::ble_central::{ReadingsServiceClient, ReadingsServiceClientEvent};
use crate::constants::{BLE_SENSOR_NAME, TOTAL_SENSORS};

pub mod ble_central;

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

    let conn = central::connect(sd, &config).await.expect("Failed to connect");
    let client: ReadingsServiceClient = discover(&conn).await.expect("Failed to discover ReadingsService");

    defmt::info!("curr mtu {}", conn.att_mtu());

    client
        .readings_cccd_write(true)
        .await
        .expect("Failed to enable notifications");

    ble::gatt_client::run(&conn, &client, |event| match event {
        ReadingsServiceClientEvent::ReadingsNotification(readings) => {
            defmt::info!("last batch {}", readings)
        }
    })
    .await;

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
