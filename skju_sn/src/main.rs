//! Skju Sensor Firmware.
//! The node collects seismic data from an MPU6500 sensor and sends it to a Sensor Gateway.
#![no_std]
#![no_main]
mod ble;
mod constants;
mod spi;
mod timer;

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_executor::Spawner;
use embassy_nrf::config::Config;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::interrupt::{InterruptExt, Priority};
use embassy_nrf::spim::Spim;
use embassy_nrf::{bind_interrupts, peripherals, spim};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use futures::future::{Either, select};
use futures::pin_mut;
use nrf_softdevice::Softdevice;
use nrf_softdevice::ble::{Connection, gatt_server, peripheral};
use skju_peripherals::mpu6500::MPU6500;
use skju_peripherals::mpu6500::accel::{AccelConfig, AccelDLPFOptions};
use skju_peripherals::mpu6500::config::{ConfigDLPFOptions, MPU6500Config};
use skju_peripherals::mpu6500::fifo::{FIFOConfig, FIFOMode, FIFOSensors, MAX_FIFO_BUFFER_SIZE};
use skju_peripherals::mpu6500::gyro::GyroConfig;
use skju_peripherals::mpu6500::interrupts::{INTConfig, INTEnableFlags, INTFlags, InterruptStatus};
use skju_peripherals::mpu6500::registers::{INT_ENABLE, USER_CTRL, WHO_AM_I};
use skju_peripherals::mpu6500::user_control::UserControlConfig;
use {defmt_rtt as _, panic_probe as _};

use crate::ble::{ADV_DATA, ReadingsServer, ReadingsServerEvent, ReadingsServiceEvent, SCAN_DATA, softdevice_task};
use crate::constants::{MAX_SAMPLE_COUNT, SAMPLE_RATE_HZ, SAMPLE_SIZE};
use crate::spi::SpiDeviceBus;
use crate::timer::TimerHandler;

bind_interrupts!(struct Irqs {
    SPI2 => spim::InterruptHandler<peripherals::SPI2>;
});

static NOTIFY_ENABLED: AtomicBool = AtomicBool::new(false);
static READINGS_CHANNEL: Channel<CriticalSectionRawMutex, Readings, 1> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();

    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;

    let p = embassy_nrf::init(config);
    let config = spim::Config::default();

    embassy_nrf::interrupt::SPI2.set_priority(Priority::P3);

    let sd_config = nrf_softdevice::Config::default();
    let soft_device = Softdevice::enable(&sd_config);
    let server = ReadingsServer::new(soft_device).unwrap();
    let spim = Spim::new(p.SPI2, Irqs, p.P0_27, p.P0_26, p.P0_29, config);
    let spim: Mutex<NoopRawMutex, Spim<'static>> = Mutex::new(spim);
    let mpu_cs: Output = Output::new(p.P0_30, Level::High, OutputDrive::Standard);
    let spi_bus = SpiDeviceBus::new(spim, mpu_cs);
    let mpu_int: Input = Input::new(p.P0_31, Pull::Up);
    let fifo_sensors = FIFOSensors::GYRO_X | FIFOSensors::GYRO_Y | FIFOSensors::GYRO_Z | FIFOSensors::ACCEL;
    let sample_rate_divider = (1000 / SAMPLE_RATE_HZ) - 1;
    let sample_rate_divider = sample_rate_divider.clamp(0, 255) as u8;

    let mpu6500 = MPU6500::<SpiDeviceBus, TimerHandler>::builder()
        .with_bus(spi_bus)
        .with_timer(TimerHandler)
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

    defmt::info!("SKJU sgw starting");

    spawner
        .spawn(softdevice_task(soft_device))
        .expect("softdevice task failed to spawn");

    spawner
        .spawn(handle_mpu_interrupts(mpu6500, mpu_int))
        .expect("mpu interrupt task failed to spawn");

    spawner
        .spawn(advertise_ble(soft_device, server))
        .expect("advertising task failed to spawn");
}

