use crate::domain::sensor::{Sensor, SensorCreateRequest, SensorError, SensorUpdateRequest};
use async_trait::async_trait;

#[async_trait]
pub trait SensorService: Send + Sync + 'static {
    async fn create(&self, req: SensorCreateRequest) -> Result<Sensor, SensorError>;
    async fn update(&self, id: i32, req: SensorUpdateRequest) -> Result<Sensor, SensorError>;
    async fn delete(&self, id: i32) -> Result<(), SensorError>;
    async fn list(&self) -> Result<Vec<Sensor>, SensorError>;
    async fn get_by_id(&self, id: i32) -> Result<Option<Sensor>, SensorError>;
    async fn delete_all(&self) -> Result<(), SensorError>;
}
