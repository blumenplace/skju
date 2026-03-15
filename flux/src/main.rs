use anyhow::{anyhow, Result};
use mqtt_protocol_core::mqtt::{
    Connection, Version,
    connection::{Event, role::Server},
    common::Cursor as MqttCursor,
};
use rama::{
    Context,
    service::service_fn,
    tcp::server::TcpListener,
    graceful::Shutdown,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use krafka::producer::Producer;
use mqtt_protocol_core::mqtt::packet::GenericPacket;

mod pods;
mod models;
mod events;

pub(crate) use models::*;

static FLUX_MQTT_BIND_ADDR: &str = "FLUX_MQTT_BIND_ADDR";

static FLUX_MQTT_BIND_ADDR_DEFAULT: &str = "0.0.0.0:1883";

static FLUX_KAFKA_BROKERS: &str = "FLUX_KAFKA_BROKERS";

static FLUX_KAFKA_BROKERS_DEFAULT: &str = "localhost:9092";

static FLUX_KAFKA_TOPIC: &str = "events";

static FLUX_KAFKA_TOPIC_DEFAULT: &str = "events";

const EVENTS_TOPIC: &'static str = "evt";

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

    let svc_shutdown = graceful.guard();
    let svc = service_fn(|ctx, mut steam| async move {
        // drop(svc_shutdown);
        if let Err(err) = handle_mqtt_connection(&ctx, &mut steam).await {
            tracing::error!(error = %err, "mqtt connection failed");
        }
        Ok::<_, std::convert::Infallible>(())
    });

    graceful.spawn_task(listener.serve(svc));
    graceful
        .shutdown_with_limit(Duration::from_secs(30))
        .await?;

    Ok(())
}

async fn handle_mqtt_connection<S, T>(
    _ctx: &Context<S>,
    stream: &mut T,
) -> Result<()>
where
    T: AsyncReadExt + AsyncWriteExt + Unpin,
{
    let mut server = Connection::<Server>::new(Version::V5_0);

    let mut read_buf = [0_u8; 8 * 1024];
    let mut inbound = Vec::with_capacity(16 * 1024);

    loop {
        let n = stream.read(&mut read_buf).await?;
        if n == 0 {
            tracing::info!("mqtt client disconnected");
            let left_events = server.notify_closed();
            todo!("left_events...");
            return Ok(());
        }

        inbound.extend_from_slice(&read_buf[..n]);

        loop {
            let mut cursor = MqttCursor::new(&inbound[..]);
            let events = server.recv(&mut cursor);
            if events.is_empty() {
                break;
            }

            for event in events {
                match event {
                    Event::NotifyPacketReceived(packet) => {
                        tracing::info!(?packet, "received mqtt packet");
                        match packet {
                            GenericPacket::<u16>::V5_0Publish(publish) => {
                                let topic = publish.topic_name();
                                match topic {
                                    EVENTS_TOPIC => {
                                        let payload = publish.payload().as_slice();
                                        let event: &pods::Event = bytemuck::try_from_bytes(payload).map_err(|e| anyhow!(e))?;
                                        todo!("write data to Kafka");
                                    },
                                    _ => {
                                        tracing::warn!(topic = ?topic, "received non-events topic");
                                    }
                                }
                            }
                            other => {
                                tracing::info!(?other, "received non-publish packet");
                            }
                        }

                    },
                    Event::RequestSendPacket {
                        packet,
                        release_packet_id_if_send_error,
                    } => {
                        todo!("send packet");
                    },
                    Event::RequestTimerReset{ .. } => {},
                    Event::RequestTimerCancel(_) => {},
                    Event::NotifyError(mqtt_error) => {
                        todo!()
                    },
                    Event::RequestClose => {
                        tracing::info!("mqtt connection requested close");
                        return Ok(());
                    },
                    Event::NotifyPacketIdReleased(_) => todo!(),
                }
            }

            let consumed = cursor.position() as usize;
            if consumed == 0 {
                break;
            }
            inbound.drain(..consumed);
        }
    }

    Ok(())
}
