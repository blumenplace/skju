//! Implementation of MPU6500 peripheral instance.

use crate::bus::Bus;
use crate::mpu6500::builder::NoTimer;
use crate::mpu6500::fifo::MAX_FIFO_BUFFER_SIZE;
use crate::mpu6500::registers::{SIGNAL_PATH_RESET, USER_CTRL};
use crate::peripherals::mpu6500::builder::{MPU6500Builder, NoBus};
use crate::peripherals::mpu6500::fifo::FIFOLayout;
use crate::peripherals::mpu6500::interrupts::InterruptStatus;
use crate::peripherals::mpu6500::registers::{
    ACCEL_XOUT_H, FIFO_COUNT_H, FIFO_EN, FIFO_R_W, GYRO_XOUT_H, INT_STATUS, PWR_MGMT_1, PWR_MGMT_2,
};
use crate::peripherals::mpu6500::utils::{READ_MASK, WRITE_MASK};
use crate::timer::Timer;

pub struct MPU6500<T: Bus, U: Timer> {
    /// Provides a common interface to communicate with the MPU6500 bus.
    /// See [crate::bus].
    pub bus: T,

    /// Provides a common interface to wait for a specified amount of time.
    /// See [crate::timer].
    pub timer: U,

    /// Internal state of latest interrupts.
    /// When the interrupt register is read, all the interrupts are cleared.
    /// This field preserves the latest interrupts, allowing to process them later.
    pub(crate) latest_interrupts: u8,
}

impl<T: Bus, U: Timer> MPU6500<T, U> {
    /// Create a builder instance for MPU6500 peripheral.
    /// See [super::builder].
    pub fn builder() -> MPU6500Builder<NoBus, NoTimer> {
        MPU6500Builder {
            bus: NoBus,
            timer: NoTimer,
            config: None,
            int_config: None,
            fifo_config: None,
            gyro_config: None,
            accel_config: None,
            user_ctrl_config: None,
            power_management_config: None,
            sample_rate_divider: 0,
            with_full_reset: true,
        }
    }

    /// Read the specified register from the MPU6500.
    /// Does not support multibyte read.
    pub async fn read_register(&mut self, register: u8) -> u8 {
        let bytes_to_send = [register | READ_MASK, 0x00];
        let mut read_into = [0x00; 2];

        self.bus
            .send_then_read(&bytes_to_send, &mut read_into)
            .await;

        read_into[1]
    }

    /// Write the specified register to the MPU6500.
    /// Does not support multibyte write.
    pub async fn write_register(&mut self, register: u8, value: u8) {
        let address = register & WRITE_MASK;
        self.bus.send(&[address, value]).await;
    }

    /// Read the [INT_STATUS] register and updates the internal state of the latest interrupts.
    pub async fn set_interrupt_status(&mut self) {
        let mut read_into = [0x00; 2];
        let bytes_to_send = [INT_STATUS | READ_MASK, 0x00];

        self.bus
            .send_then_read(&bytes_to_send, &mut read_into)
            .await;

        self.latest_interrupts = self.latest_interrupts | read_into[1];
    }

    /// Check if the specified interrupt status is set and clears it from the internal state.
    /// If it is suspected that interrupt status changed, [MPU6500::set_interrupt_status] should be called first.
    pub fn test_interrupt_status(&mut self, status: InterruptStatus) -> bool {
        let mask = status.bits();
        let result = self.latest_interrupts & mask != 0;

        // Clear status bit to mark it as processed
        self.latest_interrupts &= !mask;

        result
    }

    /// Read the latest accel data from accel registers.
    pub async fn read_accel(&mut self) -> (i16, i16, i16) {
        let mut bytes_to_send = [0x00; 7];
        let mut read_into = [0u8; 7];

        bytes_to_send[0] = ACCEL_XOUT_H | READ_MASK;

        self.bus
            .send_then_read(&bytes_to_send, &mut read_into)
            .await;

        let [_, x_high, x_low, y_high, y_low, z_high, z_low] = read_into;
        let x = i16::from_be_bytes([x_high, x_low]);
        let y = i16::from_be_bytes([y_high, y_low]);
        let z = i16::from_be_bytes([z_high, z_low]);

        (x, y, z)
    }

