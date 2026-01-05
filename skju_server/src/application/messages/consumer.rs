use super::AppMessage;
use crate::application::readings::ReadingsBuffer;
use crate::ports::bus_service::BusMessage;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tracing::instrument;

pub struct ConsumerContext {
    pub readings_buffer: Arc<ReadingsBuffer>,
}

pub struct AppConsumer {
    pub receiver: Receiver<BusMessage<AppMessage>>,
    pub context: ConsumerContext,
}

impl AppConsumer {
    pub fn new(receiver: Receiver<BusMessage<AppMessage>>, context: ConsumerContext) -> Self {
        Self { receiver, context }
    }

    #[instrument(name = "pipeline.app_consumer.run", skip(self))]
    pub async fn run(mut self) {
        while let Some(message) = self.receiver.recv().await {
            let message_str = message.to_string();

            if let Err(error) = self.dispatch(message).await {
                tracing::error!(
                    "Failed to dispatch message.\n Message:  {}.\n Error: {}",
                    message_str,
                    error
                )
            };
        }
    }

    #[instrument(name = "pipeline.app_consumer.dispatch", skip(self), err)]
    async fn dispatch(&self, message: BusMessage<AppMessage>) -> Result<(), String> {
        match message.message {
            AppMessage::SensorReadingReceived(reading) => {
                self.context
                    .readings_buffer
                    .add(reading)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }
}
