use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::task::JoinSet;
use krafka::producer::Producer;

mod pods;
mod models;
mod events;
mod mqtt_svc;
mod shutdown;

pub(crate) use models::*;
use crate::mqtt_svc::{MqttTcpService, MqttPacketService};
use crate::shutdown::Shutdown;

static FLUX_MQTT_BIND_ADDR: &str = "FLUX_MQTT_BIND_ADDR";
static FLUX_MQTT_BIND_ADDR_DEFAULT: &str = "0.0.0.0:1883";

static FLUX_KAFKA_BROKERS: &str = "FLUX_KAFKA_BROKERS";
static FLUX_KAFKA_BROKERS_DEFAULT: &str = "localhost:9092";

static FLUX_KAFKA_TOPIC: &str = "events";
static FLUX_KAFKA_TOPIC_DEFAULT: &str = "events";

static FLUX_MQTT_TOPIC: &str = "MQTT_TOPIC";
static FLUX_MQTT_TOPIC_DEFAULT: &str = "evt";

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
    let bootstrap_servers = dotenvy::var("FLUX_KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let kafka_topic = dotenvy::var(FLUX_KAFKA_TOPIC).unwrap_or_else(|_| FLUX_KAFKA_TOPIC_DEFAULT.to_string());
    let producer = Producer::builder()
        .bootstrap_servers(bootstrap_servers)
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

    let _kafka_producer: Arc<_> = make_kafka_producer().await?;

    let shutdown = Shutdown::new().map_err(|e| anyhow::anyhow!(e))?;
    let token = shutdown.token();

    let bind_addr: SocketAddr = dotenvy::var(FLUX_MQTT_BIND_ADDR)
        .unwrap_or_else(|_| FLUX_MQTT_BIND_ADDR_DEFAULT.to_string())
        .parse()?;
    let mqtt_topic = dotenvy::var(FLUX_MQTT_TOPIC)
        .unwrap_or_else(|_| FLUX_MQTT_TOPIC_DEFAULT.to_string());

    let listener = TcpListener::bind(bind_addr).await?;
    tracing::info!(%bind_addr, "mqtt listener started");

    let mut join_set: JoinSet<()> = JoinSet::new();

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                tracing::info!("shutdown signal received, stopping accept loop");
                break;
            },
            res = listener.accept() => {
                let (stream, addr) = res?;
                tracing::info!(%addr, "accepted mqtt connection");
                let event_svc = MqttPacketService::new(Arc::new(mqtt_topic.clone()));
                let mut svc = MqttTcpService::new(token.clone());
                join_set.spawn(async move {
                    if let Err(e) = svc.serve(stream, event_svc).await {
                        tracing::error!(%addr, error = ?e, "mqtt connection error");
                    }
                });
            }
        }
    }

    let drain = async move { while join_set.join_next().await.is_some() {} };
    tokio::time::timeout(Duration::from_secs(30), drain).await.ok();

    Ok(())
}
