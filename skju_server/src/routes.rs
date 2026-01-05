use crate::api::{readings, sensors};
use crate::state::AppState;
use axum::http::{StatusCode, Uri};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use tower_http::trace::TraceLayer;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .layer(TraceLayer::new_for_http())
        .nest("/sensors", sensors::routes())
        .nest("/readings", readings::routes())
        .fallback(get(fallback))
}

async fn fallback(uri: Uri) -> impl IntoResponse {
    let body = json!({ "message": format!("Route for {uri} not found")});
    (StatusCode::NOT_FOUND, Json(body))
}
