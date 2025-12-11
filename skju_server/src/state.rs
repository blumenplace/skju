use crate::application::sensors::SensorService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub sensor_service: Arc<dyn SensorService>,
}

impl AppState {
    pub fn new(sensor_service: Arc<dyn SensorService>) -> Self {
        Self { sensor_service }
    }
}
