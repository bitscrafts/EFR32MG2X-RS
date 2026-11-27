//! # Which edge samples, and what a slave does when it is late
//!
//! ```
//! use efr32mg2x_spi_core::{Mode, SlaveError, SlaveRx};
//!
//! // Mode 0 samples on the leading edge, mode 1 on the trailing one.
//! assert!(Mode::Mode0.samples_on_rising());
//! assert!(!Mode::Mode1.samples_on_rising());
//!
//! // A slave cannot refuse a transfer, so a word that arrives before the last one was read is an overrun -- and the
//! // word that was there first is the one kept.
//! let mut rx = SlaveRx::new();
//! assert!(rx.store(0x11).is_ok());
//! assert_eq!(rx.store(0x22), Err(SlaveError::Overrun));
//! assert_eq!(rx.take(), Some(0x11));
//! ```
#![deny(missing_docs)]

//! Host-test harness for the SPI logic.
//!
//! The source lives in the HAL, because the HAL is the crate that gets published and a
//! published crate cannot depend on a path outside its own package. This file compiles
//! that same file, so `cargo test -p efr32mg2x-spi-core` runs the real code on the host.
//! Do not paste the code back here: scripts/check-core-split.py fails if you do.
#![no_std]

#[path = "../../../crates/efr32mg2x-hal/src/cores/spi.rs"]
mod spi;
pub use spi::*;
