//! Host-side copy of the claim table, so `Slots` keeps its tests.
//!
//! The source lives beside the HAL that uses it (`crates/efr32mg2x-hal/src/cores/slots.rs`) and is compiled here
//! through `#[path]`, the same arrangement the other `libraries/*-core` crates use: one copy, and the tests run on the
//! host where they can run at all.

#![no_std]
#![forbid(unsafe_code)]

#[path = "../../../crates/efr32mg2x-hal/src/cores/slots.rs"]
mod slots;

pub use slots::{Slot, Slots, Taken, SLOT_COUNT};
