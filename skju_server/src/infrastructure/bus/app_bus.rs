use async_trait::async_trait;
use tokio::sync::mpsc::error::SendError;
use crate::application::messages::AppMessage;
use crate::ports::bus_service::{BusMessage, BusService};
use tokio::sync::mpsc::Sender;

pub struct AppBus {
    sender: Sender<BusMessage<AppMessage>>,
}

impl AppBus {
    pub fn new(sender: Sender<BusMessage<AppMessage>>) -> Self {
        Self { sender }
    }
}


#[async_trait]
impl BusService<AppMessage> for AppBus {
    async fn send(&self, message: BusMessage<AppMessage>) -> Result<(), SendError<BusMessage<AppMessage>>> {
        self.sender.send(message).await
    }
}
