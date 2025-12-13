use async_trait::async_trait;
use std::fmt::Debug;
use tokio::sync::mpsc::error::SendError;

pub struct BusMessage<T: Debug> {
    pub message: T,
}

#[async_trait]
pub trait BusService<T: Debug>: Send + Sync + 'static {
    async fn send(&self, message: BusMessage<T>) -> Result<(), SendError<BusMessage<T>>>;
}
