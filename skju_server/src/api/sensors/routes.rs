use super::{create_sensor, delete_all_sensors, delete_sensor, get_all_sensors, get_sensor_by_id, update_sensor};
use crate::state::AppState;
use axum::Router;
use axum::routing::get;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(get_all_sensors)
                .post(create_sensor)
                .delete(delete_all_sensors),
        )
        .route(
            "/{id}",
            get(get_sensor_by_id)
                .put(update_sensor)
                .delete(delete_sensor),
        )
}
