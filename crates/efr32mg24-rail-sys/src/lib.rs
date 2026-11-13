#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

//! Low-level FFI bindings to Silicon Labs RAIL (Radio Abstraction Interface Layer).
//!
//! # Safety
//!
//! This crate provides raw, unsafe FFI bindings to the RAIL library. All functions
//! are `unsafe` and require careful handling of:
//!
//! - Memory safety: RAIL operates on hardware registers and DMA buffers
//! - Thread safety: RAIL callbacks execute in interrupt context
//! - Lifetime management: RAIL handles must outlive their associated resources
//! - Initialization order: RAIL requires specific initialization sequences
//!
//! **Do not use this crate directly.** Use the safe wrapper in `efr32mg24-hal::radio` instead.
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
//!
//! See NOTICE file for third-party attributions.
//!
//! # Usage
//!
//! To use this crate, you must:
//!
//! 1. Download Gecko SDK from Silicon Labs
//! 2. Extract `librail_efr32xg24_gcc_release.a` to `blobs/efr32mg24b220/`
//! 3. Extract `rail.h` and dependencies to `vendor/rail/`
//! 4. Build the crate (bindings will be auto-generated)
//!
//! ```toml
//! [dependencies]
//! efr32mg24-rail-sys = "0.1"
//! ```
//!
//! # Example (Unsafe!)
//!
//! ```no_run
//! use efr32mg24_rail_sys as rail;
//!
//! // SAFETY: This is highly unsafe and provided only as an example.
//! // Actual RAIL initialization requires extensive setup not shown here.
//! unsafe {
//!     // Initialize RAIL (requires proper configuration struct)
//!     // let handle = rail::RAIL_Init(&config, &post_wakeup_callback);
//! }
//! ```
//!
//! **Again: Do not use this API directly. Use `efr32mg24-hal::radio` instead.**

// Include the auto-generated bindings from build.rs
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// The Radio Configurator's BLE 39 MHz channel configuration, generated into Rust (see the module note).
pub mod ble_config;

// The vendor blob references the `CORE_*` helpers as external symbols, and nothing in Rust calls
// them — so without this `#[link]` the archive is dropped by the linker and every one of those
// symbols comes back undefined. The definitions live in the `cmsis-stubs` crate, compiled once.
#[link(name = "cmsis_stubs", kind = "static")]
extern "C" {}

// Same reason, one level up: the RAIL blob references emlib functions (`CMU_ClockEnable`,
// `SystemSYSCLKGet`, `SYSTEM_ChipRevisionGet`, `EMU_DCDCSetPFMXTimeoutMaxCtrl`) and nothing in Rust
// names them, so the emlib archive would be dropped by an unused-dependency rule and every one of
// those symbols would come back undefined despite existing two directories away.
#[link(name = "emlib_min", kind = "static")]
extern "C" {}
