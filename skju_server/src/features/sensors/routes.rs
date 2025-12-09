use super::handlers::set_sensors;
use crate::state::AppState;
use axum::{Router, routing::post};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", post(set_sensors))
}
