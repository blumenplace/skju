bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct SelfTestFlags : u8 {
        const X_SELF_TEST = 1 << 7;
        const Y_SELF_TEST = 1 << 6;
        const Z_SELF_TEST = 1 << 5;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct DLTFFlags : u8 {
        const FCHOICE_B = 1 << 3;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AccelConfig {
    pub range: AccelRange,
    pub st_flags: SelfTestFlags,
    pub dlpf_enabled: bool,
    pub dlpf_cfg: DLPFOptions,
}

impl AccelConfig {
    pub fn range(mut self, range: AccelRange) -> Self {
        self.range = range;
        self
    }

    pub fn flags(mut self, flags: SelfTestFlags) -> Self {
        self.st_flags = flags;
        self
    }

    pub fn dlpf(mut self, enabled: bool, cfg: DLPFOptions) -> Self {
        self.dlpf_enabled = enabled;
        self.dlpf_cfg = cfg;
        self
    }

    pub fn bits(&self) -> [u8; 2] {
        let accel_one_bits = self.st_flags.bits();
        let mut accel_two_bits = self.dlpf_cfg as u8;

        if self.dlpf_enabled {
            accel_two_bits = 1 << 3;
        }

        [accel_one_bits, accel_two_bits]
    }
}

impl Default for AccelConfig {
    fn default() -> Self {
        Self {
            range: AccelRange::G2,
            st_flags: SelfTestFlags::empty(),
            dlpf_enabled: true,
            dlpf_cfg: DLPFOptions::Hz460,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AccelRange {
    G2 = 0b00,
    G4 = 0b01,
    G8 = 0b10,
    G16 = 0b11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DLPFOptions {
    Hz460 = 1,
    Hz184 = 2,
    Hz92 = 3,
    Hz41 = 4,
    Hz20 = 5,
    Hz10 = 6,
    Hz5 = 7,
}
