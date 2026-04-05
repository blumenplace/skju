use std::future::Future;
use std::result::Result as StdResult;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};
use futures::stream::{BoxStream, StreamExt as _};
use tower::ServiceExt as _;
use tokio_util::sync::CancellationToken;
use mqtt_protocol_core::mqtt::{
    Connection, Version,
    connection::{Event, role::Server, GenericEvent},
    common::Cursor as MqttCursor,
    packet::GenericPacket,
};
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
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

pub(crate) type EventStream = BoxStream<'static, Result<GenericPacket<u16>, SvcError>>;

pub(crate) struct MqttTcpService {
    events_topic: String,
    shutdown: CancellationToken,
}

impl MqttTcpService {
    pub(crate) fn new(events_topic: String, shutdown: CancellationToken) -> Self {
        Self { events_topic, shutdown }
    }

    pub(crate) async fn serve<Stream, S>(
        &self,
        stream: Stream,
        mut event_svc: S,
    ) -> StdResult<(), SvcError>
    where
        Stream: AsyncRead + AsyncWrite + Unpin + Send,
        S: tower::Service<Vec<GenericEvent<u16>>, Response = EventStream, Error = SvcError> + Send,
        S::Future: Send,
    {
        let mut server = Connection::<Server>::new(Version::V5_0);
        let mut read_buf = [0u8; 8 * 1024];
        let mut inbound: Vec<u8> = Vec::with_capacity(16 * 1024);
        let (mut read_half, mut write_half) = tokio::io::split(stream);

        loop {
            tokio::select! {
                _ = self.shutdown.cancelled() => {
                    tracing::info!("mqtt service shutdown triggered");
                    return Ok(());
                },
                res = read_half.read(&mut read_buf) => {
                    let n = res.map_err(SvcError::IoError)?;
                    if n == 0 {
                        tracing::info!("mqtt client disconnected");
                        return Ok(());
                    }
                    inbound.extend_from_slice(&read_buf[..n]);
                    loop {
                        let mut cursor = MqttCursor::new(&inbound[..]);
                        let events = server.recv(&mut cursor);
                        if events.is_empty() {
                            break;
                        }
                        let consumed = cursor.position() as usize;

                        let mut app_events = Vec::new();
                        for event in events {
                            match event {
                                Event::NotifyPacketReceived(_) => app_events.push(event),
                                Event::RequestSendPacket {
                                    packet,
                                    release_packet_id_if_send_error: _,
                                } => {
                                    // let bytes = packet.into_bytes();
                                    // write_half.write_all(&bytes).await.map_err(SvcError::IoError)?;
                                    todo!("serialize and write packet");
                                },
                                Event::RequestTimerReset { .. } | Event::RequestTimerCancel(_) => {},
                                Event::NotifyPacketIdReleased(_) => {},
                                Event::NotifyError(e) => {
                                    tracing::error!(?e, "mqtt error");
                                    return Err(SvcError::MqttError(e));
                                },
                                Event::RequestClose => {
                                    tracing::info!("mqtt connection close requested");
                                    return Ok(());
                                },
                            }
                        }

                        if !app_events.is_empty() {
                            let mut cmd_stream = event_svc.ready().await?.call(app_events).await?;
                            while let Some(cmd) = cmd_stream.next().await {
                                let packet = cmd?;
                                let _new_events = server.send(packet);
                                dbg!(&_new_events);
                            }
                        }

                        if consumed == 0 {
                            break;
                        }
                        inbound.drain(..consumed);
                    }
                },
            }
        }
    }
}

impl tower::Service<tokio::net::TcpStream> for MqttTcpService {
    type Response = ();
    type Error = SvcError;
    type Future = std::pin::Pin<Box<dyn Future<Output = StdResult<(), SvcError>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut TaskContext<'_>) -> Poll<StdResult<(), SvcError>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, stream: tokio::net::TcpStream) -> Self::Future {
        let events_topic = self.events_topic.clone();
        let shutdown = self.shutdown.clone();
        Box::pin(async move {
            let event_svc = MqttEventService::new(Arc::new(events_topic.clone()));
            let svc = MqttTcpService::new(events_topic, shutdown);
            svc.serve(stream, event_svc).await
        })
    }
}

