use crate::domain::reading::{Reading, ReadingCreateRequest, ReadingError, ReadingGetBetweenRequest};
use async_trait::async_trait;

#[async_trait]
pub trait ReadingService: Send + Sync + 'static {
    async fn create(&self, req: ReadingCreateRequest) -> Result<(), ReadingError>;
    async fn get_between(&self, req: ReadingGetBetweenRequest) -> Result<Vec<Reading>, ReadingError>;
}
