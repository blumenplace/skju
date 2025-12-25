mod api;
mod app;
mod application;
mod config;
mod domain;
mod error;
mod infrastructure;
mod ports;
mod routes;
mod state;

use crate::app::create_app;
use crate::config::Config;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config = Config::from_env();
    let pool = get_db_pool(&config).await;

    run_db_migrations(&pool).await;

    let listener = get_tcp_listener(&config).await;
    let app = create_app(pool);

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

async fn get_tcp_listener(config: &Config) -> TcpListener {
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind TcpListener to port");

    println!("Server is running on port {}...", config.server_port);

    listener
}
