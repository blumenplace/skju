use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use crate::constants::{BLE_BATCH_SIZE, MAX_SAMPLE_COUNT, SAMPLE_SIZE, TIMESTAMP_BYTES};

pub type ReadingsChannel = Channel<CriticalSectionRawMutex, Readings, 1>;

pub struct Readings {
    pub batch_timestamp: u64,
    pub readings: [u8; MAX_SAMPLE_COUNT * SAMPLE_SIZE],
}

impl Readings {
    pub fn bytes(&self) -> [u8; BLE_BATCH_SIZE] {
        let timestamp_bytes: [u8; TIMESTAMP_BYTES] = self.batch_timestamp.to_be_bytes();
        let mut batch_bytes = [0x00; BLE_BATCH_SIZE];

        batch_bytes[..TIMESTAMP_BYTES].copy_from_slice(&timestamp_bytes);
        batch_bytes[TIMESTAMP_BYTES..].copy_from_slice(&self.readings[..BLE_BATCH_SIZE]);

        batch_bytes
    }

    pub fn adjust_timestamp(&mut self, timestamp_diff: i64) -> Result<(), &'static str> {
        let new_timestamp = self.batch_timestamp as i64 + timestamp_diff;

        if new_timestamp < 0 {
            return Err("somehow got a negative timestamp");
        }

        self.batch_timestamp = new_timestamp as u64;
        Ok(())
    }

    pub fn print(&self) {
        let samples_count = self.readings.len() / SAMPLE_SIZE;
        let samples = self.readings;

        for i in 0..samples_count {
            let offset = i * SAMPLE_SIZE;

            let ax = i16::from_be_bytes([samples[offset + 0], samples[offset + 1]]);
            let ay = i16::from_be_bytes([samples[offset + 2], samples[offset + 3]]);
            let az = i16::from_be_bytes([samples[offset + 4], samples[offset + 5]]);

            let gx = i16::from_be_bytes([samples[offset + 6], samples[offset + 7]]);
            let gy = i16::from_be_bytes([samples[offset + 8], samples[offset + 9]]);
            let gz = i16::from_be_bytes([samples[offset + 10], samples[offset + 11]]);

            defmt::info!("S{} ACC[x:{} y:{} z:{}] GYR[x:{} y:{} z:{}]", i, ax, ay, az, gx, gy, gz);
        }
    }
}
