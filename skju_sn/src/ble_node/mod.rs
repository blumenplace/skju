mod ble_peripheral;

pub use ble_peripheral::*;
use defmt::println;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::{Instant, Timer};
use futures::future::{Either, join, select};
use futures::pin_mut;
use heapless::Deque;
use nrf_softdevice::ble::gatt_server::NotifyValueError;
use nrf_softdevice::ble::peripheral::{ConnectableAdvertisement, advertise_connectable};
use nrf_softdevice::ble::{Connection, DisconnectedError, HciStatus, gatt_server};
use nrf_softdevice::{RawError, Softdevice};

use crate::ble::BleDisconnectReason;
use crate::constants::{
    BLE_COMPLETE_EXCHANGE_REASON, BLE_PERI_ADVERTISEMENT_DURATION, BLE_PERI_ADVERTISEMENT_INTERVAL,
    BLE_PERI_NOTIFICATION_WINDOW,
};
use crate::mpu_sensor::readings::{Readings, ReadingsChannel};

static NOTIFICATION_SIGNAL: Signal<CriticalSectionRawMutex, bool> = Signal::new();
static TIMESTAMP_SIGNAL: Signal<CriticalSectionRawMutex, i64> = Signal::new();
static READINGS_QUEUE: Mutex<CriticalSectionRawMutex, Deque<Readings, 100>> = Mutex::new(Deque::new());

#[embassy_executor::task]
pub async fn advertise_ble(sd: &'static Softdevice, server: ReadingsServer) {
    loop {
        if READINGS_QUEUE.lock().await.is_empty() {
            Timer::after_millis(BLE_PERI_ADVERTISEMENT_INTERVAL).await;
            continue;
        }

        let timeout_future = Timer::after_millis(BLE_PERI_ADVERTISEMENT_DURATION);
        let advertisement_future = async {
            let adv = ConnectableAdvertisement::ScannableUndirected {
                adv_data: &ADV_DATA,
                scan_data: &SCAN_DATA,
            };

            let result = advertise_connectable(&sd, adv, &Default::default())
                .await
                .expect("Adv connection failed");

            result
        };

        pin_mut!(advertisement_future);
        pin_mut!(timeout_future);

        let connection = match select(advertisement_future, timeout_future).await {
            Either::Left((connection, _)) => connection,
            Either::Right(_) => {
                Timer::after_millis(BLE_PERI_ADVERTISEMENT_INTERVAL).await;
                continue;
            }
        };

        NOTIFICATION_SIGNAL.reset();
        TIMESTAMP_SIGNAL.reset();

        let gatt_future = gatt_server::run(&connection, &server, |server_event| match server_event {
            ReadingsServerEvent::Readings(e) => match e {
                ReadingsServiceEvent::ReadingsCccdWrite { notifications } => {
                    NOTIFICATION_SIGNAL.signal(notifications);
                }
                ReadingsServiceEvent::CentralTimestampWrite(curr_central_timestamp) => {
                    let curr_local_timestamp = Instant::now().as_millis();
                    let timestamp_diff = curr_central_timestamp as i64 - curr_local_timestamp as i64;

                    TIMESTAMP_SIGNAL.signal(timestamp_diff);
                }
            },
        });

        let reading_process_future = process_sensor_data(&connection, &server);

        pin_mut!(gatt_future);
        pin_mut!(reading_process_future);

        let _ = match select(gatt_future, reading_process_future).await {
            Either::Left(_) => {
                defmt::info!("Connection lost...");
            }
            Either::Right((_, gatt_fut)) => {
                defmt::info!("Readings processing complete, disconnecting...");
                gatt_fut.await;
            }
        };

        Timer::after_millis(BLE_PERI_ADVERTISEMENT_INTERVAL).await;
    }
}

#[embassy_executor::task]
pub async fn collect_readings(readings_channel: &'static ReadingsChannel) {
    loop {
        let readings = readings_channel.receiver().receive().await;
        let mut queue = READINGS_QUEUE.lock().await;

        if queue.is_full() {
            queue.pop_front().expect("cannot pop despite queue being full");
        }

        queue
            .push_back(readings)
            .expect("cannot push despite queue not being full");
    }
}

async fn process_sensor_data(connection: &Connection, server: &ReadingsServer) {
    let len = READINGS_QUEUE.lock().await.len();
    let timeout_future = Timer::after_millis(BLE_PERI_NOTIFICATION_WINDOW);
    let notify_future = join(NOTIFICATION_SIGNAL.wait(), TIMESTAMP_SIGNAL.wait());

    let timestamp_offset = match select(timeout_future, notify_future).await {
        Either::Left(_) => return,
        Either::Right(((_, offset), _)) => offset,
    };

    defmt::info!("Sending {} readings to the central.", len);

    for i in 0..len {
        let mut batch = READINGS_QUEUE
            .lock()
            .await
            .pop_front()
            .expect("cannot pop despite queue not being empty");

        let _ = batch.adjust_timestamp(timestamp_offset);
        let bytes = batch.into();

        loop {
            match server.readings.readings_notify(connection, &bytes) {
                Ok(_) => break,
                Err(NotifyValueError::Disconnected) => return,
                Err(NotifyValueError::Raw(raw_error)) => match raw_error {
                    RawError::Resources => Timer::after_millis(10).await,
                    RawError::Busy => Timer::after_millis(1).await,
                    _ => return,
                },
            }
        }
    }

    defmt::info!("batches after drain remaining: {}", READINGS_QUEUE.lock().await.len());

    connection.disconnect().expect("Failed to disconnect");
}
