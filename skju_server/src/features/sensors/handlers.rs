use crate::domain::sensor::{SensorCreateRequest, SensorUpdateRequest};
use crate::{error::ApiError, state::AppState};
use axum::extract::Path;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

pub async fn create_sensor(
    State(state): State<AppState>,
    Json(sensor): Json<SensorCreateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let sensor = state.app_services.sensor_service.create(sensor).await?;

    let response = (StatusCode::CREATED, Json(sensor));

    Ok(response)
}

pub async fn update_sensor(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(sensor): Json<SensorUpdateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let sensor = state.app_services.sensor_service.update(id, sensor).await?;

    let response = (StatusCode::OK, Json(sensor));

    Ok(response)
}

pub async fn delete_sensor(State(state): State<AppState>, Path(id): Path<i32>) -> Result<impl IntoResponse, ApiError> {
    state.app_services.sensor_service.delete(id).await?;

    let response = (StatusCode::NO_CONTENT, ());

    Ok(response)
}

pub async fn get_all_sensors(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let sensors = state.app_services.sensor_service.list().await?;

    let response = (StatusCode::OK, Json(sensors));

    Ok(response)
}

pub async fn get_sensor_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let sensor = state.app_services.sensor_service.get_by_id(id).await?;

    let response = (StatusCode::OK, Json(sensor));

    Ok(response)
}

pub async fn delete_all_sensors(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    state.app_services.sensor_service.delete_all().await?;

    let response = (StatusCode::NO_CONTENT, ());

    Ok(response)
}
