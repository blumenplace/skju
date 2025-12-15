use async_trait::async_trait;
use std::fmt::{Debug, Display};

#[derive(Debug)]
pub struct BusMessage<T> {
    pub message: T,
}

#[derive(Debug)]
pub enum BusError<T: Debug> {
    SendError(BusMessage<T>),
}

impl<T: Debug> Display for BusMessage<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}", self.message)
    }
}

#[async_trait]
pub trait BusService<T: Debug>: Send + Sync + 'static {
    async fn send(&self, message: BusMessage<T>) -> Result<(), BusError<T>>;
}
