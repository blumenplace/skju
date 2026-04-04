use nrf_softdevice::ble::advertisement_builder::{AdvertisementBuilder, AdvertisementPayload, Flag};

use crate::constants::{BLE_BATCH_SIZE, BLE_SENSOR_NAME};

pub static SCAN_DATA: AdvertisementPayload<32> = AdvertisementBuilder::new().build();
pub static ADV_DATA: AdvertisementPayload<32> = AdvertisementBuilder::new()
    .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
    .full_name(BLE_SENSOR_NAME)
    .build();

#[nrf_softdevice::gatt_service(uuid = "8700bc0e-1510-4ece-851e-56940ff8757a")]
pub struct ReadingsService {
    #[characteristic(uuid = "937eb842-06ce-49b6-a840-3e6c09151ce4", notify)]
    pub readings: [u8; BLE_BATCH_SIZE],
    #[characteristic(uuid = "7312f746-8e77-4e05-8841-f81dfa95580b", read)]
    pub config: u8,
    #[characteristic(uuid = "09e6884c-e11b-468a-8da6-68f6de6e4328", write)]
    pub central_timestamp: u64,
}

#[nrf_softdevice::gatt_server]
pub struct ReadingsServer {
    pub readings: ReadingsService,
}

pub fn get_softdevice_config() -> nrf_softdevice::Config {
    nrf_softdevice::Config {
        conn_gap: Some(nrf_softdevice::raw::ble_gap_conn_cfg_t {
            conn_count: 1,
            event_length: 24,
        }),
        conn_gatts: Some(nrf_softdevice::raw::ble_gatts_conn_cfg_t { hvn_tx_queue_size: 3 }),
        conn_gatt: Some(nrf_softdevice::raw::ble_gatt_conn_cfg_t { att_mtu: 247 }),
        ..Default::default()
    }
}