    /// Read the latest gyro data from gyro registers.
    pub async fn read_gyro(&mut self) -> (i16, i16, i16) {
        let mut bytes_to_send = [0x00; 7];
        let mut read_into = [0u8; 7];

        bytes_to_send[0] = GYRO_XOUT_H | READ_MASK;

        self.bus
            .send_then_read(&bytes_to_send, &mut read_into)
            .await;

        let [_, x_high, x_low, y_high, y_low, z_high, z_low] = read_into;
        let x = i16::from_be_bytes([x_high, x_low]);
        let y = i16::from_be_bytes([y_high, y_low]);
        let z = i16::from_be_bytes([z_high, z_low]);

        (x, y, z)
    }

    /// Read the contents of the FIFO into the provided buffer.
    /// After the read, bytes are removed from the FIFO.
    ///
    /// This method does not validate buffer length.
    /// To have a valid set of readings, make sure fifo contains the required number of bytes set.
    pub async fn drain_fifo(&mut self, buffer: &mut [u8]) {
        let address = FIFO_R_W | READ_MASK;

        let mut bytes_to_send = [0u8; MAX_FIFO_BUFFER_SIZE + 1];
        let mut read_into = [0u8; MAX_FIFO_BUFFER_SIZE + 1];

        bytes_to_send[0] = address;

        let len = buffer.len() + 1;

        self.bus
            .send_then_read(&bytes_to_send[..len], &mut read_into[..len])
            .await;

        buffer.copy_from_slice(&read_into[1..len]);
    }

    /// Reset the FIFO buffer with the following steps:
    /// 1. Save USER_CTRL and FIFO_EN configuration to set it back again later.
    /// 2. Temporarily disable FIFO and mark it for reset via [USER_CTRL] register.
    /// 3. Reset gyro / accel / temp signal paths via [SIGNAL_PATH_RESET] register.
    /// 4. Reset INT statuses through reading the [INT_STATUS] register.
    /// 5. Disable FIFO devices (accel / gyro / temp) via [FIFO_EN] register.
    /// 6. Restore initial [USER_CTRL] and [FIFO_EN] register values.
    /// 7. Reset the internal interrupts state.
    pub async fn reset_fifo(&mut self) {
        let mut initial_user_ctrl = [0x00; 2];
        let mut initial_fifo_en = [0x00; 2];
        let partial_reset = 1 << 2 | 1 << 1 | 1 << 0;

        // Save current values of user_ctrn and enabled fifo flags
        self.bus
            .send_then_read(&[USER_CTRL | READ_MASK, 0x00], &mut initial_user_ctrl)
            .await;

        self.bus
            .send_then_read(&[FIFO_EN | READ_MASK, 0x00], &mut initial_fifo_en)
            .await;

        let updated_user_ctrl = initial_user_ctrl[1] | (1 << 2);

        // Temporary disable fifo and mark it for reset
        self.bus
            .send(&[USER_CTRL & WRITE_MASK, updated_user_ctrl])
            .await;

        // Reset gyro / accel / temp signal paths (same as for full device reset)
        self.bus
            .send(&[SIGNAL_PATH_RESET & WRITE_MASK, partial_reset])
            .await;

        // Read int status to fully reset it
        // TODO: consider preserving other flags when updating self.latest_interrupts
        self.bus
            .send_then_read(&[INT_STATUS | READ_MASK, 0x00], &mut [0x00; 2])
            .await;

        // Temporary disable FIFO to prevent further sampling
        self.bus.send(&[FIFO_EN & WRITE_MASK, 0x00]).await;

        // Restore initial user_ctrl state
        self.bus
            .send(&[USER_CTRL & WRITE_MASK, initial_user_ctrl[1]])
            .await;

        // Restore initial enabled fifo flags
        self.bus
            .send(&[FIFO_EN & WRITE_MASK, initial_fifo_en[1]])
            .await;

        // Reset internal interrupts state
        self.latest_interrupts = 0x00;
    }

    /// Read the current FIFO layout from [FIFO_EN] register.
    /// See [FIFOLayout].
    pub async fn fifo_layout(&mut self) -> FIFOLayout {
        let bytes_to_send = [FIFO_EN | READ_MASK, 0];
        let mut read_into = [0x00; 2];

        self.bus
            .send_then_read(&bytes_to_send, &mut read_into)
            .await;

        let fifo_layout = FIFOLayout::from_fifo_register(read_into[1]);

        fifo_layout
    }

