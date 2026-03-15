use serde::{Serialize, Deserialize};
use crate::{ReadingValue, SensorId};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct Event {
    sensor_id: SensorId,
    gyro_x: ReadingValue,
    gyro_y: ReadingValue,
    gyro_z: ReadingValue,
    accel_x: ReadingValue,
    accel_y: ReadingValue,
    accel_z: ReadingValue,
}

unsafe impl bytemuck::Zeroable for Event {}

unsafe impl bytemuck::Pod for Event {}

// impl From<crate::Event> for Event {
//     fn from(value: crate::Event) -> Self {
//         Self {
//             sensor_id: value.id,
//             gyro_x: value.gyro.x,
//             gyro_y: value.gyro.y,
//             gyro_z: value.gyro.z,
//             accel_x: value.axel.x,
//             accel_y: value.axel.y,
//             accel_z: value.axel.z,
//         }
//     }
// }

/*
    event_id
    occurred_at
    updated_at
    latitude
    longitude
    depth_km
    magnitude
    mag_type
    place
    region
    source
    status
    tsunami
*/