pub(crate) struct MqttEventService {
    events_topic: Arc<String>,
}

impl MqttEventService {
    pub(crate) fn new(events_topic: Arc<String>) -> Self {
        Self { events_topic }
    }
}

impl tower::Service<Vec<GenericEvent<u16>>> for MqttEventService {
    type Response = EventStream;
    type Error = SvcError;
    type Future = std::future::Ready<StdResult<EventStream, SvcError>>;

    fn poll_ready(&mut self, _cx: &mut TaskContext<'_>) -> Poll<StdResult<(), SvcError>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, events: Vec<GenericEvent<u16>>) -> Self::Future {
        let events_topic = self.events_topic.clone();
        let result = (|| -> StdResult<EventStream, SvcError> {
            let cmds: Vec<_> = Vec::new();
            for event in events {
                let Event::NotifyPacketReceived(packet) = event else {
                    continue;
                };
                tracing::info!(?packet, "received mqtt packet");
                match packet {
                    GenericPacket::<u16>::V5_0Publish(publish) => {
                        let topic = publish.topic_name();
                        if topic == &*events_topic {
                            let payload = publish.payload().as_slice();
                            let pod_event: pods::Event = bytemuck::try_pod_read_unaligned(payload)
                                .map_err(|e| SvcError::MalformedEventStructure(e.to_string()))?;
                            dbg!(&pod_event);
                            // Push write-back bytes here when needed:
                            // cmds.push(some_response_bytes);
                        } else {
                            tracing::warn!(topic = ?topic, "received non-events topic");
                        }
                    },
                    other => {
                        tracing::info!(?other, "received non-publish packet");
                    }
                }
            }
            Ok(futures::stream::iter(cmds.into_iter().map(Ok)).boxed())
        })();
        std::future::ready(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{duplex, AsyncWriteExt};
    use std::time::Duration;
    use tokio::sync::Notify;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_mqtt_service_publish() {
        let token = CancellationToken::new();
        let (mut ostream, istream) = duplex(1024);

        let svc_started = Arc::new(Notify::new());
        let ss = Arc::clone(&svc_started);

        let events_topic = "test-topic";
        let et = events_topic.to_string();
        let child_token = token.clone();
        let handle = tokio::spawn(async move {
            let event_svc = MqttEventService::new(et.clone().into());
            let svc = MqttTcpService::new(et, child_token);
            ss.notify_one();
            svc.serve(istream, event_svc).await
        });

        svc_started.notified().await;

        let event = pods::Event::default();
        let buf = make_mqtt_packet_data(events_topic, bytemuck::bytes_of(&event));

        ostream.write_all(&buf).await.expect("failed to write to stream");

        // Give some time for the service to process the message
        tokio::time::sleep(Duration::from_millis(100)).await;

        token.cancel();
        let result = tokio::time::timeout(Duration::from_secs(1), handle)
            .await
            .expect("timed out")
            .expect("task panicked");
        println!("error: {:?}", result);
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
        inner_buf.extend_from_slice(payload);

        // Remaining length (variable-length encoding)
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
        let token = CancellationToken::new();
        let (_ostream, istream) = duplex(1024);

        let svc_started = Arc::new(Notify::new());
        let ss = Arc::clone(&svc_started);

        let child_token = token.clone();
        let handle = tokio::spawn(async move {
            let event_svc = MqttEventService::new("test-topic".to_string().into());
            let svc = MqttTcpService::new("test-topic".to_string(), child_token);
            ss.notify_one();
            svc.serve(istream, event_svc).await
        });

        svc_started.notified().await;

        token.cancel();
        let result = tokio::time::timeout(Duration::from_secs(1), handle)
            .await
            .expect("timed out")
            .expect("task panicked");
        assert!(result.is_ok());
    }
}
