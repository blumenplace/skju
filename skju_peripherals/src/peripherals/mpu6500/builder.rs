use crate::bus::Bus;
use crate::peripherals::mpu6500::accel::AccelConfig;
use crate::peripherals::mpu6500::config::MPU6500Config;
use crate::peripherals::mpu6500::fifo::{FIFOConfig, FIFOLayout, FIFOMode};
use crate::peripherals::mpu6500::gyro::GyroConfig;
use crate::peripherals::mpu6500::interrupts::INTConfig;
use crate::peripherals::mpu6500::mpu6500::MPU6500;
use crate::peripherals::mpu6500::power_management::PowerManagementConfig;
use crate::peripherals::mpu6500::registers::*;
use crate::peripherals::mpu6500::user_control::UserControlConfig;
use crate::peripherals::mpu6500::utils::WRITE_MASK;
use crate::timer::Timer;

pub struct NoBus;
pub struct WithBus<B: Bus>(B);

pub struct NoTimer;
pub struct WithTimer<T: Timer>(T);

pub struct MPU6500Builder<B, T> {
    pub bus: B,
    pub timer: T,
    pub config: Option<MPU6500Config>,
    pub accel_config: Option<AccelConfig>,
    pub gyro_config: Option<GyroConfig>,
    pub fifo_config: Option<FIFOConfig>,
    pub user_ctrl_config: Option<UserControlConfig>,
    pub int_config: Option<INTConfig>,
    pub power_management_config: Option<PowerManagementConfig>,
}

impl<T> MPU6500Builder<NoBus, T> {
    pub fn with_bus<B: Bus>(self, bus: B) -> MPU6500Builder<WithBus<B>, T> {
        MPU6500Builder {
            bus: WithBus(bus),
            timer: self.timer,
            config: self.config,
            gyro_config: self.gyro_config,
            accel_config: self.accel_config,
            fifo_config: self.fifo_config,
            user_ctrl_config: self.user_ctrl_config,
            int_config: self.int_config,
            power_management_config: self.power_management_config,
        }
    }
}

impl<B> MPU6500Builder<B, NoTimer> {
    pub fn with_timer<T: Timer>(self, timer: T) -> MPU6500Builder<B, WithTimer<T>> {
        MPU6500Builder {
            bus: self.bus,
            timer: WithTimer(timer),
            config: self.config,
            gyro_config: self.gyro_config,
            accel_config: self.accel_config,
            fifo_config: self.fifo_config,
            user_ctrl_config: self.user_ctrl_config,
            int_config: self.int_config,
            power_management_config: self.power_management_config,
        }
    }
}

impl<B, T> MPU6500Builder<B, T> {
    pub fn with_config(mut self, config: MPU6500Config) -> MPU6500Builder<B, T> {
        self.config = Some(config);
        self
    }

    pub fn with_gyro_config(mut self, config: GyroConfig) -> MPU6500Builder<B, T> {
        self.gyro_config = Some(config);
        self
    }

    pub fn with_accel_config(mut self, config: AccelConfig) -> MPU6500Builder<B, T> {
        self.accel_config = Some(config);
        self
    }

    pub fn with_fifo_config(mut self, config: FIFOConfig) -> MPU6500Builder<B, T> {
        self.fifo_config = Some(config);
        self
    }

    pub fn with_user_ctrl_config(mut self, config: UserControlConfig) -> MPU6500Builder<B, T> {
        self.user_ctrl_config = Some(config);
        self
    }

    pub fn with_int_config(mut self, config: INTConfig) -> MPU6500Builder<B, T> {
        self.int_config = Some(config);
        self
    }

    pub fn with_power_management_config(mut self, config: PowerManagementConfig) -> MPU6500Builder<B, T> {
        self.power_management_config = Some(config);
        self
    }
}

