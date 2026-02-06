use crate::application::messages::AppMessage;
use crate::ports::bus_service::{BusError, BusMessage, BusService};
use async_trait::async_trait;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::error::SendError;
use tracing::instrument;

pub struct AppBus {
    sender: Sender<BusMessage<AppMessage>>,
}

impl AppBus {
    pub fn new(sender: Sender<BusMessage<AppMessage>>) -> Self {
        Self { sender }
    }
}

impl From<SendError<BusMessage<AppMessage>>> for BusError<AppMessage> {
    fn from(error: SendError<BusMessage<AppMessage>>) -> Self {
        BusError::SendError(error.0)
    }
}

#[async_trait]
impl BusService<AppMessage> for AppBus {
    #[instrument(name = "pipeline.app_bus.send", skip(self), err)]
    async fn send(&self, message: BusMessage<AppMessage>) -> Result<(), BusError<AppMessage>> {
        self.sender.send(message).await.map_err(Into::into)
    }
}
