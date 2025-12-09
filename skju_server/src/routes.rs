use crate::features::{health_check, sensors};
use crate::state::AppState;
use axum::http::{StatusCode, Uri};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .nest("/health_check", health_check::routes())
        .nest("/sensors", sensors::routes())
        .fallback(get(fallback))
}

async fn fallback(uri: Uri) -> impl IntoResponse {
    let body = json!({ "message": format!("Route for {uri} not found")});
    let response = (StatusCode::NOT_FOUND, Json(body));

    response
}
