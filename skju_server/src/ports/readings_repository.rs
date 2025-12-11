use crate::domain::reading::{Reading, ReadingCreateRequest};
use anyhow::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait ReadingsRepository {
    async fn create(&self, request: ReadingCreateRequest) -> Result<Reading, Error>;
    async fn get_range(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<Vec<Reading>, Error>;
}
