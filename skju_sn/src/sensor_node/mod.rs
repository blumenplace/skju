pub mod ble_peripheral;
mod spi;
mod timer;

use core::cell::Cell;
use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use ble_peripheral::{ADV_DATA, ReadingsServer, ReadingsServerEvent, ReadingsServiceEvent, SCAN_DATA};
use embassy_nrf::gpio::{Input, Output};
use embassy_nrf::spim::Spim;
use embassy_sync::blocking_mutex::Mutex as BlockingMutex;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_time::{Instant, Timer};
use futures::future::{Either, select};
use futures::pin_mut;
use mpu6500::MPU6500;
use mpu6500::accel::AccelConfig;
use mpu6500::config::{ConfigDLPFOptions, MPU6500Config};
use mpu6500::fifo::{FIFOConfig, FIFOMode, FIFOSensors};
use mpu6500::gyro::GyroConfig;
use mpu6500::interrupts::{INTConfig, INTEnableFlags, INTFlags, InterruptStatus};
use mpu6500::registers::WHO_AM_I;
use mpu6500::user_control::UserControlConfig;
use nrf_softdevice::Softdevice;
use nrf_softdevice::ble::{Connection, gatt_server, peripheral};
use spi::SpiDeviceBus;
use timer::TimerHandler;

use crate::constants::{BLE_BATCH_SIZE, MAX_SAMPLE_COUNT, SAMPLE_RATE_HZ, SAMPLE_SIZE, TIMESTAMP_BYTES};

static CENTRAL_TIMESTAMP_OFFSET: BlockingMutex<CriticalSectionRawMutex, Cell<i64>> = BlockingMutex::new(Cell::new(0));
static NOTIFY_ENABLED: AtomicBool = AtomicBool::new(false);
static READINGS_CHANNEL: Channel<CriticalSectionRawMutex, Readings, 1> = Channel::new();

#[embassy_executor::task]
pub async fn handle_mpu_interrupts(spim: Spim<'static>, mpu_cs: Output<'static>, mut int_pin: Input<'static>) {
    let mut mpu6500 = init_mpu(spim, mpu_cs).await;

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
        print_readings(&readings);

        let batch_span_millis = (1000 / SAMPLE_RATE_HZ) * (current_sample_count - 1) as u64;
        let central_offset = CENTRAL_TIMESTAMP_OFFSET.lock(|v| v.get());
        let batch_start_timestamp = (current_timestamp - batch_span_millis) as i64 + central_offset;

        // not synced yet or something went wrong, drop the batch
        if batch_start_timestamp < 0 {
            continue;
        }

        let _ = READINGS_CHANNEL.sender().try_send(Readings {
            batch_timestamp: batch_start_timestamp as u64,
            readings,
        });
    }
}

#[embassy_executor::task]
pub async fn advertise_ble(sd: &'static Softdevice, server: ReadingsServer) {
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
                ReadingsServiceEvent::CentralTimestampWrite(curr_central_timestamp) => {
                    let curr_local_timestamp = Instant::now().as_millis();
                    let timestamp_diff = curr_central_timestamp as i64 - curr_local_timestamp as i64;

                    CENTRAL_TIMESTAMP_OFFSET.lock(|v| v.set(timestamp_diff));
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

async fn process_readings(connection: &Connection, server: &ReadingsServer) {
    loop {
        let batch = READINGS_CHANNEL.receiver().receive().await;

        if !NOTIFY_ENABLED.load(Ordering::Acquire) {
            Timer::after_millis(1000).await;
            continue;
        }

        let _res = server.readings.readings_notify(connection, &batch.bytes());
    }
}

struct Readings {
    batch_timestamp: u64,
    readings: [u8; MAX_SAMPLE_COUNT * SAMPLE_SIZE],
}

impl Readings {
    fn bytes(&self) -> [u8; BLE_BATCH_SIZE] {
        let timestamp_bytes: [u8; TIMESTAMP_BYTES] = self.batch_timestamp.to_be_bytes();
        let mut batch_bytes = [0x00; BLE_BATCH_SIZE];

        batch_bytes[..TIMESTAMP_BYTES].copy_from_slice(&timestamp_bytes);
        batch_bytes[TIMESTAMP_BYTES..].copy_from_slice(&self.readings[..BLE_BATCH_SIZE]);

        batch_bytes
    }
}

fn print_readings(readings: &[u8]) {
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

async fn init_mpu(spim: Spim<'static>, mpu_cs: Output<'static>) -> MPU6500<SpiDeviceBus, TimerHandler> {
    let spim: Mutex<NoopRawMutex, Spim<'static>> = Mutex::new(spim);
    let spi_bus = SpiDeviceBus::new(spim, mpu_cs);
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

    mpu6500
}
