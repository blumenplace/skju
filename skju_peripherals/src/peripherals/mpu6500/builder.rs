use crate::bus::Bus;
use crate::peripherals::mpu6500::accel::AccelConfig;
use crate::peripherals::mpu6500::fifo::{FIFOConfig, FIFOMode};
use crate::peripherals::mpu6500::registers::*;
use core::option::Option;

pub struct MPU6500<T: Bus> {
    pub bus: T,
}

pub struct NoBus;
pub struct WithBus<T: Bus>(T);

pub struct MPU6500Builder<B> {
    pub bus: B,
    pub gyro_config: Option<u8>,
    pub accel_config: Option<AccelConfig>,
    pub fifo_config: Option<FIFOConfig>,
}

impl MPU6500Builder<NoBus> {
    pub fn with_bus<B: Bus>(self, bus: B) -> MPU6500Builder<WithBus<B>> {
        MPU6500Builder {
            bus: WithBus(bus),
            gyro_config: self.gyro_config,
            accel_config: self.accel_config,
            fifo_config: self.fifo_config,
        }
    }
}

impl<B> MPU6500Builder<B> {
    pub fn with_gyro_config(mut self, config: u8) -> MPU6500Builder<B> {
        self.gyro_config = Some(config);
        self
    }

    pub fn with_accel_config(mut self, config: AccelConfig) -> MPU6500Builder<B> {
        self.accel_config = Some(config);
        self
    }

    pub fn with_fifo_config(mut self, config: FIFOConfig) -> MPU6500Builder<B> {
        self.fifo_config = Some(config);
        self
    }
}

impl<T: Bus> MPU6500Builder<WithBus<T>> {
    pub async fn build(self) -> MPU6500<T> {
        let mut result = MPU6500 { bus: self.bus.0 };
        let fifo_enabled = &self.fifo_config.is_some();

        if let Some(config) = &self.fifo_config {
            let config_register_byte = encode_config_register(&config);

            result
                .bus
                .write(for_write(CONFIG), config_register_byte)
                .await;
        }

        if let Some(config) = &self.fifo_config {
            let fifo_en_register_byte = encode_fifo_en_register(&config);

            result
                .bus
                .write(for_write(FIFO_EN), fifo_en_register_byte)
                .await;
        }

        result
    }
}

fn encode_config_register(fifo: &FIFOConfig) -> u8 {
    let mut register_byte = 0;

    if fifo.mode == FIFOMode::StopWhenFull {
        register_byte |= 1 << 6;
    }

    register_byte
}

fn encode_fifo_en_register(fifo: &FIFOConfig) -> u8 {
    fifo.sensors.bits()
}

fn for_write(register: u8) -> u8 {
    register | 0x7F
}

fn for_read(register: u8) -> u8 {
    register & 0x80
}
