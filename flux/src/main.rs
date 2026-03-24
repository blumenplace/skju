use anyhow::{Result};
use rama::{
    tcp::server::TcpListener,
    graceful::Shutdown,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncWriteExt};
use krafka::producer::Producer;
use rama::graceful::ShutdownGuard;

mod pods;
mod models;
mod events;
mod mqtt_svc;

pub(crate) use models::*;
use crate::mqtt_svc::MqttService;

static FLUX_MQTT_BIND_ADDR: &str = "FLUX_MQTT_BIND_ADDR";

static FLUX_MQTT_BIND_ADDR_DEFAULT: &str = "0.0.0.0:1883";

static FLUX_KAFKA_BROKERS: &str = "FLUX_KAFKA_BROKERS";

static FLUX_KAFKA_BROKERS_DEFAULT: &str = "localhost:9092";

static FLUX_KAFKA_TOPIC: &str = "events";

static FLUX_KAFKA_TOPIC_DEFAULT: &str = "events";

static FLUX_MQTT_TOPIC: &str = "MQTT_TOPIC";

static FLUX_MQTT_TOPIC_DEFAULT: &'static str = "evt";

fn setup_opentelemetry() {
    let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder().build();
    opentelemetry::global::set_meter_provider(provider);
    opentelemetry_instrumentation_tokio::observe_current_runtime();
}

fn setup_tracing() {
    use tracing_subscriber::prelude::*;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn make_kafka_producer() -> Result<Arc<Producer>> {
    let bootsreap_servers = dotenvy::var("FLUX_KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let kafka_topic = dotenvy::var(FLUX_KAFKA_TOPIC).unwrap_or_else(|_| FLUX_KAFKA_TOPIC_DEFAULT.to_string());
    let producer = Producer::builder()
        .bootstrap_servers(bootsreap_servers)
        .client_id(kafka_topic)
        .build()
        .await?;
    Ok(Arc::new(producer))
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    setup_opentelemetry();
    setup_tracing();

    let kafka_producer: Arc<_> = make_kafka_producer().await?;

    let graceful = Shutdown::builder()
        .with_delay(Duration::from_secs(30))
        .build();

    let state = Arc::new(123);
    let bind_addr: SocketAddr = dotenvy::var(FLUX_MQTT_BIND_ADDR).unwrap_or_else(|_| FLUX_MQTT_BIND_ADDR_DEFAULT.to_string()).parse()?;
    let listener = TcpListener::build_with_state(state)
        .bind(bind_addr)
        .await
        .map_err(|err| anyhow::anyhow!(err))?;

    let svc_shutdown: ShutdownGuard = graceful.guard();
    let svc = MqttService::new("events-topic".to_string(), svc_shutdown);
    graceful.spawn_task(listener.serve(svc));
    graceful
        .shutdown_with_limit(Duration::from_secs(30))
        .await?;

    Ok(())
}