#[embassy_executor::task]
async fn handle_mpu_interrupts(mut mpu6500: MPU6500<SpiDeviceBus, TimerHandler>, mut int_pin: Input<'static>) {
    let who = mpu6500.read_register(WHO_AM_I).await;
    let fifo_layout = mpu6500.fifo_layout().await;
    let batch_size = MAX_SAMPLE_COUNT * fifo_layout.size;

    defmt::info!("WHOAMI  {:08b}", who);

    loop {
        int_pin.wait_for_falling_edge().await;
        mpu6500.set_interrupt_status().await;

        if mpu6500.test_interrupt_status(InterruptStatus::FIFO_OVERFLOW_INT) {
            defmt::info!("FIFO overflow occured");
            mpu6500.reset_fifo().await;
            continue;
        }

        let current_sample_count = mpu6500.fifo_bytes_count().await;

        if (current_sample_count as usize) < batch_size {
            continue;
        }

        let mut buffer: [u8; MAX_FIFO_BUFFER_SIZE] = [0x00; MAX_FIFO_BUFFER_SIZE];
        let mut readings = [0x00; MAX_SAMPLE_COUNT * SAMPLE_SIZE];

        mpu6500.drain_fifo(&mut buffer[..batch_size]).await;
        readings.copy_from_slice(&buffer[..MAX_SAMPLE_COUNT * SAMPLE_SIZE]);

        let readings = Readings {
            bytes_to_read: batch_size,
            readings,
        };

        print_readings(&readings.readings);

        let _ = READINGS_CHANNEL.sender().try_send(readings);
    }
}

#[embassy_executor::task]
async fn advertise_ble(sd: &'static Softdevice, server: ReadingsServer) {
    loop {
        let config = peripheral::Config::default();
        let adv = peripheral::ConnectableAdvertisement::ScannableUndirected {
            adv_data: &ADV_DATA,
            scan_data: &SCAN_DATA,
        };

        defmt::info!("Waiting for connection...");

        let connection: Connection = peripheral::advertise_connectable(&sd, adv, &config)
            .await
            .expect("Adv connection failed");

        let gatt_future = gatt_server::run(&connection, &server, |server_event| match server_event {
            ReadingsServerEvent::Readings(e) => match e {
                ReadingsServiceEvent::ReadingsCccdWrite { notifications } => {
                    NOTIFY_ENABLED.store(notifications, Ordering::Release);
                }
            },
        });

        let reading_process_future = process_readings(&connection, &server);

        pin_mut!(gatt_future);
        pin_mut!(reading_process_future);

        let _ = match select(gatt_future, reading_process_future).await {
            Either::Left(_) => defmt::info!("Readings processing error"),
            Either::Right(_) => defmt::info!("Connection lost"),
        };
    }
}

// TODO: rewrite to send values in chunks
async fn process_readings(connection: &Connection, server: &ReadingsServer) {
    loop {
        let batch = READINGS_CHANNEL.receiver().receive().await;

        defmt::info!("Readings: {=[u8]:x}", &batch.readings[..batch.bytes_to_read]);

        if !NOTIFY_ENABLED.load(Ordering::Acquire) {
            Timer::after_millis(1000).await;
            continue;
        }

        match server.readings.readings_notify(connection, &batch.readings) {
            Ok(_) => {}
            Err(_) => {
                let _ = server.readings.readings_set(&batch.readings);
            }
        }
    }
}

struct Readings {
    bytes_to_read: usize,
    readings: [u8; MAX_SAMPLE_COUNT * SAMPLE_SIZE],
}

pub fn print_readings(readings: &[u8]) {
    let samples = readings.len() / SAMPLE_SIZE;

    for i in 0..samples {
        let offset = i * SAMPLE_SIZE;

        let ax = i16::from_be_bytes([readings[offset + 0], readings[offset + 1]]);
        let ay = i16::from_be_bytes([readings[offset + 2], readings[offset + 3]]);
        let az = i16::from_be_bytes([readings[offset + 4], readings[offset + 5]]);

        let gx = i16::from_be_bytes([readings[offset + 6], readings[offset + 7]]);
        let gy = i16::from_be_bytes([readings[offset + 8], readings[offset + 9]]);
        let gz = i16::from_be_bytes([readings[offset + 10], readings[offset + 11]]);

        defmt::info!("S{} ACC[x:{} y:{} z:{}] GYR[x:{} y:{} z:{}]", i, ax, ay, az, gx, gy, gz);
    }
}
