// CONFIG
pub const WHO_AM_I: u8 = 0x75;
pub const CONFIG: u8 = 0x1A;
pub const GYRO_CONFIG: u8 = 0x1B;
pub const ACCEL_CONFIG: u8 = 0x1C;
pub const ACCEL_CONFIG_2: u8 = 0x1D;
pub const FIFO_EN: u8 = 0x23;
pub const USER_CTRL: u8 = 0x6A;

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
pub const FIFO_COUNT_H: u8 = 0x72; // [12:8]
pub const FIFO_COUNT_L: u8 = 0x73; // [7:0]
pub const FIFO_R_W: u8 = 0x74;
