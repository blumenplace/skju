use crate::domain::reading::{Reading, ReadingCreate, ReadingError, ReadingsRange};
use async_trait::async_trait;

#[async_trait]
pub trait ReadingService: Send + Sync + 'static {
    async fn create(&self, req: ReadingCreate) -> Result<(), ReadingError>;
    async fn get_between(&self, req: ReadingsRange) -> Result<Vec<Reading>, ReadingError>;
}
