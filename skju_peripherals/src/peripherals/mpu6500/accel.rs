bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct SelfTestFlags : u8 {
        const X_SELF_TEST = 1 << 7;
        const Y_SELF_TEST = 1 << 6;
        const Z_SELF_TEST = 1 << 5;
    }
}

bitflags::bitflags! {
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
        let mut accel_two_bits = self.dlpf_cfg.bits();

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DLPFOptions {
    Hz460,
    Hz184,
    Hz92,
    Hz41,
    Hz20,
    Hz10,
    Hz5,
}

impl DLPFOptions {
    pub fn bits(self) -> u8 {
        match self {
            DLPFOptions::Hz460 => 1,
            DLPFOptions::Hz184 => 2,
            DLPFOptions::Hz92 => 3,
            DLPFOptions::Hz41 => 4,
            DLPFOptions::Hz20 => 5,
            DLPFOptions::Hz10 => 6,
            DLPFOptions::Hz5 => 7,
        }
    }
}
