use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use crate::constants::{BLE_BATCH_SIZE, DEVICE_ID_BYTES, MAX_SAMPLE_COUNT, SAMPLE_BYTES, TIMESTAMP_BYTES};

pub type ReadingsChannel = Channel<CriticalSectionRawMutex, Readings, 1>;

#[derive(Debug, Clone, Copy)]
pub struct Readings {
    pub device_id: u64,
    pub batch_timestamp: u64,
    pub readings: [u8; MAX_SAMPLE_COUNT * SAMPLE_BYTES],
}

impl Readings {
    pub fn adjust_timestamp(&mut self, timestamp_diff: i64) -> Result<(), &'static str> {
        let new_timestamp = self.batch_timestamp as i64 + timestamp_diff;

        if new_timestamp < 0 {
            return Err("somehow got a negative timestamp");
        }

        self.batch_timestamp = new_timestamp as u64;
        Ok(())
    }

    pub fn print(&self) {
        let samples_count = self.readings.len() / SAMPLE_BYTES;
        let samples = self.readings;

        for i in 0..samples_count {
            let offset = i * SAMPLE_BYTES;

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

impl From<[u8; BLE_BATCH_SIZE]> for Readings {
    fn from(batch: [u8; BLE_BATCH_SIZE]) -> Self {
        let mut offset = 0;
        let mut device_id = [0u8; DEVICE_ID_BYTES];
        let mut timestamp = [0u8; TIMESTAMP_BYTES];
        let mut readings = [0u8; SAMPLE_BYTES * MAX_SAMPLE_COUNT];

        device_id.copy_from_slice(&batch[offset..offset + DEVICE_ID_BYTES]);
        offset += DEVICE_ID_BYTES;

        timestamp.copy_from_slice(&batch[offset..offset + TIMESTAMP_BYTES]);
        offset += TIMESTAMP_BYTES;

        readings.copy_from_slice(&batch[offset..]);

        Self {
            device_id: u64::from_be_bytes(device_id),
            batch_timestamp: u64::from_be_bytes(timestamp),
            readings,
        }
    }
}

impl From<Readings> for [u8; BLE_BATCH_SIZE] {
    fn from(value: Readings) -> Self {
        let mut batch = [0u8; BLE_BATCH_SIZE];
        let mut offset = 0;

        batch[offset..offset + DEVICE_ID_BYTES].copy_from_slice(&value.device_id.to_be_bytes());
        offset += DEVICE_ID_BYTES;

        batch[offset..offset + TIMESTAMP_BYTES].copy_from_slice(&value.batch_timestamp.to_be_bytes());
        offset += TIMESTAMP_BYTES;

        batch[offset..].copy_from_slice(&value.readings);

        batch
    }
}
