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
    async fn send(&mut self, bytes_to_send: &[u8]) {
        let mut spi_guard = self.spi.lock().await;

        self.cs.set_low();
        spi_guard.write(bytes_to_send).await.unwrap();
        self.cs.set_high();
    }

    async fn send_then_read(&mut self, bytes_to_send: &[u8], buffer: &mut [u8]) {
        let mut spi_guard = self.spi.lock().await;

        self.cs.set_low();
        spi_guard.transfer(buffer, bytes_to_send).await.unwrap();
        self.cs.set_high();
    }
}
