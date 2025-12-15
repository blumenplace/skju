use crate::domain::reading::ReadingError;
use crate::domain::sensor::SensorError;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    Internal,
    NotFound,
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string()),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not Found".to_string()),
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
        };
        let body = (status, Json(json!({ "error": message })));

        body.into_response()
    }
}

impl From<SensorError> for ApiError {
    fn from(error: SensorError) -> Self {
        match error {
            SensorError::NotFound => ApiError::NotFound,
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
        }
    }
}
