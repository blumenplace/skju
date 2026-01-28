mod accel;
mod builder;
mod fifo;
mod gyro;
mod registers;
mod user_control;

use crate::bus::Bus;
use core::option::Option::{None};
use crate::peripherals::mpu6500::fifo::FIFOLayout;
use builder::*;
use registers::*;

impl<T: Bus> MPU6500<T> {
    pub fn builder() -> MPU6500Builder<NoBus> {
        MPU6500Builder {
            bus: NoBus,
            gyro_config: None,
            accel_config: None,
            fifo_config: None,
            user_ctrl_config: None,
        }
    }

    pub async fn read_accel(&mut self) -> (i16, i16, i16) {
        let start_address = ACCEL_XOUT_H | 0x80;
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
        let start_address = GYRO_XOUT_H | 0x80;
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
        let address = FIFO_R_W | 0x80;
        self.bus.send_then_read(&[address], buffer).await;
    }

    pub async fn fifo_layout(&mut self) -> FIFOLayout {
        let address = FIFO_EN | 0x80;
        let mut read_into = [0];

        self.bus
            .send_then_read(&[address], &mut read_into)
            .await;

        let fifo_layout = FIFOLayout::from_fifo_register(read_into[0]);

        fifo_layout
    }
    
    // Do we need to handle a case where the total count changes before draining the fifo?
    pub async fn total_fifo_bytes(&mut self) -> u16 {
        let fifo_layout = self.fifo_layout().await;
        let fifo_sample_count = self.fifo_sample_count().await;
        
        fifo_sample_count * fifo_layout.size as u16
    }

    async fn fifo_sample_count(&mut self) -> u16 {
        let start_address = FIFO_COUNT_H | 0x80;
        let mut read_into = [0u8; 2];

        self.bus
            .send_then_read(&[start_address], &mut read_into)
            .await;

        let [count_high, count_low] = read_into;
        let count_high_mask = 0x1F;
        let total_count = u16::from_be_bytes([count_high & count_high_mask, count_low]);

        total_count
    }
}
