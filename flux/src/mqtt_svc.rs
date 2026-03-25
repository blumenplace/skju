use std::result::Result as StdResult;
use rama::Context;
use rama::graceful::ShutdownGuard;
use mqtt_protocol_core::mqtt::{
    Connection, Version,
    connection::{Event, role::Server, GenericEvent},
    common::Cursor as MqttCursor,
    packet::GenericPacket,
};
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt};
use crate::pods;

#[derive(Debug, thiserror::Error)]
pub(crate) enum SvcError {
    #[error(transparent)]
    IoError(std::io::Error),
    #[error(transparent)]
    MqttError(mqtt_protocol_core::mqtt::result_code::MqttError),
    #[error("malformed event structure: {0}")]
    MalformedEventStructure(String),
}

pub(crate) struct MqttService {
    events_topic: String,
    shutdown: ShutdownGuard,
}

impl<S, Stream> rama::Service<S, Stream> for MqttService
where
    Stream: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    type Response = ();
    type Error = SvcError;

    fn serve<'a>(&'a self, ctx: Context<S>, mut stream: Stream) -> impl Future<Output=StdResult<Self::Response, Self::Error>> + Send + 'a
    {
        let mut server = Connection::<Server>::new(Version::V5_0);
        let mut read_buf = [0u8; 8 * 1024];
        let mut inbound: Vec<u8> = Vec::with_capacity(16 * 1024);

        async move {
            loop {
                let n = tokio::select! {
                    _ = self.shutdown.shutdown_signal_triggered() => {
                        tracing::info!("mqtt service shutdown triggered");
                        // let left_events = server.notify_closed();
                        return Ok(());
                    },
                    res = stream.read(&mut read_buf) => {
                        let n = res.map_err(SvcError::IoError)?;
                        n
                    },
                };

                if n == 0 {
                    tracing::info!("mqtt client disconnected");
                    // let left_events = server.notify_closed();
                    return Ok(());
                }

                inbound.extend_from_slice(&read_buf[..n]);

                loop {
                    let mut cursor = MqttCursor::new(&inbound[..]);
                    let events = server.recv(&mut cursor);
                    if events.is_empty() {
                        break;
                    }

                    self.handle_mqtt_events(events)?;

                    let consumed = cursor.position() as usize;
                    if consumed == 0 {
                        break;
                    }
                    inbound.drain(..consumed);
                }
            }
        }
    }
}

impl MqttService {
    pub(crate) fn new(events_topic: String, shutdown: ShutdownGuard) -> Self {
        Self { events_topic, shutdown }
    }

    fn handle_mqtt_events(&self, events: Vec<GenericEvent<u16>>) -> Result<(), SvcError> {
        for event in events {
            match event {
                Event::NotifyPacketReceived(packet) => {
                    tracing::info!(?packet, "received mqtt packet");
                    match packet {
                        GenericPacket::<u16>::V5_0Publish(publish) => {
                            let topic = publish.topic_name();
                            if topic == self.events_topic {
                                let payload = publish.payload().as_slice();
                                let _event: &pods::Event = bytemuck::try_from_bytes(payload).map_err(|e| SvcError::MalformedEventStructure(e.to_string()))?;
                                // todo!("write data to Kafka");
                            } else {
                                tracing::warn!(topic = ?topic, "received non-events topic");
                            }
                        },
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
                Event::RequestTimerReset { .. } => {},
                Event::RequestTimerCancel(_) => {},
                Event::NotifyError(mqtt_error) => {
                    tracing::error!(?mqtt_error, "mqtt error");
                    return Err(SvcError::MqttError(mqtt_error))
                },
                Event::RequestClose => {
                    tracing::info!("mqtt connection requested close");
                    return Ok(());
                },
                Event::NotifyPacketIdReleased(_) => todo!(),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rama::graceful::Shutdown;
    use tokio::io::{duplex, AsyncWriteExt};
    use std::time::Duration;
    use tokio::sync::Notify;
    use std::sync::Arc;
    use rama::Service;

    #[tokio::test] // (flavor = "multi_thread", worker_threads = 4)
    async fn test_mqtt_service_publish() {
        let shutdown = Shutdown::builder().build();
        let (mut ostream, istream) = duplex(1024);

        let svc_started = Arc::new(Notify::new());
        let ss = Arc::clone(&svc_started);

        let events_topic = "test-topic";
        let et = events_topic.to_string();
        let handle = shutdown.spawn_task_fn(move |guard| {
            let svc = MqttService::new(et, guard);
            let ss = ss.clone();
            async move {
                ss.notify_one();
                svc.serve(Context::default(), istream).await
            }
        });

        svc_started.notified().await;

        let event = pods::Event::default();
        let buf = make_mqtt_packet_data(events_topic, bytemuck::bytes_of(&event));

        ostream.write_all(&buf).await.expect("failed to write to stream");

        // Give some time for the service to process the message
        tokio::time::sleep(Duration::from_millis(100)).await;

        let _ = shutdown.shutdown_with_limit(Duration::from_secs(1)).await;
        let result = handle.await.expect("task panicked");
        assert!(result.is_ok());
    }

    fn make_mqtt_packet_data(topic: &str, payload: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();
        // Fixed header: type=3 (PUBLISH), flags=0
        buf.push(0x30);

        let mut inner_buf = Vec::new();
        // Topic name
        inner_buf.extend_from_slice(&(topic.len() as u16).to_be_bytes());
        inner_buf.extend_from_slice(topic.as_bytes());
        // No Packet Identifier (QoS 0)
        // Properties (length 0 for simple test)
        inner_buf.push(0);
        // Payload
        inner_buf.extend_from_slice(&payload);

        // Remaining length
        let rem_len = inner_buf.len();
        let mut val = rem_len;
        loop {
            let mut byte = (val % 128) as u8;
            val /= 128;
            if val > 0 {
                byte |= 128;
            }
            buf.push(byte);
            if val == 0 {
                break;
            }
        }
        buf.extend_from_slice(&inner_buf);
        buf
    }

    #[tokio::test]
    async fn test_mqtt_service_shutdown() {
        let shutdown = Shutdown::builder().build();
        let (_ostream, istream) = duplex(1024);

        let svc_started = Arc::new(Notify::new());
        let ss = Arc::clone(&svc_started);

        let handle = shutdown.spawn_task_fn(|guard| async move {
            let svc = MqttService::new("test-topic".to_string(), guard);
            ss.notify_one();
            svc.serve(Context::default(), istream).await
        });

        svc_started.notified().await;

        let shutdown_res = shutdown.shutdown_with_limit(Duration::from_secs(1)).await;
        assert!(shutdown_res.is_ok(), "must successfully shutdown");

        let result = handle.await.expect("task panicked");
        assert!(result.is_ok());
    }
}