impl<T: Bus, U: Timer> MPU6500Builder<WithBus<T>, WithTimer<U>> {
    pub async fn build(self) -> MPU6500<T, U> {
        let mut bus = self.bus.0;
        let mut timer = self.timer.0;
        let fifo_enabled = self.fifo_config.is_some();
        let config_register_byte = encode_config_register(&self.config, &self.fifo_config);
        let config_bytes_to_send = [CONFIG & WRITE_MASK, config_register_byte];
        let user_ctrl_config = self.user_ctrl_config.unwrap_or_default();

        if fifo_enabled {
            user_ctrl_config.enable_fifo();
        }

        let user_ctrl_config_byte = encode_user_ctrl_register(&user_ctrl_config);
        let ctrl_config_bytes_to_send = [USER_CTRL & WRITE_MASK, user_ctrl_config_byte];

        full_reset(&mut bus, &mut timer).await;

        bus.send(&config_bytes_to_send).await;
        bus.send(&ctrl_config_bytes_to_send).await;

        if let Some(config) = self.fifo_config {
            let fifo_en_register_byte = encode_fifo_en_register(&config);
            let bytes_to_send = [FIFO_EN & WRITE_MASK, fifo_en_register_byte];

            bus.send(&bytes_to_send).await;
        }

        if let Some(accel_config) = self.accel_config {
            let accel_bytes = encode_accel_registers(&accel_config);
            let bytes_to_send = [ACCEL_CONFIG & WRITE_MASK, accel_bytes[0], accel_bytes[1]];

            bus.send(&bytes_to_send).await;
        }

        if let Some(gyro_config) = self.gyro_config {
            let gyro_config_byte = encode_gyro_register(&gyro_config);
            let bytes_to_send = [GYRO_CONFIG & WRITE_MASK, gyro_config_byte];
            bus.send(&bytes_to_send).await;
        }

        if let Some(int_config) = self.int_config {
            let int_cfg_bytes = encode_int_cfg_registers(&int_config);
            let bytes_to_send = [INT_PIN_CFG & WRITE_MASK, int_cfg_bytes[0], int_cfg_bytes[1]];
            bus.send(&bytes_to_send).await;
        }

        if let Some(power_management_config) = self.power_management_config {
            let power_management_bytes = encode_power_management_registers(&power_management_config);
            let bytes_to_send = [
                PWR_MGMT_1 & WRITE_MASK,
                power_management_bytes[0],
                power_management_bytes[1],
            ];

            bus.send(&bytes_to_send).await;
        }

        MPU6500 { bus, timer, latest_interrupts: 0 }
    }
}

async fn full_reset<T: Bus, U: Timer>(bus: &mut T, timer: &mut U) {
    bus.send(&[PWR_MGMT_1 & WRITE_MASK, 0b1000_0000]).await;
    timer.wait_ms(100).await;

    bus.send(&[PWR_MGMT_1 & WRITE_MASK, 0b0000_0001]).await;
    timer.wait_ms(10).await;

    bus.send(&[USER_CTRL & WRITE_MASK, 0b0000_1100]).await;
    timer.wait_ms(10).await;

    bus.send(&[USER_CTRL & WRITE_MASK, 0b0000_0000]).await;
    bus.send(&[FIFO_EN & WRITE_MASK, 0b0000_0000]).await;
    bus.send(&[INT_ENABLE & WRITE_MASK, 0b0000_0000]).await;
    bus.send(&[CONFIG & WRITE_MASK, 0b0000_0000]).await;
    bus.send(&[GYRO_CONFIG & WRITE_MASK, 0b0000_0000]).await;
    bus.send(&[ACCEL_CONFIG & WRITE_MASK, 0b0000_0000]).await;
    bus.send(&[ACCEL_CONFIG_2 & WRITE_MASK, 0b0000_0000]).await;
    bus.send(&[INT_PIN_CFG & WRITE_MASK, 0b0000_0000]).await;

    timer.wait_ms(5).await;
}

fn encode_config_register(config: &Option<MPU6500Config>, fifo_config: &Option<FIFOConfig>) -> u8 {
    let mut register_byte = 0;

    if let Some(config) = config {
        register_byte |= config.bits();
    }

    if let Some(fifo) = fifo_config {
        if fifo.mode == FIFOMode::StopWhenFull {
            register_byte |= 1 << 6;
        }
    }

    register_byte
}

fn encode_fifo_en_register(fifo_config: &FIFOConfig) -> u8 {
    fifo_config.sensors.bits()
}

fn encode_accel_registers(accel_config: &AccelConfig) -> [u8; 2] {
    accel_config.bits()
}

fn encode_gyro_register(gyro_config: &GyroConfig) -> u8 {
    gyro_config.bits()
}

fn encode_user_ctrl_register(user_ctrl_config: &UserControlConfig) -> u8 {
    user_ctrl_config.bits()
}

fn encode_int_cfg_registers(int_config: &INTConfig) -> [u8; 2] {
    int_config.bits()
}

fn encode_power_management_registers(power_management_config: &PowerManagementConfig) -> [u8; 2] {
    power_management_config.bits()
}
