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
    Internal(String),
    Database(String),
    Validation(String),
}

impl fmt::Display for SensorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SensorError::NotFound => write!(formatter, "Sensor not found"),
            SensorError::Internal(e) => write!(formatter, "Internal error: {}", e),
            SensorError::Database(e) => write!(formatter, "Database error: {}", e),
            SensorError::Validation(e) => write!(formatter, "Validation error: {}", e),
        }
    }
}
