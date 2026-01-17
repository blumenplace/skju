#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::Peri;
use embassy_nrf::gpio::{AnyPin, Level, Output, OutputDrive};
use embassy_time::Timer;
use nrf_softdevice::ble::peripheral::ConnectableAdvertisement;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let config = nrf_softdevice::Config::default();
    let instance = nrf_softdevice::Softdevice::enable(&config);

    defmt::info!("SKJU sn starting");

    spawner
        .spawn(ble_softdevice_task(&instance))
        .expect("softdevice task failed to spawn");

    spawner
        .spawn(ble_advertisement_task(&instance))
        .expect("advertisement task failed");

    spawner
        .spawn(blink_task(p.P0_13.into()))
        .expect("blink task failed to spawn");
}

#[embassy_executor::task]
async fn blink_task(pin: Peri<'static, AnyPin>) {
    let mut led = Output::new(pin, Level::Low, OutputDrive::Standard);

    let mut counter = 0usize;
    loop {
        defmt::info!("LED tick");

        led.set_high();
        Timer::after_millis(500).await;

        led.set_low();
        Timer::after_millis(500).await;

        defmt::info!("SKJU loop iteration {}", counter);

        counter = counter.saturating_add(1);
    }
}

#[embassy_executor::task]
async fn ble_softdevice_task(instance: &'static nrf_softdevice::Softdevice) {
    instance.run().await;
}

#[embassy_executor::task]
async fn ble_advertisement_task(instance: &'static nrf_softdevice::Softdevice) {
    let peripheral_config = nrf_softdevice::ble::peripheral::Config::default();
    let advert = ConnectableAdvertisement::ScannableUndirected {
        adv_data: "SKJU sn".as_bytes(),
        scan_data: &[],
    };

    nrf_softdevice::ble::peripheral::advertise_connectable(&instance, advert, &peripheral_config)
        .await
        .unwrap();
}
