#![no_std]
#![doc = include_str!("../README.md")]

//! The radio layer: what a product needs to talk, and nothing about which pins it uses.
//!
//! ## Where this sits
//!
//! ```text
//! efr32mg24-radio          this crate: a safe surface over the vendor stacks, and the buffers
//!      ↑                   they assume (event queue, advertising data, GATT table)
//! efr32mg24-bluetooth-sys  bindings generated from `sl_bt_api.h` + the three MSLA blobs
//! efr32mg24-hal            clock, delay, pins — the parts the stack asks for at boot
//! ```
//!
//! It is a sibling of the HAL rather than a module inside it, for one reason: the Bluetooth stack
//! is a binary blob with its own event loop, its own buffer sizes and its own licence. A HAL that
//! pulled it in would make every user of a GPIO depend on MSLA-licensed object code.
//!
//! ## What it deliberately does not do
//!
//! It does not own the radio's power, its antenna switch or its clock tree — those are board
//! knowledge, and they belong in a BSP. It does not hide the event model either: a caller polls
//! [`Bluetooth::poll_event`], and an event it does not recognise is reported rather than dropped.
//!
//! ## Bluetooth, in one screen
//!
//! ```no_run
//! use efr32mg24_radio::Bluetooth;
//!
//! let mut ble = Bluetooth::beacon(&mut delay, "xiao-mg24")?;
//! loop {
//!     if let Some(event) = ble.poll_event() {
//!         // Connected / Disconnected / GattWrite / ...
//!     }
//! }
//! ```

pub mod ble;
#[cfg(feature = "ble")]
pub mod fault;

#[cfg(feature = "ble")]
pub use ble::{BleError, BleEvent, Bluetooth, CharHandle, GapAdBuilder};

#[cfg(feature = "rail")]
compile_error!(
    "the RAIL feature is staged: its headers and blob are not vendored yet — see \
                 docs/04-reference/RADIO-NOTES.md"
);
