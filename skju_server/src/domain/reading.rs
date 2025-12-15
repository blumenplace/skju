use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;

#[derive(Debug, Clone, Serialize, FromRow)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct ReadingGetBetweenRequest {
    pub sensor_id: Option<i32>,
    pub from: DateTime<Utc>,
    pub to: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub enum ReadingError {
    Database(String),
    Internal(String),
}

impl fmt::Display for ReadingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadingError::Database(e) => write!(formatter, "Database error: {}", e),
            ReadingError::Internal(e) => write!(formatter, "Internal error: {}", e),
        }
    }
}
