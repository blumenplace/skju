use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string()),
        };
        let body = (status, Json(json!({ "error": message })));

        body.into_response()
    }
}

pub trait IntoInternal<T> {
    fn into_internal(self) -> Result<T, ApiError>;
}

impl<T, E: std::fmt::Debug> IntoInternal<T> for Result<T, E> {
    fn into_internal(self) -> Result<T, ApiError> {
        self.map_err(|err| {
            eprintln!("Internal error: {:?}", err);
            ApiError::Internal
        })
    }
}
