use super::sensor::SensorID;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct ReadingID(i64);

#[derive(Debug, Clone, Copy)]
pub struct ReadingValue(f64);

#[derive(Debug, Clone, Copy)]
pub struct ReadingTimestamp(DateTime<Utc>);

#[derive(Debug, FromRow)]
pub struct DBReading {
    pub id: i64,
    pub sensor_id: i32,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
}

pub struct Reading {
    pub id: ReadingID,
    pub sensor_id: SensorID,
    pub value: ReadingValue,
    pub timestamp: ReadingTimestamp,
}

#[derive(Debug)]
pub struct ReadingCreate {
    pub sensor_id: SensorID,
    pub value: ReadingValue,
    pub timestamp: ReadingTimestamp,
}

#[derive(Debug)]
pub struct ReadingsRange {
    sensor_id: Option<SensorID>,
    from: ReadingTimestamp,
    to: ReadingTimestamp,
}

#[derive(Debug)]
pub enum ReadingError {
    InvalidRange(String),
    Database(String),
    Internal(String),
}

impl fmt::Display for ReadingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadingError::Database(e) => write!(formatter, "Database error: {}", e),
            ReadingError::Internal(e) => write!(formatter, "Internal error: {}", e),
            ReadingError::InvalidRange(e) => write!(formatter, "Invalid range: {}", e),
        }
    }
}

impl From<DBReading> for Reading {
    fn from(db_reading: DBReading) -> Self {
        Reading {
            id: ReadingID::new(db_reading.id),
            sensor_id: SensorID::new(db_reading.sensor_id),
            value: ReadingValue::new(db_reading.value),
            timestamp: ReadingTimestamp::new(db_reading.timestamp),
        }
    }
}

impl From<Reading> for DBReading {
    fn from(reading: Reading) -> Self {
        DBReading {
            id: reading.id.value(),
            sensor_id: reading.sensor_id.value(),
            value: reading.value.value(),
            timestamp: reading.timestamp.value(),
        }
    }
}

impl ReadingsRange {
    pub fn new(
        sensor_id: Option<SensorID>,
        from: ReadingTimestamp,
        to: ReadingTimestamp,
    ) -> Result<Self, ReadingError> {
        let error_message = String::from("readings 'from' should be before 'to'");

        if to.value() <= from.value() {
            return Err(ReadingError::InvalidRange(error_message));
        }

        Ok(Self { sensor_id, from, to })
    }

    pub fn sensor_id(&self) -> Option<SensorID> {
        self.sensor_id
    }

    pub fn from(&self) -> ReadingTimestamp {
        self.from
    }

    pub fn to(&self) -> ReadingTimestamp {
        self.to
    }
}

impl ReadingID {
    pub fn new(id: i64) -> Self {
        ReadingID(id)
    }

    pub fn value(&self) -> i64 {
        self.0
    }
}

impl ReadingValue {
    pub fn new(value: f64) -> Self {
        ReadingValue(value)
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

impl ReadingTimestamp {
    pub fn new(timestamp: DateTime<Utc>) -> Self {
        ReadingTimestamp(timestamp)
    }

    pub fn value(&self) -> DateTime<Utc> {
        self.0
    }
}
