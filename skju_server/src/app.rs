use crate::{routes::create_routes, state::AppState};
use axum::Router;

pub fn create_app() -> Router<AppState> {
    Router::new().nest("/api", create_routes())
}
