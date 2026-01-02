#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

//! Low-level FFI bindings to Silicon Labs Bluetooth stack (BGAPI).
//!
//! # Safety
//!
//! This crate provides raw, unsafe FFI bindings to the Silicon Labs Bluetooth stack.
//! All functions are `unsafe` and require careful handling of:
//!
//! - Memory safety: Bluetooth stack operates on hardware and memory buffers
//! - Thread safety: Bluetooth callbacks execute in interrupt/event context
//! - Lifetime management: Bluetooth handles must outlive their associated resources
//! - Initialization order: Bluetooth requires specific initialization sequences
//!
//! **Do not use this crate directly.** Use the safe wrapper in `efr32mg24-hal::radio::ble` instead.
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
//! 2. Extract `libbluetooth_efr32xg24_gcc_release.a` to `blobs/efr32mg24b220/`
//! 3. Extract `sl_bt_api.h` and dependencies to `vendor/bluetooth/`
//! 4. Build the crate (bindings will be auto-generated)
//!
//! ```toml
//! [dependencies]
//! efr32mg24-bluetooth-sys = "0.1"
//! ```
//!
//! # Example (Unsafe!)
//!
//! ```no_run
//! use efr32mg24_bluetooth_sys as bt;
//!
//! // SAFETY: This is highly unsafe and provided only as an example.
//! // Actual Bluetooth initialization requires extensive setup not shown here.
//! unsafe {
//!     // Initialize Bluetooth stack (requires proper configuration)
//!     // sl_bt_init_stack(&config);
//!
//!     // Set device name
//!     // sl_bt_gatt_server_write_attribute_value(...);
//!
//!     // Start advertising
//!     // sl_bt_advertiser_start(...);
//! }
//! ```
//!
//! **Again: Do not use this API directly. Use `efr32mg24-hal::radio::ble` and the BSP instead.**

// Include the auto-generated bindings from build.rs
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
