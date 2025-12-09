use super::handlers::{get_sensors, set_sensors};
use crate::state::AppState;
use axum::Router;
use axum::routing::get;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(get_sensors).post(set_sensors))
}
