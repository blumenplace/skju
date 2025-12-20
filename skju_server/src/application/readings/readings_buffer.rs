use crate::domain::reading::{ReadingCreate, ReadingError};
use crate::ports::reading_repository::ReadingRepository;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ReadingsBuffer {
    pub readings: Mutex<Vec<ReadingCreate>>,
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
            readings.drain(..).collect::<Vec<ReadingCreate>>()
        };

        self.try_commit(readings_to_commit).await
    }

    pub async fn add(&self, reading: ReadingCreate) -> Result<(), ReadingError> {
        let readings_to_commit = {
            let mut readings = self.readings.lock().await;

            readings.push(reading);

            if readings.len() >= self.max_readings {
                readings.drain(..).collect::<Vec<ReadingCreate>>()
            } else {
                vec![]
            }
        };

        self.try_commit(readings_to_commit).await?;

        Ok(())
    }

    async fn try_commit(&self, readings: Vec<ReadingCreate>) -> Result<(), ReadingError> {
        if !readings.is_empty() {
            self.readings_repository.create(readings).await?;
        }

        Ok(())
    }
}
