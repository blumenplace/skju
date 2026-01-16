//! Skju Sensor Firmware.
//! The node collects seismic data from an MPU6500 sensor and sends it to a Sensor Gateway.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::{bind_interrupts, peripherals, spim};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SPI2 => spim::InterruptHandler<peripherals::SPI2>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    defmt::info!("SKJU sgw starting");

    let mut config = spim::Config::default();
    config.frequency = spim::Frequency::M1;

    let spi = spim::Spim::new(p.SPI2, Irqs, p.P0_27, p.P0_29, p.P0_26, config);
    let cs: Output = Output::new(p.P0_30, Level::High, OutputDrive::Standard);
    spawner
        .spawn(test_task(spi, cs))
        .expect("spim_test_task failed to spawn");
}

#[embassy_executor::task]
async fn test_task(mut spi: spim::Spim<'static>, mut cs: Output<'static>) {
    let mut buf = [0u8; 2];
    loop {
        defmt::info!("MPU6500 WHO_AM_I test");
        buf.copy_from_slice(&[0x75, 0x00]);

        cs.set_low();
        let _ = spi.transfer_in_place(&mut buf).await;
        cs.set_high();

        defmt::info!("MPU6500 WHO_AM_I: 0x{=u8:X}", buf[1]);
        Timer::after_millis(500).await;
    }
}
