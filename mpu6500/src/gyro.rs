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
    pub f_choice_b: u8,
}

impl Default for GyroConfig {
    fn default() -> Self {
        Self {
            range: GyroRange::R250dps,
            st_flags: SelfTestFlags::empty(),
            f_choice_b: 0,
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

    /// FCHOICE_B bits [1; 0], value should be {0, 1, 2, 3}
    pub fn f_choice_b(mut self, f_choice: u8) -> Self {
        self.f_choice_b = f_choice;
        self
    }

    pub fn bits(&self) -> u8 {
        self.st_flags.bits() | ((self.range as u8) << 3) | (self.f_choice_b & 0b11)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GyroRange {
    R250dps = 0b00,
    R500dps = 0b01,
    R1000dps = 0b10,
    R2000dps = 0b11,
}
