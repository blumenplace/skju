use crate::domain::sensor::{Sensor, SensorCoordinates, SensorCreate, SensorDescription, SensorName, SensorUpdate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct SensorModel {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub x: f64,
    pub y: f64,
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

impl From<SensorCreateRequest> for SensorCreate {
    fn from(request: SensorCreateRequest) -> Self {
        SensorCreate {
            name: SensorName::new(request.name),
            description: SensorDescription::new(request.description),
            coordinates: SensorCoordinates::new(request.x, request.y),
        }
    }
}

impl From<SensorUpdateRequest> for SensorUpdate {
    fn from(request: SensorUpdateRequest) -> SensorUpdate {
        SensorUpdate {
            name: SensorName::new(request.name),
            description: SensorDescription::new(request.description),
            coordinates: SensorCoordinates::new(request.x, request.y),
        }
    }
}

impl From<Sensor> for SensorModel {
    fn from(sensor: Sensor) -> Self {
        SensorModel {
            id: sensor.id.value(),
            name: sensor.name.value().to_string(),
            description: sensor.description.value(),
            x: sensor.coordinates.x(),
            y: sensor.coordinates.y(),
        }
    }
}
