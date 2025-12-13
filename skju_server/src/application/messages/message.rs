use crate::domain::reading::ReadingCreateRequest;

#[derive(Debug)]
pub enum AppMessage {
    SensorReadingReceived(ReadingCreateRequest),
}
