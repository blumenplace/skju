use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize)]
pub struct Reading {
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

#[derive(Debug)]
pub enum ReadingError {
    NotFound,
    DatabaseError(String),
    ValidationError(String),
}

impl fmt::Display for ReadingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadingError::NotFound => write!(formatter, "Sensor not found"),
            ReadingError::DatabaseError(e) => write!(formatter, "Database error: {}", e),
            ReadingError::ValidationError(msg) => write!(formatter, "Validation error: {}", msg),
        }
    }
}

impl From<sqlx::Error> for ReadingError {
    fn from(err: sqlx::Error) -> Self {
        ReadingError::DatabaseError(err.to_string())
    }
}
