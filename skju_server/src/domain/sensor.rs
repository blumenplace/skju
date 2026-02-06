use chrono::{DateTime, Utc};
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct SensorID(i32);

impl fmt::Display for SensorID {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(transparent)]
pub struct SensorName(String);

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(transparent)]
pub struct SensorDescription(Option<String>);

#[derive(Debug, Clone, Copy)]
pub struct SensorCoordinates {
    x: f64,
    y: f64,
}

#[derive(Debug)]
pub struct Sensor {
    pub id: SensorID,
    pub name: SensorName,
    pub description: SensorDescription,
    pub coordinates: SensorCoordinates,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct DBSensor {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub x: f64,
    pub y: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct SensorCreate {
    pub name: SensorName,
    pub description: SensorDescription,
    pub coordinates: SensorCoordinates,
}

#[derive(Debug)]
pub struct SensorUpdate {
    pub name: SensorName,
    pub description: SensorDescription,
    pub coordinates: SensorCoordinates,
}

#[derive(Debug)]
pub enum SensorError {
    NotFound(SensorID),
    Internal(String),
    Database(String),
    Validation(String),
}

impl fmt::Display for SensorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SensorError::NotFound(id) => write!(formatter, "Sensor {id} not found"),
            SensorError::Internal(e) => write!(formatter, "Internal error: {e}"),
            SensorError::Database(e) => write!(formatter, "Database error: {e}"),
            SensorError::Validation(e) => write!(formatter, "Validation error: {e}"),
        }
    }
}

impl From<DBSensor> for Sensor {
    fn from(db_sensor: DBSensor) -> Self {
        Sensor {
            id: SensorID::new(db_sensor.id),
            name: SensorName::new(db_sensor.name),
            description: SensorDescription::new(db_sensor.description),
            coordinates: SensorCoordinates::new(db_sensor.x, db_sensor.y),
            created_at: db_sensor.created_at,
        }
    }
}

impl From<Sensor> for DBSensor {
    fn from(sensor: Sensor) -> Self {
        DBSensor {
            id: sensor.id.value(),
            name: sensor.name.value().to_string(),
            description: sensor.description.value(),
            x: sensor.coordinates.x(),
            y: sensor.coordinates.y(),
            created_at: sensor.created_at,
        }
    }
}

impl SensorID {
    pub fn new(id: i32) -> Self {
        SensorID(id)
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

impl SensorName {
    pub fn new(name: String) -> Self {
        SensorName(name)
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl SensorDescription {
    pub fn new(description: Option<String>) -> Self {
        SensorDescription(description)
    }

    pub fn value(&self) -> Option<String> {
        self.0.clone()
    }
}

impl SensorCoordinates {
    pub fn new(x: f64, y: f64) -> Self {
        SensorCoordinates { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}
