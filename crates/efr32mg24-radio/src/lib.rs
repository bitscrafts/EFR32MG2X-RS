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
#[cfg(feature = "diag")]
pub mod diag;
#[cfg(feature = "ble")]
pub mod fault;

#[cfg(feature = "ble")]
pub use ble::{BleError, BleEvent, Bluetooth, CharHandle, GapAdBuilder};
// A BSP that builds a GATT service needs the descriptor types, and making it depend on the `-sys` crate directly would
// put two BLE layers in one manifest. Re-exporting keeps the stack a detail of this crate.
pub use efr32mg24_bluetooth_sys::gatt_db;

/// `(damaged, total, first_size, first_offset)` for the whole arena, walked linearly.
pub use efr32mg24_bluetooth_sys::heap::audit as heap_audit;
/// `(count, first_offset)` of frees of a block that was already free.
pub use efr32mg24_bluetooth_sys::heap::double_frees as heap_double_frees;
/// The largest single block the heap can still serve.
pub use efr32mg24_bluetooth_sys::heap::largest_free as heap_largest_free;
/// `(usable_size, offset)` of the last block found with a damaged canary.
pub use efr32mg24_bluetooth_sys::heap::last_damaged as heap_last_damaged;
/// The last few allocations, newest first: `(requested_size, offset)`.
pub use efr32mg24_bluetooth_sys::heap::recent as heap_recent;
/// `(in_use, failures, violations)` from the stack's heap. Re-exported so a BSP can report the heap's health without
/// depending on the `-sys` crate.
pub use efr32mg24_bluetooth_sys::heap::stats as heap_stats;
/// `(generated, exported, agreed)`: how far a pairing got through the elliptic-curve calls.
pub use efr32mg24_bluetooth_sys::psa_ec::call_counts as psa_ec_counts;

#[cfg(feature = "rail")]
compile_error!(
    "the RAIL feature is staged: its headers and blob are not vendored yet — see \
                 docs/04-reference/RADIO-NOTES.md"
);
