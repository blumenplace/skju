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
use opentelemetry::trace::TracerProvider;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, Protocol, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use tokio::net::TcpListener;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, registry};

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_tracing();

    let config = Config::from_env();
    let pool = get_db_pool(&config).await;

    run_db_migrations(&pool).await;

    let listener = get_tcp_listener(&config).await;
    let app = create_app(pool);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

fn init_tracing() {
    let provider_resource = Resource::builder().with_service_name("skju").build();
    let trace_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_protocol(Protocol::Grpc)
        .with_endpoint("http://localhost:4317")
        .build()
        .expect("Error building GRPC exporter");

    let log_exporter = LogExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpJson)
        .with_endpoint("http://localhost:4318/v1/logs")
        .build()
        .expect("Error building log exporter");

    let trace_provider = SdkTracerProvider::builder()
        .with_batch_exporter(trace_exporter)
        .with_resource(provider_resource.clone())
        .build();

    let log_provider = SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .with_resource(provider_resource.clone())
        .build();

    let log_layer = OpenTelemetryTracingBridge::new(&log_provider);
    let trace_layer = OpenTelemetryLayer::new(trace_provider.tracer("skju"));
    let debug_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_line_number(true)
        .with_target(false);

    registry()
        .with(EnvFilter::from_default_env())
        .with(debug_layer)
        .with(log_layer)
        .with(trace_layer)
        .init();
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
