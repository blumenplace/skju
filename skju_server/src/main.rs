mod app;
mod application;
mod config;
mod domain;
mod error;
mod features;
mod infrastructure;
mod ports;
mod routes;
mod state;

use crate::app::create_app;
use crate::application::sensors;
use crate::config::Config;
use crate::infrastructure::pg_sensor_repository::PgSensorRepository;
use crate::state::AppState;

use axum::Router;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config = Config::from_env();
    let pool = get_db_pool(&config).await;

    run_db_migrations(&pool).await;

    let listener = get_tcp_listener(&config).await;
    let app = get_app(pool);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn get_db_pool(config: &Config) -> Pool<Postgres> {
    PgPoolOptions::new()
        .max_connections(config.database_pool_size)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database")
}

async fn run_db_migrations(pool: &Pool<Postgres>) {
    sqlx::migrate!()
        .run(pool)
        .await
        .expect("Failed to run database migrations");
}

fn get_app(db_pool: Pool<Postgres>) -> Router<()> {
    let sensor_repository = PgSensorRepository::new(db_pool.clone());
    let sensor_service = sensors::Service::new(Arc::new(sensor_repository));

    let app_state = AppState::new(Arc::new(sensor_service));
    let app = create_app().with_state(app_state);

    app
}

async fn get_tcp_listener(config: &Config) -> TcpListener {
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind TcpListener to port");

    println!("Server is running on port {}...", config.server_port);

    listener
}
