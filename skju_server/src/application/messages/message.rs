use crate::domain::reading::ReadingCreate;

#[derive(Debug)]
pub enum AppMessage {
    SensorReadingReceived(ReadingCreate),
}
