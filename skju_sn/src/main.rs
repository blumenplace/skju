//! Skju Sensor Firmware.
//! The node collects seismic data from an MPU6500 sensor and sends it to a Sensor Gateway.
#![no_std]
#![no_main]

#[cfg(all(feature = "ble-bridge", feature = "ble-node"))]
compile_error!("ble-bridge and sensor-node features cannot be enabled at the same time");

#[cfg(not(any(feature = "ble-bridge", feature = "ble-node")))]
compile_error!("sensor-node or ble-bridge features should be enabled");

#[cfg(feature = "ble-bridge")]
mod ble_bridge;
#[cfg(feature = "ble-node")]
mod ble_node;
mod constants;
mod mpu_sensor;

use embassy_executor::Spawner;
use embassy_nrf::config::Config;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::interrupt::{InterruptExt, Priority};
use embassy_nrf::spim::Spim;
use embassy_nrf::{bind_interrupts, peripherals, spim};
use embassy_sync::channel::Channel;
use heapless::Vec;
use nrf_softdevice::Softdevice;
#[cfg(feature = "ble-bridge")]
use nrf_softdevice::raw::ble_gap_addr_t;
use {defmt_rtt as _, panic_probe as _};

#[cfg(feature = "ble-bridge")]
use crate::ble_bridge::{process_sensor_readings, scan_ble_devices};
#[cfg(feature = "ble-node")]
use crate::ble_node::advertise_ble;
#[cfg(feature = "ble-node")]
use crate::ble_node::ble_peripheral::{ReadingsServer, get_softdevice_config};
use crate::mpu_sensor::handle_mpu_interrupts;
use crate::mpu_sensor::readings::ReadingsChannel;

#[cfg(feature = "mpu-sensor")]
bind_interrupts!(struct Irqs {
    SPI2 => spim::InterruptHandler<peripherals::SPI2>;
});

static READINGS_CHANNEL: ReadingsChannel = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();

    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;

    let p = embassy_nrf::init(config);

    embassy_nrf::interrupt::SPI2.set_priority(Priority::P3);

    #[cfg(feature = "mpu-sensor")]
    {
        let spim_config = spim::Config::default();
        let spim = Spim::new(p.SPI2, Irqs, p.P0_27, p.P0_26, p.P0_29, spim_config);
        let mpu_cs: Output = Output::new(p.P0_30, Level::High, OutputDrive::Standard);
        let mpu_int: Input = Input::new(p.P0_31, Pull::Up);

        spawner
            .spawn(handle_mpu_interrupts(spim, mpu_cs, mpu_int, &READINGS_CHANNEL))
            .expect("mpu interrupt task failed to spawn");
    }

    #[cfg(feature = "ble-node")]
    {
        let softdevice_config = get_softdevice_config();
        let softdevice = Softdevice::enable(&softdevice_config);
        let server = ReadingsServer::new(softdevice).unwrap();

        spawner
            .spawn(softdevice_task(softdevice))
            .expect("softdevice task failed to spawn");

        spawner
            .spawn(advertise_ble(softdevice, server, &READINGS_CHANNEL))
            .expect("advertising task failed to spawn");
    }

    #[cfg(feature = "ble-bridge")]
    {
        let softdevice_config = ble_bridge::ble_central::get_softdevice_config();
        let softdevice = Softdevice::enable(&softdevice_config);

        spawner
            .spawn(softdevice_task(softdevice))
            .expect("softdevice task failed to spawn");

        spawner
            .spawn(scan_ble_devices(softdevice, spawner.clone(), &READINGS_CHANNEL))
            .expect("scan_ble_devices task failed to spawn");

        spawner
            .spawn(process_sensor_readings(&READINGS_CHANNEL))
            .expect("process_sensor_readings task failed to spawn");
    }
}

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await
}
