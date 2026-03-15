pub(crate) struct Location {
    long: f64,
    lat: f64
}

pub(crate) type ReadingValue = i32;

pub(crate) type SensorId = u32;

pub(crate) struct Reading {
    pub x: ReadingValue,
    pub y: ReadingValue,
    pub z: ReadingValue,
}

pub(crate) struct Event {
    pub id: SensorId,
    pub gyro: Reading,
    pub axel: Reading,
}