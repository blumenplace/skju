use crate::application::messages::AppMessage;
use crate::application::sensors::service::SensorService;
use crate::domain::sensor::{Sensor, SensorCreateRequest, SensorError, SensorUpdateRequest};
use crate::ports::bus_service::BusService;
use crate::ports::sensors_repository::SensorRepository;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Clone)]
pub struct Service {
    repository: Arc<dyn SensorRepository>,
    bus_service: Arc<dyn BusService<AppMessage>>,
}

impl Service {
    pub fn new(repository: Arc<dyn SensorRepository>, bus_service: Arc<dyn BusService<AppMessage>>) -> Self {
        Self { repository, bus_service }
    }
}

#[async_trait]
impl SensorService for Service {
    async fn create(&self, request: SensorCreateRequest) -> Result<Sensor, SensorError> {
        let result = self.repository.create(request).await?;
        Ok(result)
    }

    async fn update(&self, id: i32, request: SensorUpdateRequest) -> Result<Sensor, SensorError> {
        let result = self.repository.update(id, request).await?;
        Ok(result)
    }

    async fn delete(&self, id: i32) -> Result<(), SensorError> {
        self.repository.delete(id).await?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Sensor>, SensorError> {
        let result = self.repository.list().await?;
        Ok(result)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<Sensor>, SensorError> {
        let result = self.repository.get_by_id(id).await?;
        Ok(result)
    }

    async fn delete_all(&self) -> Result<(), SensorError> {
        self.repository.delete_all().await?;
        Ok(())
    }
}
