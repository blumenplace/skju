use super::SensorService;
use crate::application::messages::AppMessage;
use crate::domain::sensor::{Sensor, SensorCreate, SensorError, SensorID, SensorUpdate};
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
    async fn create(&self, request: SensorCreate) -> Result<Sensor, SensorError> {
        let result = self.repository.create(request).await?;
        Ok(result)
    }

    async fn update(&self, id: SensorID, request: SensorUpdate) -> Result<Sensor, SensorError> {
        let result = self.repository.update(id, request).await?;
        Ok(result)
    }

    async fn delete(&self, id: SensorID) -> Result<(), SensorError> {
        self.repository.delete(id).await?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Sensor>, SensorError> {
        let result = self.repository.list().await?;
        Ok(result)
    }

    async fn get_by_id(&self, id: SensorID) -> Result<Option<Sensor>, SensorError> {
        let result = self.repository.get_by_id(id).await?;
        Ok(result)
    }

    async fn delete_all(&self) -> Result<(), SensorError> {
        self.repository.delete_all().await?;
        Ok(())
    }
}
