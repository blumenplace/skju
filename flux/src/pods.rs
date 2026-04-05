use serde::{Serialize, Deserialize};
use crate::{ReadingValue, SensorId};
use bytemuck::{AnyBitPattern, Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default)]
pub(crate) struct Event {
    sensor_id: SensorId,
    gyro_x: ReadingValue,
    gyro_y: ReadingValue,
    gyro_z: ReadingValue,
    accel_x: ReadingValue,
    accel_y: ReadingValue,
    accel_z: ReadingValue,
}

/// SAFETY: All fields are Zeroable.
unsafe impl Zeroable for Event {}

/// SAFETY: All fields in the struct implement Pod
///     the struct implement repr(C), it is packed and fields are defined from biggest to smallest.
///     and there are no generics.
unsafe impl Pod for Event {}

// impl AnyBitPattern for Event {}

const _: () = assert!(
    size_of::<Event>() == size_of::<u64>() + size_of::<i32>() * 6,
    "Event structure has paddings"
);
