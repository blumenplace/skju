use crate::application::readings::ReadingService;
use crate::application::sensors::SensorService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppServices {
    pub sensor_service: Arc<dyn SensorService>,
    pub reading_service: Arc<dyn ReadingService>,
}

#[derive(Clone)]
pub struct AppState {
    pub app_services: AppServices,
}

impl AppState {
    pub fn new(app_services: AppServices) -> Self {
        Self { app_services }
    }
}
