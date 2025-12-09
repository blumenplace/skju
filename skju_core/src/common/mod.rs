use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coord {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SensorConfig {
    pub id: u64,
    pub name: String,
    pub coord: Coord,
}

#[derive(Clone, Debug)]
pub struct SensorData {
    pub value: f64,
    pub timestamp: u128,
}

#[derive(Clone, Debug)]
pub struct SensorOutput {
    pub sensor_id: u64,
    pub sensor_name: String,
    pub sensor_coord: Coord,
    pub value: f64,
    pub timestamp: u128,
}

pub struct FilterContext<'a> {
    pub readings: &'a VecDeque<SensorData>,
    pub raw_value: f64,
    pub timestamp: u128,
    pub capacity: usize,
}

pub trait LowPassFilter {
    fn apply(&mut self, context: &FilterContext) -> f64;
}
