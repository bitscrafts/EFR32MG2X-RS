//! Sensor drivers.
//!
//! `bme68x` is the temperature/pressure/humidity sensor on the XIAO Expansion
//! Base (0x76 or 0x77 on the same I2C bus as the panel). Like the display
//! driver it is generic over the I2C transport.

pub mod bme68x;

pub use bme68x::{Bme68x, Reading, SensorError};
