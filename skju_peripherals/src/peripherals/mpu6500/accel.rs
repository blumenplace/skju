bitflags::bitflags! {
    pub struct AccelFlags : u8 {
        const X_SELF_TEST = 1 << 7;
        const Y_SELF_TEST = 1 << 6;
        const Z_SELF_TEST = 1 << 5;
    }
}

pub struct AccelConfig {
    pub range: AccelRange,
    pub flags: AccelFlags,
}

impl AccelConfig {
    pub fn range(mut self, range: AccelRange) -> Self {
        self.range = range;
        self
    }
    
    pub fn flags(mut self, flags: AccelFlags) -> Self {
        self.flags = flags;
        self
    }
}

impl Default for AccelConfig {
    fn default() -> Self {
        Self {
            range: AccelRange::G2,
            flags: AccelFlags::empty(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccelRange {
    G2,
    G4,
    G8,
    G16,
}

impl AccelRange {
    fn bits(self) -> u8 {
        match self {
            AccelRange::G2 => 0b00,
            AccelRange::G4 => 0b01,
            AccelRange::G8 => 0b10,
            AccelRange::G16 => 0b11,
        }
    }
}
