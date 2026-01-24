use crate::bus::Bus;
use core::future::Future;
use core::marker::Send;
use core::option::Option;
use core::option::Option::{None, Some};

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

pub struct MPU6500<T: Bus> {
    bus: T,
}

pub struct NoBus;
pub struct WithBus<T: Bus>(T);

pub struct MPU6500Builder<B> {
    bus: B,
    gyro_config: Option<u8>,
    accel_config: Option<[u8; 2]>,
}

impl<T: Bus> MPU6500<T> {
    pub fn builder() -> MPU6500Builder<NoBus> {
        MPU6500Builder {
            bus: NoBus,
            gyro_config: None,
            accel_config: None,
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

    fn read(&mut self, register: u8) -> impl Future<Output = u8> {
        let adjusted_register = register | 0x80;
        self.bus.read(adjusted_register)
    }

    fn write(&mut self, register: u8, value: u8) -> impl Future<Output = ()> {
        let adjusted_register = register | 0x7F;
        self.bus.write(adjusted_register, value)
    }
}

impl MPU6500Builder<NoBus> {
    pub fn with_bus<B: Bus>(self, bus: B) -> MPU6500Builder<WithBus<B>> {
        MPU6500Builder {
            bus: WithBus(bus),
            gyro_config: self.gyro_config,
            accel_config: self.accel_config,
        }
    }
}

impl<B> MPU6500Builder<B> {
    pub fn with_gyro_config(self, config: u8) -> MPU6500Builder<B> {
        MPU6500Builder {
            bus: self.bus,
            gyro_config: Some(config),
            accel_config: self.accel_config,
        }
    }

    pub fn with_accel_config(self, config: [u8; 2]) -> MPU6500Builder<B> {
        MPU6500Builder {
            bus: self.bus,
            gyro_config: self.gyro_config,
            accel_config: Some(config),
        }
    }
}

impl<T: Bus> MPU6500Builder<WithBus<T>> {
    pub async fn build(self) -> MPU6500<T> {
        let mut result = MPU6500 { bus: self.bus.0 };

        if let Some(config) = self.gyro_config {
            result.bus.write(GYRO_CONFIG | 0x7F, config).await;
        }

        if let Some(config) = self.accel_config {
            result.bus.write(ACCEL_CONFIG | 0x7F, config[0]).await;
            result.bus.write(ACCEL_CONFIG_2 | 0x7F, config[1]).await;
        }

        result
    }
}
