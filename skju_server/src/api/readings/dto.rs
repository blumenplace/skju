use crate::domain::reading::{Reading, ReadingCreate, ReadingTimestamp, ReadingValue};
use crate::domain::sensor::SensorID;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ReadingModel {
    pub id: i64,
    pub sensor_id: i32,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReadingCreateRequest {
    pub sensor_id: i32,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReadingGetBetweenRequest {
    pub sensor_id: Option<i32>,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

impl From<Reading> for ReadingModel {
    fn from(reading: Reading) -> Self {
        ReadingModel {
            id: reading.id.value(),
            sensor_id: reading.sensor_id.value(),
            value: reading.value.value(),
            timestamp: reading.timestamp.value(),
        }
    }
}

impl From<ReadingCreateRequest> for ReadingCreate {
    fn from(request: ReadingCreateRequest) -> Self {
        ReadingCreate {
            sensor_id: SensorID::new(request.sensor_id),
            value: ReadingValue::new(request.value),
            timestamp: ReadingTimestamp::new(request.timestamp),
        }
    }
}
