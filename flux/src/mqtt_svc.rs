use std::result::Result as StdResult;
use std::task::{Context as TaskContext, Poll};
use tokio_util::sync::CancellationToken;
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

pub(crate) struct TcpMqttService {
    events_topic: String,
    shutdown: CancellationToken,
}

impl TcpMqttService {
    pub(crate) fn new(events_topic: String, shutdown: CancellationToken) -> Self {
        Self { events_topic, shutdown }
    }

    pub(crate) async fn serve<Stream>(&self, mut stream: Stream) -> StdResult<(), SvcError>
    where
        Stream: AsyncRead + AsyncWrite + Unpin + Send,
    {
        let mut server = Connection::<Server>::new(Version::V5_0);
        let mut read_buf = [0u8; 8 * 1024];
        let mut inbound: Vec<u8> = Vec::with_capacity(16 * 1024);

        loop {
            let n = tokio::select! {
                _ = self.shutdown.cancelled() => {
                    tracing::info!("mqtt service shutdown triggered");
                    return Ok(());
                },
                res = stream.read(&mut read_buf) => {
                    let n = res.map_err(SvcError::IoError)?;
                    n
                },
            };

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

                self.handle_mqtt_events(events)?;

                let consumed = cursor.position() as usize;
                if consumed == 0 {
                    break;
                }
                inbound.drain(..consumed);
            }
        }
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
                                let _event: pods::Event = bytemuck::try_pod_read_unaligned(payload)
                                    .map_err(|e| SvcError::MalformedEventStructure(e.to_string()))?;
                                dbg!(&_event);
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
                    packet: _,
                    release_packet_id_if_send_error: _,
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

impl tower::Service<tokio::net::TcpStream> for TcpMqttService {
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
            TcpMqttService::new(events_topic, shutdown).serve(stream).await
        })
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
            let svc = TcpMqttService::new(et, child_token);
            ss.notify_one();
            svc.serve(istream).await
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
        let token = CancellationToken::new();
        let (_ostream, istream) = duplex(1024);

        let svc_started = Arc::new(Notify::new());
        let ss = Arc::clone(&svc_started);

        let child_token = token.clone();
        let handle = tokio::spawn(async move {
            let svc = TcpMqttService::new("test-topic".to_string(), child_token);
            ss.notify_one();
            svc.serve(istream).await
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
