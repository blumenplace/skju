use serde::{Serialize, Deserialize};
use crate::{ReadingValue, SensorId};
use bytemuck::{Pod, Zeroable};


pub type Timestamp = u64;

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default)]
pub struct Event {
    pub sensor_id: SensorId,
    pub ts: Timestamp,
    pub gyro_x: ReadingValue,
    pub gyro_y: ReadingValue,
    pub gyro_z: ReadingValue,
    pub accel_x: ReadingValue,
    pub accel_y: ReadingValue,
    pub accel_z: ReadingValue,
}

/// SAFETY: All fields are Zeroable.
unsafe impl Zeroable for Event {}

/// SAFETY: All fields in the struct implement Pod
///     the struct implement repr(C), it is packed and fields are defined from biggest to smallest.
///     and there are no generics.
unsafe impl Pod for Event {}

// impl AnyBitPattern for Event {}

const _: () = assert!(
    size_of::<Event>() == size_of::<u64>() + size_of::<u64>() + size_of::<i32>() * 6,
    "Event structure has paddings"
);
