use std::sync::Mutex;

#[derive(Default, Debug, uniffi::Object)]
#[uniffi::export(Debug)]
pub struct Summer {
    sum: Mutex<f64>,
}

#[uniffi::export]
impl Summer {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self { sum: Mutex::new(0.0) }
    }

    pub fn add(&self, value: f64) {
        *self.sum.lock().unwrap() += value;
    }

    pub fn sum(&self) -> f64 {
        *self.sum.lock().unwrap()
    }
}

uniffi::setup_scaffolding!();
