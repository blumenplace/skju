use crate::bus::Bus;
use crate::peripherals::mpu6500::accel::AccelConfig;
use crate::peripherals::mpu6500::fifo::{FIFOConfig, FIFOLayout, FIFOMode};
use crate::peripherals::mpu6500::gyro::GyroConfig;
use crate::peripherals::mpu6500::registers::*;
use crate::peripherals::mpu6500::user_control::UserControlConfig;
use core::option::Option;

pub struct MPU6500<T: Bus> {
    pub bus: T,
}

pub struct NoBus;
pub struct WithBus<T: Bus>(T);

pub struct MPU6500Builder<B> {
    pub bus: B,
    pub accel_config: Option<AccelConfig>,
    pub gyro_config: Option<GyroConfig>,
    pub fifo_config: Option<FIFOConfig>,
    pub user_ctrl_config: Option<UserControlConfig>,
}

impl MPU6500Builder<NoBus> {
    pub fn with_bus<B: Bus>(self, bus: B) -> MPU6500Builder<WithBus<B>> {
        MPU6500Builder {
            bus: WithBus(bus),
            gyro_config: self.gyro_config,
            accel_config: self.accel_config,
            fifo_config: self.fifo_config,
            user_ctrl_config: self.user_ctrl_config,
        }
    }
}

impl<B> MPU6500Builder<B> {
    pub fn with_gyro_config(mut self, config: GyroConfig) -> MPU6500Builder<B> {
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

    pub fn with_user_ctrl_config(mut self, config: UserControlConfig) -> MPU6500Builder<B> {
        self.user_ctrl_config = Some(config);
        self
    }
}

impl<T: Bus> MPU6500Builder<WithBus<T>> {
    pub async fn build(mut self) -> MPU6500<T> {
        let mut bus = self.bus.0;
        let fifo_enabled = self.fifo_config.is_some();
        let config_register_byte = encode_config_register(&self.fifo_config);
        let config_bytes_to_send = [for_write(CONFIG), config_register_byte];

        // TODO: Implement general configuration module
        bus.send(&config_bytes_to_send).await;

        if let Some(config) = self.fifo_config {
            let fifo_en_register_byte = encode_fifo_en_register(&config);
            let bytes_to_send = [for_write(FIFO_EN), fifo_en_register_byte];

            bus.send(&bytes_to_send).await;
        }

        if let Some(accel_config) = self.accel_config {
            let accel_bytes = encode_accel_register(&accel_config);
            let bytes_to_send = [for_write(ACCEL_CONFIG), accel_bytes[0], accel_bytes[1]];

            bus.send(&bytes_to_send).await;
        }

        if let Some(gyro_config) = self.gyro_config {
            let gyro_config_byte = encode_gyro_register(&gyro_config);
            let bytes_to_send = [for_write(GYRO_CONFIG), gyro_config_byte];

            bus.send(&bytes_to_send).await;
        }

        if let Some(user_ctrl_config) = self.user_ctrl_config {
            if fifo_enabled {
                user_ctrl_config.enable_fifo();
            }

            let user_ctrl_config_byte = encode_user_ctrl_register(&user_ctrl_config);
            let bytes_to_send = [for_write(USER_CTRL), user_ctrl_config_byte];

            bus.send(&bytes_to_send).await;
        }

        MPU6500 { bus }
    }
}

fn encode_config_register(fifo_config: &Option<FIFOConfig>) -> u8 {
    let mut register_byte = 0;

    if let Some(fifo) = fifo_config {
        if fifo.mode == FIFOMode::StopWhenFull {
            register_byte |= 1 << 6;
        }
    }

    register_byte
}

fn for_write(register: u8) -> u8 {
    register | 0x7F
}
fn for_read(register: u8) -> u8 {
    register & 0x80
}
fn encode_fifo_en_register(fifo_config: &FIFOConfig) -> u8 {
    fifo_config.sensors.bits()
}
fn encode_accel_register(accel_config: &AccelConfig) -> [u8; 2] {
    accel_config.bits()
}
fn encode_gyro_register(gyro_config: &GyroConfig) -> u8 {
    gyro_config.bits()
}
fn encode_user_ctrl_register(user_ctrl_config: &UserControlConfig) -> u8 {
    user_ctrl_config.bits()
}
