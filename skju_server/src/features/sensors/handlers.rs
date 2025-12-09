use super::service;
use crate::{error::ApiError, state::AppState};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use skju_core::SensorConfig;

pub async fn set_sensors(
    State(state): State<AppState>,
    Json(sensors): Json<Vec<SensorConfig>>,
) -> Result<impl IntoResponse, ApiError> {
    service::save_sensors(&state, sensors).await?;

    let body = json!({ "success": true, "message": "Sensors were successfully saved" });
    let response = (StatusCode::CREATED, Json(body));

    Ok(response)
}
