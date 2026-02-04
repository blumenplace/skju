use crate::domain::sensor::{Sensor, SensorCreate, SensorError, SensorID, SensorUpdate};
use async_trait::async_trait;

#[async_trait]
pub trait SensorRepository: Send + Sync + 'static {
    async fn create(&self, request: SensorCreate) -> Result<Sensor, SensorError>;
    async fn update(&self, id: SensorID, request: SensorUpdate) -> Result<Sensor, SensorError>;
    async fn delete(&self, id: SensorID) -> Result<(), SensorError>;
    async fn list(&self) -> Result<Vec<Sensor>, SensorError>;
    async fn get_by_id(&self, id: SensorID) -> Result<Sensor, SensorError>;
    async fn delete_all(&self) -> Result<(), SensorError>;
}
