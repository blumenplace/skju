use core::clone::Clone;
use core::fmt::Debug;
use core::marker::Copy;

pub const MAX_FIFO_BUFFER_SIZE: usize = 512;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct FIFOSensors : u8 {
        const TEMP = 1 << 7;
        const GYRO_X = 1 << 6;
        const GYRO_Y = 1 << 5;
        const GYRO_Z = 1 << 4;
        const ACCEL  = 1 << 3;
        const SLV_2 = 1 << 2;
        const SLV_1 = 1 << 1;
        const SLV_0 = 1 << 0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FIFOMode {
    Override,
    StopWhenFull,
}

pub struct FIFOConfig {
    pub mode: FIFOMode,
    pub sensors: FIFOSensors,
}

impl FIFOConfig {
    pub fn mode(mut self, mode: FIFOMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn sensors(mut self, sensors: FIFOSensors) -> Self {
        self.sensors = sensors;
        self
    }
}

impl Default for FIFOConfig {
    fn default() -> Self {
        Self {
            mode: FIFOMode::Override,
            sensors: FIFOSensors::ACCEL | FIFOSensors::GYRO_X | FIFOSensors::GYRO_Y | FIFOSensors::GYRO_Z,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FIFOEntryType {
    AccelX,
    AccelY,
    AccelZ,
    Temp,
    GyroX,
    GyroY,
    GyroZ,
}

#[derive(Clone, Copy, Debug)]
pub struct FIFOEntry {
    pub entry_type: FIFOEntryType,
    pub offset: usize,
}

impl FIFOEntry {
    pub fn new(entry_type: FIFOEntryType, offset: usize) -> Self {
        Self { entry_type, offset }
    }
}

pub struct FIFOLayout {
    pub fields: heapless::Vec<FIFOEntry, 7>,
    pub sample_size: usize,
}

impl FIFOLayout {
    pub fn from_fifo_register(fifo_en: u8) -> Self {
        let mut fields = heapless::Vec::<FIFOEntry, 7>::new();
        let mut offset = 0;
        let error_message = "Unexpected number of FIFO fields enabled";

        if fifo_en & FIFOSensors::ACCEL.bits() != 0 {
            fields
                .push(FIFOEntry::new(FIFOEntryType::AccelX, offset))
                .expect(&error_message);
            fields
                .push(FIFOEntry::new(FIFOEntryType::AccelY, offset + 2))
                .expect(&error_message);
            fields
                .push(FIFOEntry::new(FIFOEntryType::AccelZ, offset + 4))
                .expect(&error_message);
            offset += 6;
        }

        if fifo_en & FIFOSensors::TEMP.bits() != 0 {
            fields
                .push(FIFOEntry::new(FIFOEntryType::Temp, offset))
                .expect(&error_message);
            offset += 2;
        }

        if fifo_en & FIFOSensors::GYRO_X.bits() != 0 {
            fields
                .push(FIFOEntry::new(FIFOEntryType::GyroX, offset))
                .expect(&error_message);
            offset += 2;
        }

        if fifo_en & FIFOSensors::GYRO_Y.bits() != 0 {
            fields
                .push(FIFOEntry::new(FIFOEntryType::GyroY, offset))
                .expect(&error_message);
            offset += 2;
        }

        if fifo_en & FIFOSensors::GYRO_Z.bits() != 0 {
            fields
                .push(FIFOEntry::new(FIFOEntryType::GyroZ, offset))
                .expect(&error_message);
            offset += 2;
        }

        Self { fields, sample_size: offset }
    }
}

pub struct FIFOSample<'a> {
    data: &'a [u8],
    layout: &'a FIFOLayout,
}

impl<'a> FIFOSample<'a> {
    pub fn new(data: &'a [u8], layout: &'a FIFOLayout) -> Self {
        Self { data, layout }
    }

    pub fn get_value(&self, entry_type: FIFOEntryType) -> Option<i16> {
        let entry_offset = self
            .layout
            .fields
            .iter()
            .find(|f| f.entry_type == entry_type)
            .map(|f| f.offset)?;

        let value = i16::from_be_bytes([self.data[entry_offset], self.data[entry_offset + 1]]);
        Some(value)
    }
}
