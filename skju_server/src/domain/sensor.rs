use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize)]
pub struct Sensor {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub x: f64,
    pub y: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SensorCreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SensorUpdateRequest {
    pub name: String,
    pub description: Option<String>,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug)]
pub enum SensorError {
    NotFound,
    DatabaseError(String),
    ValidationError(String),
}

impl fmt::Display for SensorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SensorError::NotFound => write!(formatter, "Sensor not found"),
            SensorError::DatabaseError(e) => write!(formatter, "Database error: {}", e),
            SensorError::ValidationError(msg) => write!(formatter, "Validation error: {}", msg),
        }
    }
}

impl From<sqlx::Error> for SensorError {
    fn from(err: sqlx::Error) -> Self {
        SensorError::DatabaseError(err.to_string())
    }
}
