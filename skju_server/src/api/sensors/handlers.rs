use super::dto::{SensorCreateRequest, SensorModel, SensorUpdateRequest};
use crate::domain::sensor::SensorID;
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
    let new_sensor = sensor.into();
    let result = state.app_services.sensor_service.create(new_sensor).await?;
    let response = (StatusCode::CREATED, Json::<SensorModel>(result.into()));

    Ok(response)
}

pub async fn update_sensor(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(sensor): Json<SensorUpdateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let sensor_id = SensorID::new(id);
    let sensor = state
        .app_services
        .sensor_service
        .update(sensor_id, sensor.into())
        .await?;
    let response = (StatusCode::OK, Json::<SensorModel>(sensor.into()));

    Ok(response)
}

pub async fn delete_sensor(State(state): State<AppState>, Path(id): Path<i32>) -> Result<impl IntoResponse, ApiError> {
    let sensor_id = SensorID::new(id);

    state.app_services.sensor_service.delete(sensor_id).await?;

    let response = (StatusCode::NO_CONTENT, ());

    Ok(response)
}

pub async fn get_all_sensors(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let result = state.app_services.sensor_service.list().await?;
    let sensors = result
        .into_iter()
        .map(|sensor| sensor.into())
        .collect::<Vec<SensorModel>>();
    let response = (StatusCode::OK, Json(sensors));

    Ok(response)
}

pub async fn get_sensor_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let sensor_id = SensorID::new(id);
    let result = state
        .app_services
        .sensor_service
        .get_by_id(sensor_id)
        .await?;
    let sensor: SensorModel = result.into();
    let response = (StatusCode::OK, Json(sensor));

    Ok(response)
}

pub async fn delete_all_sensors(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    state.app_services.sensor_service.delete_all().await?;

    let response = (StatusCode::NO_CONTENT, ());

    Ok(response)
}
