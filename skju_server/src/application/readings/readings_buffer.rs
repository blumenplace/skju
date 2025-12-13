use crate::domain::reading::{ReadingCreateRequest, ReadingError};
use crate::ports::reading_repository::ReadingRepository;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ReadingsBuffer {
    pub readings: Mutex<Vec<ReadingCreateRequest>>,
    pub max_readings: usize,
    pub readings_repository: Arc<dyn ReadingRepository>,
}

impl ReadingsBuffer {
    pub fn new(readings_repository: Arc<dyn ReadingRepository>, max_readings: usize) -> Self {
        Self {
            readings: Mutex::new(Vec::new()),
            max_readings,
            readings_repository,
        }
    }

    pub async fn flush(&self) -> Result<(), ReadingError> {
        let readings_to_commit = {
            let mut readings = self.readings.lock().await;
            readings.drain(..).collect::<Vec<ReadingCreateRequest>>()
        };

        self.try_commit(readings_to_commit).await
    }

    pub async fn add(&self, reading: ReadingCreateRequest) -> Result<(), ReadingError> {
        let readings_to_commit = {
            let mut readings = self.readings.lock().await;

            readings.push(reading);

            if readings.len() >= self.max_readings {
                readings.drain(..).collect::<Vec<ReadingCreateRequest>>()
            } else {
                vec![]
            }
        };

        self.try_commit(readings_to_commit).await?;

        Ok(())
    }

    async fn try_commit(&self, readings: Vec<ReadingCreateRequest>) -> Result<(), ReadingError> {
        if !readings.is_empty() {
            self.readings_repository.create(readings).await?;
        }

        Ok(())
    }
}
