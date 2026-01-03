//! Blocking Bluetooth API
//!
//! Synchronous/polling-based Bluetooth stack for use without async runtime.
//!
//! # Overview
//!
//! This module provides a safe, blocking wrapper around Silicon Labs BGAPI.
//! All operations are synchronous and event-driven via polling.
//!
//! # Example
//!
//! ```no_run
//! use efr32mg24_bluetooth_sys::blocking::{Bluetooth, BleEvent};
//!
//! let mut ble = Bluetooth::new(&mut delay)?;
//! ble.set_device_name("My Device")?;
//! ble.start_connectable(100, 0)?;
//!
//! loop {
//!     if let Some(event) = ble.poll_event() {
//!         match event {
//!             BleEvent::Connected { .. } => { /* handle */ }
//!             BleEvent::GattWrite { attribute, value, .. } => { /* handle */ }
//!             _ => {}
//!         }
//!     }
//!     delay.delay_ms(10);
//! }
//! ```

mod error;
mod events;
mod stack;
mod types;

pub use error::BleError;
pub use events::BleEvent;
pub use stack::Bluetooth;
pub use types::{CharHandle, GapAdBuilder};
