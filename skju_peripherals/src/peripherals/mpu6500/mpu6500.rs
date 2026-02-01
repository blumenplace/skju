use crate::bus::Bus;
use crate::peripherals::mpu6500::builder::{MPU6500Builder, NoBus};
use crate::peripherals::mpu6500::fifo::FIFOLayout;
use crate::peripherals::mpu6500::interrupts::InterruptStatus;
use crate::peripherals::mpu6500::registers::{
    ACCEL_XOUT_H, FIFO_COUNT_H, FIFO_EN, FIFO_R_W, GYRO_XOUT_H, INT_STATUS, PWR_MGMT_1, PWR_MGMT_2,
};
use crate::peripherals::mpu6500::utils::{READ_MASK, WRITE_MASK};

pub struct MPU6500<T: Bus> {
    pub bus: T,
    pub(crate) latest_interrupts: u8,
}

impl<T: Bus> MPU6500<T> {
    pub fn builder() -> MPU6500Builder<NoBus> {
        MPU6500Builder {
            bus: NoBus,
            config: None,
            int_config: None,
            fifo_config: None,
            gyro_config: None,
            accel_config: None,
            user_ctrl_config: None,
            power_management_config: None,
        }
    }

    pub async fn read_register(&mut self, register: u8) -> u8 {
        let address = register | READ_MASK;
        let mut read_into = [0x00; 1];

        self.bus.send_then_read(&[address], &mut read_into).await;

        read_into[0]
    }

    pub async fn write_register(&mut self, register: u8, value: u8) {
        let address = register & WRITE_MASK;
        self.bus.send(&[address, value]).await;
    }

    pub async fn set_interrupt_status(&mut self) {
        let mut read_into = [0x00; 1];
        let address = INT_STATUS | READ_MASK;

        self.bus.send_then_read(&[address], &mut read_into).await;
        self.latest_interrupts = self.latest_interrupts | read_into[0];
    }

    pub fn test_interrupt_status(&mut self, status: InterruptStatus) -> bool {
        let mask = status.bits();
        let result = self.latest_interrupts & mask != 0;

        // Claer status bit to mark it as processed
        self.latest_interrupts &= !mask;

        result
    }

    pub async fn read_accel(&mut self) -> (i16, i16, i16) {
        let start_address = ACCEL_XOUT_H | READ_MASK;
        let mut read_into = [0u8; 6];

        self.bus
            .send_then_read(&[start_address], &mut read_into)
            .await;

        let [x_high, x_low, y_high, y_low, z_high, z_low] = read_into;
        let x = i16::from_be_bytes([x_high, x_low]);
        let y = i16::from_be_bytes([y_high, y_low]);
        let z = i16::from_be_bytes([z_high, z_low]);

        (x, y, z)
    }

    pub async fn read_gyro(&mut self) -> (i16, i16, i16) {
        let start_address = GYRO_XOUT_H | READ_MASK;
        let mut read_into = [0u8; 6];

        self.bus
            .send_then_read(&[start_address], &mut read_into)
            .await;

        let [x_high, x_low, y_high, y_low, z_high, z_low] = read_into;
        let x = i16::from_be_bytes([x_high, x_low]);
        let y = i16::from_be_bytes([y_high, y_low]);
        let z = i16::from_be_bytes([z_high, z_low]);

        (x, y, z)
    }

    pub async fn drain_fifo(&mut self, buffer: &mut [u8]) {
        let address = FIFO_R_W | READ_MASK;
        self.bus.send_then_read(&[address], buffer).await;
    }

    pub async fn fifo_layout(&mut self) -> FIFOLayout {
        let address = FIFO_EN | READ_MASK;
        let mut read_into = [0];

        self.bus.send_then_read(&[address], &mut read_into).await;

        let fifo_layout = FIFOLayout::from_fifo_register(read_into[0]);

        fifo_layout
    }

    // TODO: Check if we should handle a case where the total count changes (prob not)
    pub async fn total_fifo_bytes(&mut self) -> u16 {
        let fifo_layout = self.fifo_layout().await;
        let fifo_sample_count = self.fifo_sample_count().await;

        fifo_sample_count * fifo_layout.size as u16
    }

    pub async fn fifo_sample_count(&mut self) -> u16 {
        let start_address = FIFO_COUNT_H | READ_MASK;
        let mut read_into = [0u8; 2];

        self.bus
            .send_then_read(&[start_address], &mut read_into)
            .await;

        let [count_high, count_low] = read_into;
        let count_high_mask = 0x1F;
        let total_count = u16::from_be_bytes([count_high & count_high_mask, count_low]);

        total_count
    }

    // TODO: Handle the case where we are resetting via SPI
    // SPI reset should include 100ms pause and separate reset for each sensor (gyro, accel, temp)
    pub async fn reset_device(&mut self) {
        let reset_bit = 1 << 7;
        self.set_power_mng_1_bit(reset_bit, false).await;
    }

    pub async fn set_sleep_mode(&mut self, sleep: bool) {
        let sleep_bit = 1 << 6;
        self.set_power_mng_1_bit(sleep_bit, sleep).await;
    }

    pub async fn set_cycle_mode(&mut self, cycle: bool) {
        let cycle_bit = 1 << 5;
        self.set_power_mng_1_bit(cycle_bit, cycle).await;
    }

    pub async fn set_gyro_standby(&mut self, standby: bool) {
        let gyro_standby_bit = 1 << 4;
        self.set_power_mng_1_bit(gyro_standby_bit, standby).await;
    }

    pub async fn set_temp_disabled(&mut self, disabled: bool) {
        let temp_disabled_bit = 1 << 3;
        self.set_power_mng_1_bit(temp_disabled_bit, disabled).await;
    }

    pub async fn disable_accel_axes(&mut self, axes: [bool; 3]) {
        let mut curr = [0x00; 1];
        let x_mask = 1 << 5;
        let y_mask = 1 << 4;
        let z_mask = 1 << 3;

        self.bus
            .send_then_read(&[PWR_MGMT_2 | READ_MASK], &mut curr)
            .await;

        let updated = if axes[0] { curr[0] | x_mask } else { curr[0] & !x_mask };
        let updated = if axes[1] { updated | y_mask } else { updated & !y_mask };
        let updated = if axes[2] { updated | z_mask } else { updated & !z_mask };

        self.bus.send(&[PWR_MGMT_2 | WRITE_MASK, updated]).await;
    }

    pub async fn disable_gyro_axes(&mut self, axes: [bool; 3]) {
        let mut curr = [0x00; 1];
        let x_mask = 1 << 2;
        let y_mask = 1 << 1;
        let z_mask = 1 << 0;

        self.bus
            .send_then_read(&[PWR_MGMT_2 | READ_MASK], &mut curr)
            .await;

        let updated = if axes[0] { curr[0] | x_mask } else { curr[0] & !x_mask };
        let updated = if axes[1] { updated | y_mask } else { updated & !y_mask };
        let updated = if axes[2] { updated | z_mask } else { updated & !z_mask };

        self.bus.send(&[PWR_MGMT_2 | WRITE_MASK, updated]).await;
    }

    async fn set_power_mng_1_bit(&mut self, bit: u8, enabled: bool) {
        let mut current = [0x00; 1];

        self.bus
            .send_then_read(&[PWR_MGMT_1 | READ_MASK], &mut current)
            .await;

        let updated = if enabled { current[0] | bit } else { current[0] & !bit };

        self.bus.send(&[PWR_MGMT_1 | WRITE_MASK, updated]).await;
    }
}
