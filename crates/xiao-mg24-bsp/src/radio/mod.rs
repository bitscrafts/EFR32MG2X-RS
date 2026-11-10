//! Radio protocol implementations for XIAO MG24 Sense
//!
//! This module provides safe Rust abstractions over the EFR32MG24's radio protocols:
//! - Bluetooth Low Energy 5.3 (BLE)
//! - Zigbee 3.0 (future)
//! - Thread/Matter (future)
//!
//! # Architecture
//!
//! Radio protocol implementations sit above the HAL layer and provide board-specific
//! abstractions. This follows industry patterns from Nordic (nrf-softdevice) and
//! Espressif (esp-wifi-sys) where radio protocols are board-level concerns rather
//! than generic HAL peripherals.
//!
//! # Example
//!
//! ```no_run
//! use xiao_mg24_bsp::{Board, radio::ble::XiaoBle};
//!
//! let board = Board::take().unwrap();
//! let mut ble = XiaoBle::beacon(&board, "XIAO Beacon")?;
//! // Beacon is now advertising
//! ```

// BLE module (Bluetooth Low Energy 5.3)
pub mod ble;

// Re-export BLE error type from bluetooth-sys
pub use efr32mg24_bluetooth_sys::blocking::BleError;
