use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

pub async fn health_check() -> impl IntoResponse {
    let body = json!({ "status": "ok", "message": "Server is up and running" });
    (StatusCode::OK, Json(body))
}
