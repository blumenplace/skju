//! MPU6500 module builder.
//!
//! This module introduces a unified way of initializing an MPU6500 instance.
//!
//! Builder requires a [Bus] and [Timer] implementation to be provided to be able
//! to build MPU6500 instance.
//!
//! The rest of the builder fields are optional and can be omitted. Omitting the field means the related
//! registers won't be written during initialization unless other configuration values require the
//! register to be updated, e.g. [USER_CTRL] must be updated when FIFO is enabled.
//!
//! Before updating configured registers, the final build() method will perform a full device reset
//! unless configured otherwise.
use crate::accel::AccelConfig;
use crate::bus::Bus;
use crate::config::MPU6500Config;
use crate::fifo::{FIFOConfig, FIFOMode};
use crate::gyro::GyroConfig;
use crate::interrupts::INTConfig;
use crate::mpu6500::MPU6500;
use crate::power_management::PowerManagementConfig;
use crate::registers::*;
use crate::timer::Timer;
use crate::user_control::UserControlConfig;
use crate::utils::WRITE_MASK;

/// Indicates Bus has not been provided to the builder; thus instance cannot be built.
pub struct NoBus;

/// Indicates Bus has been provided to the builder.
pub struct WithBus<B: Bus>(B);

/// Indicates Timer has not been provided to the builder; thus instance cannot be built.
pub struct NoTimer;

/// Indicates Timer has been provided to the builder.
pub struct WithTimer<T: Timer>(T);

/// MPU6500 builder struct.
pub struct MPU6500Builder<B, T> {
    /// Provides a common interface to communicate with the MPU6500 bus.
    /// See [crate::bus].
    pub bus: B,

    /// Provides a common interface to wait for a specified amount of time.
    /// See [crate::mpu6500::timer].
    pub timer: T,

    /// [CONFIG] register configuration.
    /// See [super::config].
    pub config: Option<MPU6500Config>,

    /// [ACCEL_CONFIG] and [ACCEL_CONFIG_2] register configuration.
    /// See [super::accel].
    pub accel_config: Option<AccelConfig>,

    /// [GYRO_CONFIG] register configuration.
    /// See [super::gyro].
    pub gyro_config: Option<GyroConfig>,

    /// [FIFO_EN] register configuration.
    /// Partially affects [USER_CTRL] register.
    /// See [super::fifo].
    pub fifo_config: Option<FIFOConfig>,

    /// [USER_CTRL] register configuration.
    /// See [super::user_control].
    pub user_ctrl_config: Option<UserControlConfig>,

    /// [INT_ENABLE] register configuration.
    /// See [super::interrupts].
    pub int_config: Option<INTConfig>,

    /// [PWR_MGMT_1] and [PWR_MGMT_2] register configuration.
    /// See [super::power_management].
    pub power_management_config: Option<PowerManagementConfig>,

    /// Value to be used for [SMPLRT_DIV] register.
    /// Required DLPF configuration for [GYRO_CONFIG] and [CONFIG] registers.
    pub sample_rate_divider: u8,

    /// Indicates whether a full device reset should be performed before configuring MPU6500 registers.
    /// Defaults to `true`.
    pub with_full_reset: bool,
}

/// [NoBus] related methods.
impl<T> MPU6500Builder<NoBus, T> {
    /// Provides a way to specify [Bus] implementation for the MPU6500 instance.
    /// Cannot be reconfigured.
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
            sample_rate_divider: self.sample_rate_divider,
            with_full_reset: self.with_full_reset,
        }
    }
}

/// [NoTimer] related methods.
impl<B> MPU6500Builder<B, NoTimer> {
    /// Provides a way to specify [Timer] implementation for the MPU6500 instance.
    /// Cannot be reconfigured.
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
            with_full_reset: self.with_full_reset,
            sample_rate_divider: self.sample_rate_divider,
        }
    }
}

/// Optional configuration methods.
impl<B, T> MPU6500Builder<B, T> {
    /// Specify the gyro configuration.
    pub fn with_config(mut self, config: MPU6500Config) -> MPU6500Builder<B, T> {
        self.config = Some(config);
        self
    }

    /// Specify the gyro configuration.
    pub fn with_gyro_config(mut self, config: GyroConfig) -> MPU6500Builder<B, T> {
        self.gyro_config = Some(config);
        self
    }

    /// Specify the accel configuration.
    pub fn with_accel_config(mut self, config: AccelConfig) -> MPU6500Builder<B, T> {
        self.accel_config = Some(config);
        self
    }

    /// Specify the FIFO configuration.
    pub fn with_fifo_config(mut self, config: FIFOConfig) -> MPU6500Builder<B, T> {
        self.fifo_config = Some(config);
        self
    }

    /// Specify the user control configuration.
    pub fn with_user_ctrl_config(mut self, config: UserControlConfig) -> MPU6500Builder<B, T> {
        self.user_ctrl_config = Some(config);
        self
    }

    /// Specify the interrupts configuration.
    pub fn with_int_config(mut self, config: INTConfig) -> MPU6500Builder<B, T> {
        self.int_config = Some(config);
        self
    }

    /// Specify the power management configuration.
    pub fn with_power_management_config(mut self, config: PowerManagementConfig) -> MPU6500Builder<B, T> {
        self.power_management_config = Some(config);
        self
    }

    /// Specify the sample rate divider. Requires DLPF configuration for [GYRO_CONFIG] and [CONFIG] registers.
    pub fn with_sample_rate_divider(mut self, divider: u8) -> MPU6500Builder<B, T> {
        self.sample_rate_divider = divider;
        self
    }

    /// Specify whether a full device reset should be performed before configuring the MPU6500 instance.
    pub fn with_full_reset(mut self, full_reset: bool) -> MPU6500Builder<B, T> {
        self.with_full_reset = full_reset;
        self
    }
}

/// Methods that require Bus and Timer implementations to be provided.
impl<T: Bus, U: Timer> MPU6500Builder<WithBus<T>, WithTimer<U>> {
    /// Builds the MPU6500 instance.
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

        if self.with_full_reset {
            full_reset(&mut bus, &mut timer).await;
        }

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

        if self.sample_rate_divider != 0 {
            let bytes_to_send = [SMPLRT_DIV & WRITE_MASK, self.sample_rate_divider];
            bus.send(&bytes_to_send).await;
        }

        MPU6500 {
            bus,
            timer,
            latest_interrupts: 0,
        }
    }
}

/// Performs a full device reset.
async fn full_reset<T: Bus, U: Timer>(bus: &mut T, timer: &mut U) {
    bus.send(&[PWR_MGMT_1 & WRITE_MASK, 0b1000_0000]).await;
    timer.wait_ms(100).await;

    bus.send(&[PWR_MGMT_1 & WRITE_MASK, 0b0000_0001]).await;
    timer.wait_ms(10).await;

    bus.send(&[USER_CTRL & WRITE_MASK, 0b0000_1111]).await;
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
