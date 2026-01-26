bitflags::bitflags! {
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
