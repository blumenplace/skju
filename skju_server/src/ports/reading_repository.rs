use crate::domain::reading::{Reading, ReadingCreateRequest, ReadingError, ReadingGetBetweenRequest};
use async_trait::async_trait;

#[async_trait]
pub trait ReadingRepository: Send + Sync + 'static {
    async fn create(&self, request: Vec<ReadingCreateRequest>) -> Result<(), ReadingError>;
    async fn get_between(&self, request: ReadingGetBetweenRequest) -> Result<Vec<Reading>, ReadingError>;
}
