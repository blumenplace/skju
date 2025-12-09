#[derive(Debug, Clone)]
pub struct AppState {
    pub data_dir: String,
}

impl AppState {
    pub fn new() -> Self {
        Self { data_dir: String::from("data") }
    }
}
