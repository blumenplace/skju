use embassy_nrf::gpio::Output;
use embassy_nrf::spim::Spim;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use skju_peripherals;
use skju_peripherals::bus::Bus;

pub struct SpiDeviceBus {
    spi: Mutex<NoopRawMutex, Spim<'static>>,
    cs: Output<'static>,
}

impl SpiDeviceBus {
    pub fn new(spim: Mutex<NoopRawMutex, Spim<'static>>, cs: Output<'static>) -> Self {
        Self { spi: spim, cs }
    }
}

impl Bus for SpiDeviceBus {
    async fn read(self: &mut Self, register: u8) -> u8 {
        let mut read = [register];
        let mut spi_guard = self.spi.lock().await;

        self.cs.set_low();
        spi_guard.write(&mut read).await.unwrap();
        spi_guard.read(&mut read).await.unwrap();
        self.cs.set_high();

        read[0]
    }

    async fn write(self: &mut Self, register: u8, value: u8) {
        let mut write = [register, value];
        let mut spi_guard = self.spi.lock().await;

        self.cs.set_low();
        spi_guard.write(&mut write).await.unwrap();
        self.cs.set_high();
    }
}
