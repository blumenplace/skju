bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct INTFlags : u8 {
        const ACTL = 1 << 7;
        const OPEN = 1 << 6;
        const LATCH_INT_EN = 1 << 5;
        const INT_ANYRD_2CLEAR = 1 << 4;
        const ACTL_FSYNC = 1 << 3;
        const FSYNC_INT_MODE_EN = 1 << 2;
        const BYPASS_EN = 1 << 1;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct INTEnableFlags : u8 {
        const WOM_EN = 1 << 6;
        const FIFO_OVERFLOW_EN = 1 << 4;
        const FSYNC_INT_EN = 1 << 4;
        const RAW_RDY_EN = 1 << 0;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct INTConfig {
    pub int_flags: INTFlags,
    pub int_enable_flags: INTEnableFlags,
}

impl Default for INTConfig {
    fn default() -> Self {
        Self {
            int_flags: INTFlags::empty(),
            int_enable_flags: INTEnableFlags::empty(),
        }
    }
}

impl INTConfig {
    pub fn int_flags(mut self, int_flags: INTFlags) -> Self {
        self.int_flags = int_flags;
        self
    }

    pub fn int_enable_flags(mut self, int_enable_flags: INTEnableFlags) -> Self {
        self.int_enable_flags = int_enable_flags;
        self
    }

    pub fn bits(&self) -> [u8; 2] {
        [self.int_flags.bits(), self.int_enable_flags.bits()]
    }
}
