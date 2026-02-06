use crate::domain::reading::ReadingError;
use crate::domain::sensor::{SensorError, SensorID};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    Internal,
    SensorNotFound(SensorID),
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string()),
            ApiError::SensorNotFound(id) => (StatusCode::NOT_FOUND, format!("sensor {id} is not found")),
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
        };
        let body = (status, Json(json!({ "error": message })));

        body.into_response()
    }
}

impl From<SensorError> for ApiError {
    fn from(error: SensorError) -> Self {
        match error {
            SensorError::NotFound(id) => ApiError::SensorNotFound(id),
            SensorError::Validation(msg) => ApiError::BadRequest(msg),
            SensorError::Database(_) => ApiError::Internal,
            SensorError::Internal(_) => ApiError::Internal,
        }
    }
}

impl From<ReadingError> for ApiError {
    fn from(error: ReadingError) -> Self {
        match error {
            ReadingError::Database(_) => ApiError::Internal,
            ReadingError::Internal(_) => ApiError::Internal,
            ReadingError::InvalidRange(_) => ApiError::BadRequest("Invalid range".to_string()),
        }
    }
}
