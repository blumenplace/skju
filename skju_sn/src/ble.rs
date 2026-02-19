use nrf_softdevice::ble::advertisement_builder::{AdvertisementBuilder, AdvertisementPayload, Flag, ServiceList};
use nrf_softdevice::{Softdevice, raw};

use crate::constants::{MAX_SAMPLE_COUNT, SAMPLE_SIZE};

pub static ADV_DATA: AdvertisementPayload<32> = AdvertisementBuilder::new()
    .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
    .full_name("SKJU_SN_BLE")
    .build();

pub static SCAN_DATA: AdvertisementPayload<32> = AdvertisementBuilder::new().build();

#[embassy_executor::task]
pub async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

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
