//! This library implements an abstraction layer over the MPU6500 module.
//!
//! The purpose of this library is to provide a simple way to initialize MPU module and
//! execute commonly used tasks such as resetting the device, reading/writing registers, or
//! draining the FIFO buffer.

pub mod accel;
pub mod builder;
pub mod config;
pub mod fifo;
pub mod gyro;
pub mod interrupts;
pub mod power_management;
pub mod registers;
pub mod user_control;

mod mpu6500;
mod utils;

pub use mpu6500::*;
