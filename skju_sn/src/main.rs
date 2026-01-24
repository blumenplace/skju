//! Skju Sensor Firmware.
//! The node collects seismic data from an MPU6500 sensor and sends it to a Sensor Gateway.

#![no_std]
#![no_main]

mod spi;

#[cfg(feature = "rtt")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::spim::Spim;
use embassy_nrf::{bind_interrupts, peripherals, spim};
use embassy_time::Timer;
use panic_probe as _;
use skju_peripherals::peripherals::mpu6500::{MPU6500, WHO_AM_I};

use crate::spi::SpiDeviceBus;

bind_interrupts!(struct Irqs {
    SPI2 => spim::InterruptHandler<peripherals::SPI2>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let config = spim::Config::default();
    let p = embassy_nrf::init(Default::default());
    let spim = Spim::new(p.SPI2, Irqs, p.P0_27, p.P0_29, p.P0_26, config);
    let spim = Mutex::new(spim);
    let mpu_cs: Output = Output::new(p.P0_30, Level::High, OutputDrive::Standard);
    let spi_bus = SpiDeviceBus::new(spim, mpu_cs);
    let mpu6500 = MPU6500::<SpiDeviceBus>::builder()
        .with_bus(spi_bus)
        .with_accel_config([0x00, 0x00])
        .with_gyro_config(0x00)
        .build()
        .await;

    defmt::info!("SKJU sgw starting");

    spawner
        .spawn(test_mpu_connection(mpu6500))
        .expect("mpu test task failed to spawn");
}

#[embassy_executor::task]
async fn test_mpu_connection(mut mpu6500: MPU6500<SpiDeviceBus>) {
    loop {
        let device_id = mpu6500.read_register(WHO_AM_I).await;

        defmt::info!("MPU6500 WHO_AM_I: {}", device_id);
        Timer::after_millis(1000).await;
    }
}
