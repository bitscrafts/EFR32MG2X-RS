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
