pub mod ble_peripheral;

use core::cell::Cell;
use core::sync::atomic::{AtomicBool, Ordering};

use ble_peripheral::{ADV_DATA, ReadingsServer, ReadingsServerEvent, ReadingsServiceEvent, SCAN_DATA};
use embassy_sync::blocking_mutex::Mutex as BlockingMutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Instant, Timer};
use futures::future::{Either, select};
use futures::pin_mut;
use nrf_softdevice::Softdevice;
use nrf_softdevice::ble::{Connection, gatt_server, peripheral};

use crate::mpu_sensor::readings::ReadingsChannel;

static CENTRAL_TIMESTAMP_OFFSET: BlockingMutex<CriticalSectionRawMutex, Cell<i64>> = BlockingMutex::new(Cell::new(0));
static NOTIFY_ENABLED: AtomicBool = AtomicBool::new(false);

#[embassy_executor::task]
pub async fn advertise_ble(
    sd: &'static Softdevice,
    server: ReadingsServer,
    readings_channel: &'static ReadingsChannel,
) {
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

        let reading_process_future = process_sensor_data(&connection, &server, readings_channel);

        pin_mut!(gatt_future);
        pin_mut!(reading_process_future);

        let _ = match select(gatt_future, reading_process_future).await {
            Either::Left(_) => defmt::info!("Readings processing error"),
            Either::Right(_) => defmt::info!("Connection lost"),
        };
    }
}

async fn process_sensor_data(connection: &Connection, server: &ReadingsServer, channel: &'static ReadingsChannel) {
    loop {
        let mut batch = channel.receiver().receive().await;

        if !NOTIFY_ENABLED.load(Ordering::Acquire) {
            Timer::after_millis(1000).await;
            continue;
        }

        if let Err(err) = batch.adjust_timestamp(CENTRAL_TIMESTAMP_OFFSET.lock(|v| v.get())) {
            defmt::error!("Failed to adjust timestamp: {:?}", err);
            continue;
        }

        let _ = server.readings.readings_notify(connection, &batch.bytes());
    }
}
