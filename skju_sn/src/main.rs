//! Skju Sensor Firmware.
//! The node collects seismic data from an MPU6500 sensor and sends it to a Sensor Gateway.

#![no_std]
#![no_main]

mod skju_peripherals;
mod skju_spi;

#[cfg(feature = "rtt")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::spim::Spim;
use embassy_nrf::{bind_interrupts, peripherals, spim};
use embassy_time::Timer;
use panic_probe as _;

use crate::skju_peripherals::mpu6500::WHO_AM_I;

bind_interrupts!(struct Irqs {
    SPI2 => spim::InterruptHandler<peripherals::SPI2>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let config = spim::Config::default();
    let p = embassy_nrf::init(Default::default());
    let mut spim = Spim::new(p.SPI2, Irqs, p.P0_27, p.P0_29, p.P0_26, config);
    let mut mpu_cs: Output = Output::new(p.P0_30, Level::High, OutputDrive::Standard);

    skju_peripherals::mpu6500::configure(&mut spim, &mut mpu_cs)
        .await
        .unwrap();

    defmt::info!("SKJU sgw starting");

    spawner
        .spawn(test_mpu_connection(spim, mpu_cs))
        .expect("mpu test task failed to spawn");
}

#[embassy_executor::task]
async fn test_mpu_connection(mut spim: Spim<'static>, mut mpu_cs: Output<'static>) {
    loop {
        // TODO: properly handle error?
        let device_id = skju_peripherals::mpu6500::read(&mut spim, &mut mpu_cs, WHO_AM_I)
            .await
            .expect("MPU WHO_AM_I read failed");

        defmt::info!("MPU6500 WHO_AM_I: {}", device_id);
        Timer::after_millis(1000).await;
    }
}
