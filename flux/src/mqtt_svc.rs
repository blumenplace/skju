use std::result::Result as StdResult;
use bytemuck::PodCastError;
use rama::Context;
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
}

impl MqttService {
    pub(crate) fn new(events_topic: String) -> Self {
        Self { events_topic }
    }
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
                let n = stream.read(&mut read_buf).await.map_err(SvcError::IoError)?;
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

                    handle_mqtt_events(events)?;

                    let consumed = cursor.position() as usize;
                    if consumed == 0 {
                        break;
                    }
                    inbound.drain(..consumed);
                }
            }

            Ok(())
        }
    }
}

fn handle_mqtt_events(events: Vec<GenericEvent<u16>>) -> Result<(), SvcError> {
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
                                let event: &pods::Event = bytemuck::try_from_bytes(payload).map_err(|e| SvcError::MalformedEventStructure(e.to_string()))?;
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
