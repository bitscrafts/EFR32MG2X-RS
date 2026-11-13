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

/// The stack's allocator, over `libraries/bt-heap`.
/// Link-layer crypto: AES-ECB and CCM*, on top of libraries/aes-modes.
pub mod crypto;
pub mod heap;

/// The RAIL crate, re-exported so that it is **linked**.
///
/// This is not decoration. `efr32mg24-rail-sys`'s build script carries the `+whole-archive` link of
/// `librail_efr32xg24_gcc_release.a`, and rustc only replays a dependency's native link flags when that dependency
/// is part of the final link. Bluetooth code reaches RAIL through `extern "C"` declarations, which reference no
/// Rust item, so when this crate stopped calling `RAIL_Init` the rail-sys rlib dropped out of the link line -- and
/// with it the radio library. The symptom was twenty "undefined symbol: RAIL_…" lines for functions that were
/// sitting in the archive the whole time.
pub use efr32mg24_rail_sys as rail;
/// PSA Crypto: key slots and CMAC, which is all the host stack asks of it.
pub mod psa;
/// RAIL's assertion callback: a failure that reports instead of looping.
pub mod rail_assert;
/// Hardware entropy: the Secure Engine's TRNG, through the mailbox.
pub mod se_trng;
/// The timer the stack schedules its events on, in Rust and polled.
pub mod sleeptimer;
mod support;
/// Vendor entry points answered in Rust (controller feature init, power manager, NVM3 surface).
pub mod vendor_support;

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
