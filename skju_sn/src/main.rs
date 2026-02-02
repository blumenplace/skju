//! Skju Sensor Firmware.
//! The node collects seismic data from an MPU6500 sensor and sends it to a Sensor Gateway.

#![no_std]
#![no_main]

mod spi;

#[cfg(feature = "rtt")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::spim::Spim;
use embassy_nrf::{bind_interrupts, peripherals, spim};
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use panic_probe as _;
use skju_peripherals::mpu6500::MPU6500;
use skju_peripherals::mpu6500::accel::AccelConfig;
use skju_peripherals::mpu6500::config::MPU6500Config;
use skju_peripherals::mpu6500::fifo::{FIFOConfig, FIFOMode, FIFOSensors, MAX_FIFO_BUFFER_SIZE};
use skju_peripherals::mpu6500::gyro::GyroConfig;
use skju_peripherals::mpu6500::interrupts::{INTConfig, INTEnableFlags, INTFlags, InterruptStatus};
use skju_peripherals::mpu6500::registers::WHO_AM_I;

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
    let mpu_int: Input = Input::new(p.P0_31, Pull::Down);
    let fifo_sensors = FIFOSensors::GYRO_X | FIFOSensors::GYRO_Y | FIFOSensors::GYRO_Z | FIFOSensors::ACCEL;
    let mpu6500 = MPU6500::<SpiDeviceBus>::builder()
        .with_bus(spi_bus)
        .with_config(MPU6500Config::default())
        .with_accel_config(AccelConfig::default())
        .with_gyro_config(GyroConfig::default())
        .with_fifo_config(FIFOConfig::default().mode(FIFOMode::Override).sensors(fifo_sensors))
        .with_int_config(
            INTConfig::default()
                .int_enable_flags(INTEnableFlags::FIFO_OVERFLOW_EN)
                .int_flags(INTFlags::ACTL),
        )
        .build()
        .await;

    defmt::info!("SKJU sgw starting");

    spawner
        .spawn(handle_mpu_interrupts(mpu6500, mpu_int))
        .expect("mpu interrupt task failed to spawn");
}

#[embassy_executor::task]
async fn handle_mpu_interrupts(mut mpu6500: MPU6500<SpiDeviceBus>, mut int_pin: Input<'static>) {
    loop {
        int_pin.wait_for_low().await;
        mpu6500.set_interrupt_status().await;

        if !mpu6500.test_interrupt_status(InterruptStatus::FIFO_OVERFLOW_INT) {
            continue;
        }

        let fifo_layout = mpu6500.fifo_layout().await;
        let sample_count = mpu6500.fifo_sample_count().await;
        let total_bytes = fifo_layout.size * sample_count as usize;
        let mut buffer: [u8; MAX_FIFO_BUFFER_SIZE] = [0x00; MAX_FIFO_BUFFER_SIZE];

        mpu6500.drain_fifo(&mut buffer[..total_bytes]).await;

        // TODO: process data inside of the buffer
    }
}

#[embassy_executor::task]
async fn test_mpu_connection(mut mpu6500: MPU6500<SpiDeviceBus>) {
    loop {
        let device_id = mpu6500.read_register(WHO_AM_I).await;

        defmt::info!("MPU6500 WHO_AM_I: {}", device_id);
        Timer::after_millis(1000).await;
    }
}
