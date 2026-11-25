//! Host-side copy of the claim table, so `Slots` keeps its tests.
//!
//! The source lives in the HAL (`crates/efr32mg2x-hal/src/cores/slots.rs`) and is compiled here through `#[path]`, the
//! same arrangement the other `libraries/*-core` crates use: one copy, and the tests run on the host.
//!
//! # The normal path, and the one the table exists for
//!
//! ```
//! use efr32mg2x_slots_core::{Slot, Slots, Taken};
//!
//! let mut slots = Slots::new();
//! assert!(slots.claim(Slot::Sysrtc).is_ok());
//!
//! // The second ask is refused, and it says what was refused.
//! assert_eq!(slots.claim(Slot::Sysrtc), Err(Taken(Slot::Sysrtc)));
//!
//! // A refusal leaves the *other* slots free: one peripheral taken twice must not cost a second, unrelated one.
//! assert!(slots.claim(Slot::Adc).is_ok());
//! ```

#![no_std]
#![forbid(unsafe_code)]

#[path = "../../../crates/efr32mg2x-hal/src/cores/slots.rs"]
mod slots;

pub use slots::{Slot, Slots, Taken, SLOT_COUNT};