    /// Read the number of bytes currently stored in the FIFO.
    pub async fn fifo_bytes_count(&mut self) -> u16 {
        let bytes_to_send = [FIFO_COUNT_H | READ_MASK, 0x00, 0x00];
        let mut read_into = [0u8; 3];

        self.bus
            .send_then_read(&bytes_to_send, &mut read_into)
            .await;

        let [_, count_high, count_low] = read_into;
        let count_high_mask = 0x1F;
        let total_count = u16::from_be_bytes([count_high & count_high_mask, count_low]);

        total_count
    }

    /// Full device reset with the following steps:
    /// 1. Mark the device for reset.
    /// 2. Wait for 100ms.
    /// 3. Reset gyro / accel / temp signal paths.
    /// 4. Wait for 100ms.
    ///
    /// Reset can be performed using only the first step.
    /// The current implementation covers the case where reset is performed via SPI.
    pub async fn reset_device(&mut self) {
        let reset_bit = 1 << 7;
        let partial_reset = 1 << 2 | 1 << 1 | 1 << 0;
        let partial_reset_address = SIGNAL_PATH_RESET | WRITE_MASK;

        self.set_power_mng_1_bit(reset_bit, false).await;
        self.timer.wait_ms(100).await;
        self.bus.send(&[partial_reset_address, partial_reset]).await;
        self.timer.wait_ms(100).await;
    }

    /// Set sleep mode via [PWR_MGMT_1] register.
    pub async fn set_sleep_mode(&mut self, sleep: bool) {
        let sleep_bit = 1 << 6;
        self.set_power_mng_1_bit(sleep_bit, sleep).await;
    }

    /// Set cycle mode via [PWR_MGMT_1] register.
    pub async fn set_cycle_mode(&mut self, cycle: bool) {
        let cycle_bit = 1 << 5;
        self.set_power_mng_1_bit(cycle_bit, cycle).await;
    }

    /// Set gyro standby via [PWR_MGMT_1] register.
    pub async fn set_gyro_standby(&mut self, standby: bool) {
        let gyro_standby_bit = 1 << 4;
        self.set_power_mng_1_bit(gyro_standby_bit, standby).await;
    }

    /// Set sleep temp disabled via [PWR_MGMT_1] register.
    pub async fn set_temp_disabled(&mut self, disabled: bool) {
        let temp_disabled_bit = 1 << 3;
        self.set_power_mng_1_bit(temp_disabled_bit, disabled).await;
    }

    /// Set disabled accel axes via [PWR_MGMT_2] register.
    pub async fn disable_accel_axes(&mut self, axes: [bool; 3]) {
        let mut curr = [0x00; 2];
        let bytes_to_send = [PWR_MGMT_2 | READ_MASK, 0x00];
        let x_mask = 1 << 5;
        let y_mask = 1 << 4;
        let z_mask = 1 << 3;

        self.bus.send_then_read(&bytes_to_send, &mut curr).await;

        let updated = if axes[0] { curr[1] | x_mask } else { curr[1] & !x_mask };
        let updated = if axes[1] { updated | y_mask } else { updated & !y_mask };
        let updated = if axes[2] { updated | z_mask } else { updated & !z_mask };

        self.bus.send(&[PWR_MGMT_2 | WRITE_MASK, updated]).await;
    }

    /// Set disabled gyro axes via [PWR_MGMT_2] register.
    pub async fn disable_gyro_axes(&mut self, axes: [bool; 3]) {
        let mut curr = [0x00; 2];
        let bytes_to_send = [PWR_MGMT_2 | READ_MASK, 0x00];
        let x_mask = 1 << 2;
        let y_mask = 1 << 1;
        let z_mask = 1 << 0;

        self.bus.send_then_read(&bytes_to_send, &mut curr).await;

        let updated = if axes[0] { curr[1] | x_mask } else { curr[1] & !x_mask };
        let updated = if axes[1] { updated | y_mask } else { updated & !y_mask };
        let updated = if axes[2] { updated | z_mask } else { updated & !z_mask };

        self.bus.send(&[PWR_MGMT_2 | WRITE_MASK, updated]).await;
    }

    /// Utility function to toggle a particular bit of [POWER_MNG_1] register.
    async fn set_power_mng_1_bit(&mut self, bit: u8, enabled: bool) {
        let mut current = [0x00; 2];
        let bytes_to_send = [PWR_MGMT_1 | READ_MASK, 0x00];

        self.bus.send_then_read(&bytes_to_send, &mut current).await;

        let updated = if enabled { current[1] | bit } else { current[1] & !bit };

        self.bus.send(&[PWR_MGMT_1 | WRITE_MASK, updated]).await;
    }
}
