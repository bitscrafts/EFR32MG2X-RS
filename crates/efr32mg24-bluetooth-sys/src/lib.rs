#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

//! Silicon Labs Bluetooth Stack (BGAPI) for EFR32MG24
//!
//! This crate provides both raw FFI bindings and safe Rust wrappers for the
//! Silicon Labs Bluetooth stack.
//!
//! # Blocking API (Recommended)
//!
//! The blocking API provides a safe, synchronous wrapper around BGAPI:
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
//!             BleEvent::Connected { .. } => { /* handle connection */ }
//!             BleEvent::GattWrite { attribute, value, .. } => { /* handle write */ }
//!             _ => {}
//!         }
//!     }
//!     delay.delay_ms(10);
//! }
//! ```
//!
//! # Raw FFI (Advanced Users)
//!
//! The `ffi` module provides raw, unsafe bindings to BGAPI:
//!
//! ```no_run
//! use efr32mg24_bluetooth_sys::ffi;
//!
//! unsafe {
//!     ffi::sl_bt_system_start_bluetooth();
//!     // ...
//! }
//! ```
//!
//! **Safety**: All FFI functions are unsafe and require careful handling of:
//! - Memory safety: Bluetooth stack operates on hardware and memory buffers
//! - Thread safety: Bluetooth callbacks execute in interrupt/event context
//! - Initialization order: Bluetooth requires specific initialization sequences
//!
//! # MSLA License
//!
//! This crate links against Silicon Labs proprietary binary blobs licensed under
//! the Master Software License Agreement (MSLA). See LICENSE-MSLA for full terms.
//!
//! **Key restrictions**:
//! - Binary blobs may only be used with Silicon Labs hardware
//! - No reverse engineering or modification of blobs
//! - Distribution requires MSLA compliance

/// The timer the stack schedules its events on, in Rust and polled.
pub mod sleeptimer;
mod support;

// Raw FFI bindings (always available)
#[allow(clippy::missing_safety_doc)]
// ^ the safety docs are in the C headers this module is generated from, and bindgen does not carry
// them across; adding them here would be documenting generated code that regenerates on every build
pub mod ffi {
    //! Raw BGAPI FFI bindings
    //!
    //! **Safety**: All functions are `unsafe`. Use the safe wrappers in
    //! `blocking` module instead.
    //!
    //! # Example
    //!
    //! ```no_run
    //! use efr32mg24_bluetooth_sys::ffi;
    //!
    //! unsafe {
    //!     ffi::sl_bt_system_start_bluetooth();
    //! }
    //! ```
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod stack_init;

// Blocking API (always available)
pub mod blocking;

// Re-export blocking API at root for convenience
pub use blocking::*;

// The vendor blob references the `CORE_*` helpers as external symbols, and nothing in Rust calls
// them — so without this `#[link]` the archive is dropped by the linker and every one of those
// symbols comes back undefined. The definitions live in the `cmsis-stubs` crate, compiled once.
#[link(name = "cmsis_stubs", kind = "static")]
extern "C" {}
