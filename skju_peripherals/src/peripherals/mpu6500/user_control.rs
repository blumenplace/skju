bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct UserControlFlags : u8 {
        const DMP_EN = 1 << 7;
        const FIFO_EN = 1 << 6;
        const I2C_MST_EN = 1 << 5;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UserControlConfig {
    pub flags: UserControlFlags,
}

impl Default for UserControlConfig {
    fn default() -> Self {
        Self { flags: UserControlFlags::empty() }
    }
}

impl UserControlConfig {
    pub fn flags(mut self, flags: UserControlFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn enable_dmp(mut self) -> Self {
        self.flags.insert(UserControlFlags::DMP_EN);
        self
    }

    pub fn enable_fifo(mut self) -> Self {
        self.flags.insert(UserControlFlags::FIFO_EN);
        self
    }

    pub fn enable_i2c_master(mut self) -> Self {
        self.flags.insert(UserControlFlags::I2C_MST_EN);
        self
    }

    pub fn bits(&self) -> u8 {
        self.flags.bits()
    }
}
