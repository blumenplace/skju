use crate::features::readings::handlers::{create_reading, get_readings_between};
use crate::state::AppState;
use axum::Router;
use axum::routing::post;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_reading))
        .route("/get_between", post(get_readings_between))
}
