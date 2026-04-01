pub mod readings;
mod spi;
mod timer;

use embassy_nrf::gpio::{Input, Output};
use embassy_nrf::spim::Spim;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Instant;
use mpu6500::MPU6500;
use mpu6500::accel::AccelConfig;
use mpu6500::config::{ConfigDLPFOptions, MPU6500Config};
use mpu6500::fifo::{FIFOConfig, FIFOMode, FIFOSensors};
use mpu6500::gyro::GyroConfig;
use mpu6500::interrupts::{INTConfig, INTEnableFlags, INTFlags, InterruptStatus};
use mpu6500::registers::WHO_AM_I;
use mpu6500::user_control::UserControlConfig;

use self::readings::{Readings, ReadingsChannel};
use self::spi::SpiDeviceBus;
use crate::constants::{MAX_SAMPLE_COUNT, SAMPLE_RATE_HZ, SAMPLE_SIZE};
use crate::mpu_sensor::timer::TimerHandler;

#[embassy_executor::task]
pub async fn handle_mpu_interrupts(
    mut mpu6500: MPU6500<SpiDeviceBus, TimerHandler>,
    mut int_pin: Input<'static>,
    channel: &'static ReadingsChannel,
) {
    let who = mpu6500.read_register(WHO_AM_I).await;
    let fifo_layout = mpu6500.fifo_layout().await;

    defmt::info!("WHOAMI  {:08b}", who);

    if fifo_layout.sample_size != SAMPLE_SIZE {
        panic!("Unexpected sample size: {}", fifo_layout.sample_size);
    }

    loop {
        int_pin.wait_for_falling_edge().await;
        mpu6500.set_interrupt_status().await;

        if mpu6500.test_interrupt_status(InterruptStatus::FIFO_OVERFLOW_INT) {
            mpu6500.reset_fifo().await;
            continue;
        }

        let current_sample_bytes = mpu6500.fifo_bytes_count().await;
        let current_sample_count = current_sample_bytes as usize / SAMPLE_SIZE;
        let mut readings = [0x00; MAX_SAMPLE_COUNT * SAMPLE_SIZE];
        let current_timestamp = Instant::now().as_millis();

        if (current_sample_bytes as usize) < MAX_SAMPLE_COUNT * SAMPLE_SIZE {
            continue;
        }

        mpu6500.drain_fifo(&mut readings).await;

        let batch_span_millis = (1000 / SAMPLE_RATE_HZ) * (current_sample_count - 1) as u64;
        let batch_start_timestamp = current_timestamp - batch_span_millis;

        let _ = channel.sender().try_send(Readings {
            batch_timestamp: batch_start_timestamp,
            readings,
        });
    }
}

pub async fn init_mpu(spim: Spim<'static>, mpu_cs: Output<'static>) -> MPU6500<SpiDeviceBus, timer::TimerHandler> {
    let spim: Mutex<NoopRawMutex, Spim<'static>> = Mutex::new(spim);
    let spi_bus = SpiDeviceBus::new(spim, mpu_cs);
    let fifo_sensors = FIFOSensors::GYRO_X | FIFOSensors::GYRO_Y | FIFOSensors::GYRO_Z | FIFOSensors::ACCEL;
    let sample_rate_divider = (1000 / SAMPLE_RATE_HZ) - 1;
    let sample_rate_divider = sample_rate_divider.clamp(0, 255) as u8;
    let mpu6500 = MPU6500::<SpiDeviceBus, timer::TimerHandler>::builder()
        .with_bus(spi_bus)
        .with_timer(timer::TimerHandler)
        .with_config(MPU6500Config::default().dlpf_cfg(ConfigDLPFOptions::CFG1))
        .with_accel_config(AccelConfig::default())
        .with_user_ctrl_config(UserControlConfig::default().enable_fifo())
        .with_gyro_config(GyroConfig::default().f_choice_b(0))
        .with_fifo_config(FIFOConfig::default().mode(FIFOMode::Override).sensors(fifo_sensors))
        .with_sample_rate_divider(sample_rate_divider)
        .with_int_config(
            INTConfig::default()
                .int_enable_flags(INTEnableFlags::FIFO_OVERFLOW_EN | INTEnableFlags::RAW_RDY_EN)
                .int_flags(INTFlags::ACTL),
        )
        .build()
        .await;

    mpu6500
}
