use embassy_nrf::gpio::Output;
use embassy_nrf::spim::{Error, Spim};

use crate::skju_spi::{read_register, write_register};

// CONFIG
pub const WHO_AM_I: u8 = 0x75;
pub const CONFIG: u8 = 0x1A;
pub const GYRO_CONFIG: u8 = 0x1B;
pub const ACCEL_CONFIG: u8 = 0x1C;
pub const ACCEL_CONFIG_2: u8 = 0x1D;

// READS
pub const ACCEL_XOUT_H: u8 = 0x3B; // [15:8]
pub const ACCEL_XOUT_L: u8 = 0x3C; // [7:0]
pub const ACCEL_YOUT_H: u8 = 0x3D; // [15:8]
pub const ACCEL_YOUT_L: u8 = 0x3E; // [7:0]
pub const ACCEL_ZOUT_H: u8 = 0x3F; // [15:8]
pub const ACCEL_ZOUT_L: u8 = 0x40; // [7:0]
pub const GYRO_XOUT_H: u8 = 0x43; // [15:8]
pub const GYRO_XOUT_L: u8 = 0x44; // [7:0]
pub const GYRO_YOUT_H: u8 = 0x45; // [15:8]
pub const GYRO_YOUT_L: u8 = 0x46; // [7:0]
pub const GYRO_ZOUT_H: u8 = 0x47; // [15:8]
pub const GYRO_ZOUT_L: u8 = 0x48; // [7:0]

pub async fn configure(spi: &mut Spim<'_>, cs: &mut Output<'_>) -> Result<(), Error> {
    // TODO: configure other registers?

    write(spi, cs, GYRO_CONFIG, 0x00).await?;
    write(spi, cs, ACCEL_CONFIG, 0x00).await?;
    write(spi, cs, ACCEL_CONFIG_2, 0x00).await?;

    Ok(())
}

pub async fn read(spi: &mut Spim<'_>, cs: &mut Output<'_>, register: u8) -> Result<u8, Error> {
    let adjusted_register = register | 0b1000_0000;
    read_register(spi, cs, adjusted_register).await
}

pub async fn write(spi: &mut Spim<'_>, cs: &mut Output<'_>, register: u8, value: u8) -> Result<(), Error> {
    let adjusted_register = register & 0b0111_1111;
    write_register(spi, cs, adjusted_register, value).await
}
