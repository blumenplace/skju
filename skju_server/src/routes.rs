use crate::features::{readings, sensors};
use crate::state::AppState;
use axum::http::{StatusCode, Uri};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .nest("/sensors", sensors::routes())
        .nest("/readings", readings::routes())
        .fallback(get(fallback))
}

async fn fallback(uri: Uri) -> impl IntoResponse {
    let body = json!({ "message": format!("Route for {uri} not found")});
    (StatusCode::NOT_FOUND, Json(body))
}
