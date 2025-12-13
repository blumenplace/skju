use crate::domain::reading::{ReadingCreateRequest, ReadingGetBetweenRequest};
use crate::error::ApiError;
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

pub async fn create_reading(
    State(state): State<AppState>,
    Json(reading): Json<ReadingCreateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .app_services
        .reading_service
        .create(reading)
        .await
        .map_err(|_| ApiError::Internal)?;

    let response = (StatusCode::CREATED, ());

    Ok(response)
}

pub async fn get_readings_between(
    State(state): State<AppState>,
    Json(request): Json<ReadingGetBetweenRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let readings = state
        .app_services
        .reading_service
        .get_between(request)
        .await
        .map_err(|e| {
            println!("{:?}", e);
            ApiError::Internal
        })?;
    let response = (StatusCode::OK, Json(readings));

    Ok(response)
}
