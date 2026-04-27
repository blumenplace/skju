#![no_std]

// mod common;
// pub mod filter;
// pub mod sensor;
// pub mod utils;
// pub use common::*;

mod consts;
pub mod pods;

pub use consts::*;
pub use pods::Event;

pub struct Location {
    pub lon: f64,
    pub lat: f64
}

pub type ReadingValue = i32;

pub type SensorId = u64;

pub struct Reading {
    pub x: ReadingValue,
    pub y: ReadingValue,
    pub z: ReadingValue,
}
