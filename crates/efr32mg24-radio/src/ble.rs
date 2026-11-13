//! Bluetooth Low Energy, on the Silicon Labs stack that ships as a blob.
//!
//! ## What is re-exported, and why here
//!
//! The bindings crate already contains a blocking wrapper around BGAPI: it builds the command
//! layout, waits for the command-complete event, and decodes the payloads out of the emulated
//! unions. That code is good and it is the part that must not be duplicated — but a crate named
//! `-sys` promises raw bindings and nothing else, and an application should not have to reach
//! through one to find a usable API.
//!
//! So this module re-exports it, at the level an application wants, and adds the two things the
//! bindings crate cannot know: which clock and interrupt configuration the stack expects at boot,
//! and what happens when it is asked for something it does not implement.
//!
//! ## The event model, stated plainly
//!
//! BGAPI is asynchronous: a command returns `sl_status_t` immediately, and the answer arrives as an
//! event. [`Bluetooth::poll_event`] drains one event per call, which means a caller who forgets to
//! poll will see the stack stop accepting new commands — the queue is finite and the stack does not
//! grow it. Every example in this crate polls in its main loop for that reason.

pub use efr32mg24_bluetooth_sys::blocking::{
    BleError, BleEvent, Bluetooth, CharHandle, GapAdBuilder,
};

/// The maximum advertising payload the legacy advertiser can carry, in bytes.
///
/// It is 31 and not 255: legacy advertising has three channels and one 31-byte PDU, and the
/// extended advertiser that lifts the limit is a different command set. A name plus a 128-bit UUID
/// already costs 18 of those bytes.
pub const LEGACY_ADV_MTU: usize = 31;

/// The ATT MTU this stack negotiates by default.
pub const DEFAULT_ATT_MTU: u16 = 247;

/// **The handler the vendor's bare-metal design requires, in the library so every example has it.**
///
/// Without a kernel, the host asks for link-layer work by setting `PENDSVSET`; `sl_bluetooth.c.jinja` defines
/// `PendSV_Handler()` as `sl_bt_priority_handle()` for exactly that reason. Measured on this board: with the start
/// finally succeeding and no handler defined, the firmware sat in `DefaultHandler_` with `ICSR.VECTACTIVE = 14` —
/// no fault, no status, the "hang" this project chased for hours.
///
/// It lives here rather than in each example because it is a property of using the stack, not of a program:
/// an example that forgot it would hang in a way that looks like a radio failure.
#[cfg(feature = "stack")]
#[cortex_m_rt::exception]
fn PendSV() {
    efr32mg24_bluetooth_sys::stack_init::service_link_layer();
}
