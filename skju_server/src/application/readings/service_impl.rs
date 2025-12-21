use super::ReadingService;
use crate::application::messages::AppMessage;
use crate::domain::reading::{Reading, ReadingCreate, ReadingError, ReadingsRange};
use crate::ports::bus_service::{BusMessage, BusService};
use crate::ports::reading_repository::ReadingRepository;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Clone)]
pub struct Service {
    repository: Arc<dyn ReadingRepository>,
    bus_service: Arc<dyn BusService<AppMessage>>,
}

impl Service {
    pub fn new(repository: Arc<dyn ReadingRepository>, bus_service: Arc<dyn BusService<AppMessage>>) -> Self {
        Self { repository, bus_service }
    }
}

#[async_trait]
impl ReadingService for Service {
    async fn create(&self, request: ReadingCreate) -> Result<(), ReadingError> {
        let message = BusMessage {
            message: AppMessage::SensorReadingReceived(request),
        };

        self.bus_service
            .send(message)
            .await
            .map_err(|_| ReadingError::Internal("Failed to publish reading".into()))?;

        Ok(())
    }

    async fn get_between(&self, request: ReadingsRange) -> Result<Vec<Reading>, ReadingError> {
        self.repository.get_between(request).await
    }
}
