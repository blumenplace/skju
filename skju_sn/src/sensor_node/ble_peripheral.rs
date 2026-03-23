use nrf_softdevice::Softdevice;
use nrf_softdevice::ble::advertisement_builder::{AdvertisementBuilder, AdvertisementPayload, Flag};

use crate::constants::{BLE_SENSOR_NAME, MAX_SAMPLE_COUNT, SAMPLE_SIZE};

pub static SCAN_DATA: AdvertisementPayload<32> = AdvertisementBuilder::new().build();
pub static ADV_DATA: AdvertisementPayload<32> = AdvertisementBuilder::new()
    .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
    .full_name(BLE_SENSOR_NAME)
    .build();

#[nrf_softdevice::gatt_service(uuid = "8700bc0e-1510-4ece-851e-56940ff8757a")]
pub struct ReadingsService {
    #[characteristic(uuid = "937eb842-06ce-49b6-a840-3e6c09151ce4", notify)]
    pub readings: [u8; MAX_SAMPLE_COUNT * SAMPLE_SIZE],
    #[characteristic(uuid = "7312f746-8e77-4e05-8841-f81dfa95580b", read)]
    pub config: u8,
}

#[nrf_softdevice::gatt_server]
pub struct ReadingsServer {
    pub readings: ReadingsService,
}

pub fn get_softdevice_config() -> nrf_softdevice::Config {
    nrf_softdevice::Config {
        conn_gatt: Some(nrf_softdevice::raw::ble_gatt_conn_cfg_t { att_mtu: 247 }),
        ..Default::default()
    }
}
