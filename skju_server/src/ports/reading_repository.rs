use crate::domain::reading::{Reading, ReadingCreate, ReadingError, ReadingsRange};
use async_trait::async_trait;

#[async_trait]
pub trait ReadingRepository: Send + Sync + 'static {
    async fn create(&self, request: Vec<ReadingCreate>) -> Result<(), ReadingError>;
    async fn get_between(&self, request: ReadingsRange) -> Result<Vec<Reading>, ReadingError>;
}
