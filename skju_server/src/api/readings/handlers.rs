use crate::api::readings::dto::{ReadingCreateRequest, ReadingGetBetweenRequest, ReadingModel};
use crate::domain::reading::{ReadingTimestamp, ReadingsRange};
use crate::domain::sensor::SensorID;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

pub async fn create_reading(
    State(state): State<AppState>,
    Json(reading): Json<ReadingCreateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .app_services
        .reading_service
        .create(reading.into())
        .await?;

    let response = (StatusCode::CREATED, ());

    Ok(response)
}

pub async fn get_readings_between(
    State(state): State<AppState>,
    Json(request): Json<ReadingGetBetweenRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let range_request = ReadingsRange::new(
        request.sensor_id.map(SensorID::new),
        ReadingTimestamp::new(request.from),
        ReadingTimestamp::new(request.to),
    )?;

    let readings = state
        .app_services
        .reading_service
        .get_between(range_request)
        .await?;

    let readings = readings
        .into_iter()
        .map(Into::into)
        .collect::<Vec<ReadingModel>>();
    let response = (StatusCode::OK, Json(readings));

    Ok(response)
}
