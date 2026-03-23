use crate::constants::{MAX_SAMPLE_COUNT, SAMPLE_SIZE, TOTAL_SENSORS};

#[nrf_softdevice::gatt_client(uuid = "8700bc0e-1510-4ece-851e-56940ff8757a")]
pub struct ReadingsServiceClient {
    #[characteristic(uuid = "937eb842-06ce-49b6-a840-3e6c09151ce4", notify)]
    pub readings: [u8; MAX_SAMPLE_COUNT * SAMPLE_SIZE],

    #[characteristic(uuid = "7312f746-8e77-4e05-8841-f81dfa95580b", read)]
    pub config: u8,
}

pub fn get_softdevice_config() -> nrf_softdevice::Config {
    nrf_softdevice::Config {
        gap_role_count: Some(nrf_softdevice::raw::ble_gap_cfg_role_count_t {
            adv_set_count: 0,
            periph_role_count: 0,
            central_role_count: TOTAL_SENSORS,
            central_sec_count: 0,
            _bitfield_1: Default::default(),
        }),
        conn_gatt: Some(nrf_softdevice::raw::ble_gatt_conn_cfg_t { att_mtu: 247 }),
        ..Default::default()
    }
}
