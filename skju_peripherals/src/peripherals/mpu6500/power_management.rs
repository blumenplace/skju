bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct DisableBits: u8 {
        const ACCEL_X = 1 << 5;
        const ACCEL_Y = 1 << 4;
        const ACCEL_Z = 1 << 3;
        const GYRO_X = 1 << 2;
        const GYRO_Y = 1 << 1;
        const GYRO_Z = 1 << 0;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct DeviceModeBits: u8 {
        const SLEEP = 1 << 6;
        const CYCLE = 1 << 5;
        const GYRO_STANDBY = 1 << 4;
        const TEMP_DISABLED = 1 << 3;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PowerManagementConfig {
    pub disable_bits: DisableBits,
    pub device_mode_bits: DeviceModeBits,
}

impl Default for PowerManagementConfig {
    fn default() -> Self {
        Self {
            disable_bits: DisableBits::empty(),
            device_mode_bits: DeviceModeBits::empty(),
        }
    }
}

impl PowerManagementConfig {
    pub fn disable_bits(mut self, disable_bits: DisableBits) -> Self {
        self.disable_bits = disable_bits;
        self
    }

    pub fn device_mode_bits(mut self, device_mode_bits: DeviceModeBits) -> Self {
        self.device_mode_bits = device_mode_bits;
        self
    }

    pub fn bits(&self) -> [u8; 2] {
        [self.device_mode_bits.bits(), self.disable_bits.bits()]
    }
}
