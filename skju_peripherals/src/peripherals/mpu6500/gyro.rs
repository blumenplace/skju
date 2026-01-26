bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct SelfTestFlags : u8 {
        const X_SELF_TEST = 1 << 7;
        const Y_SELF_TEST = 1 << 6;
        const Z_SELF_TEST = 1 << 5;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GyroConfig {
    pub range: GyroRange,
    pub st_flags: SelfTestFlags,
}

impl Default for GyroConfig {
    fn default() -> Self {
        Self {
            range: GyroRange::R250dps,
            st_flags: SelfTestFlags::empty(),
        }
    }
}

impl GyroConfig {
    pub fn range(mut self, range: GyroRange) -> Self {
        self.range = range;
        self
    }

    pub fn flags(mut self, flags: SelfTestFlags) -> Self {
        self.st_flags = flags;
        self
    }

    pub fn bits(&self) -> u8 {
        self.st_flags.bits() | (self.range.bits() << 3)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GyroRange {
    R250dps,
    R500dps,
    R1000dps,
    R2000dps,
}

impl GyroRange {
    fn bits(self) -> u8 {
        match self {
            GyroRange::R250dps => 0b00,
            GyroRange::R500dps => 0b01,
            GyroRange::R1000dps => 0b10,
            GyroRange::R2000dps => 0b11,
        }
    }
}
