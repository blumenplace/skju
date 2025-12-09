use super::handlers::health_check;
use crate::state::AppState;
use axum::Router;
use axum::routing::get;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(health_check))
}
