mod accel;
mod builder;
mod fifo;
mod gyro;
mod registers;

use crate::bus::Bus;
use core::future::Future;
use core::option::Option::{None, Some};

use builder::*;
use registers::*;

impl<T: Bus> MPU6500<T> {
    pub fn builder() -> MPU6500Builder<NoBus> {
        MPU6500Builder {
            bus: NoBus,
            gyro_config: None,
            accel_config: None,
            fifo_config: None,
        }
    }

    pub async fn read_accel(&mut self) -> (i16, i16, i16) {
        let x_high = self.read(ACCEL_XOUT_H).await;
        let x_low = self.read(ACCEL_XOUT_L).await;
        let y_high = self.read(ACCEL_YOUT_H).await;
        let y_low = self.read(ACCEL_YOUT_L).await;
        let z_high = self.read(ACCEL_ZOUT_H).await;
        let z_low = self.read(ACCEL_ZOUT_L).await;

        let x = i16::from_be_bytes([x_high, x_low]);
        let y = i16::from_be_bytes([y_high, y_low]);
        let z = i16::from_be_bytes([z_high, z_low]);

        (x, y, z)
    }

    pub async fn read_gyro(&mut self) -> (i16, i16, i16) {
        let x_high = self.read(GYRO_XOUT_H).await;
        let x_low = self.read(GYRO_XOUT_L).await;
        let y_high = self.read(GYRO_YOUT_H).await;
        let y_low = self.read(GYRO_YOUT_L).await;
        let z_high = self.read(GYRO_ZOUT_H).await;
        let z_low = self.read(GYRO_ZOUT_L).await;

        let x = i16::from_be_bytes([x_high, x_low]);
        let y = i16::from_be_bytes([y_high, y_low]);
        let z = i16::from_be_bytes([z_high, z_low]);

        (x, y, z)
    }

    pub async fn read_register(&mut self, register: u8) -> u8 {
        self.read(register).await
    }

    pub async fn write_register(&mut self, register: u8, value: u8) {
        self.write(register, value).await;
    }

    fn read(&mut self, register: u8) -> impl Future<Output = u8> {
        let adjusted_register = register | 0x80;
        self.bus.read(adjusted_register)
    }

    fn write(&mut self, register: u8, value: u8) -> impl Future<Output = ()> {
        let adjusted_register = register | 0x7F;
        self.bus.write(adjusted_register, value)
    }
}
