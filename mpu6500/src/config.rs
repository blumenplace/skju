#[derive(Debug, Clone, Copy)]
pub struct MPU6500Config {
    pub ext_sync: ExtSyncOptions,
    pub dlpf_cfg: ConfigDLPFOptions,
}

impl Default for MPU6500Config {
    fn default() -> Self {
        Self {
            ext_sync: ExtSyncOptions::Disabled,
            dlpf_cfg: ConfigDLPFOptions::CFG0,
        }
    }
}

impl MPU6500Config {
    pub fn ext_sync(mut self, ext_sync: ExtSyncOptions) -> Self {
        self.ext_sync = ext_sync;
        self
    }

    pub fn dlpf_cfg(mut self, dlpf_cfg: ConfigDLPFOptions) -> Self {
        self.dlpf_cfg = dlpf_cfg;
        self
    }

    pub fn bits(&self) -> u8 {
        0 | self.ext_sync.bits() | self.dlpf_cfg.bits()
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ExtSyncOptions {
    Disabled = 0,
    TempOutL = 1,
    GyroXOutL = 2,
    GyroYOutL = 3,
    GyroZOutL = 4,
    AccelXOutL = 5,
    AccelYOutL = 6,
    AccelZOutL = 7,
}

impl ExtSyncOptions {
    pub fn bits(self) -> u8 {
        let value = self as u8;
        value << 3
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ConfigDLPFOptions {
    CFG0 = 0,
    CFG1 = 1,
    CFG2 = 2,
    CFG3 = 3,
    CFG4 = 4,
    CFG5 = 5,
    CFG6 = 6,
    CFG7 = 7,
}

impl ConfigDLPFOptions {
    pub fn bits(self) -> u8 {
        let value = self as u8;
        value
    }
}
