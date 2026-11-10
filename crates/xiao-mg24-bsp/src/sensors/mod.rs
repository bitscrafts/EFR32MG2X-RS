//! Sensor Drivers for XIAO MG24 Sense
//!
//! This module provides drivers for the onboard sensors on the Seeed Studio
//! XIAO MG24 Sense board.
//!
//! # Onboard Sensors
//!
//! - **MSM381ACT001**: Analog MEMS microphone
//! - **LSM6DS3**: 6-axis IMU (accelerometer + gyroscope) - Future

pub mod microphone;

pub use microphone::Microphone;
