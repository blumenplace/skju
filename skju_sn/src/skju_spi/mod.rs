use embassy_nrf::gpio::Output;
use embassy_nrf::spim::{Error, Spim};

pub async fn read_register(spi: &mut Spim<'_>, cs: &mut Output<'_>, register: u8) -> Result<u8, Error> {
    let mut read = [register];

    cs.set_low();
    spi.write(&mut read).await?;
    spi.read(&mut read).await?;
    cs.set_high();

    Ok(read[0])
}

pub async fn write_register(spi: &mut Spim<'_>, cs: &mut Output<'_>, register: u8, value: u8) -> Result<(), Error> {
    let mut write = [register, value];

    cs.set_low();
    spi.write(&mut write).await?;
    cs.set_high();

    Ok(())
}
