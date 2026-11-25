//! Host-test harness for the i2c logic.
//!
//! The source lives in the HAL, because the HAL is the crate that gets published and a
//! published crate cannot depend on a path outside its own package. This file compiles
//! that same file, so the host tests keep running here.
//! Do not paste the code back: scripts/check-core-split.py fails if you do.

#![no_std]

#[path = "../../../crates/efr32mg2x-hal/src/cores/i2c/mod.rs"]
mod i2c;
pub use i2c::*;

// The mock follower and the mock bus are test-only inside the HAL now -- a test double has no business in a published
// API -- but this crate is a host harness whose whole point is running them, including from `examples/simulate.rs`.
#[path = "../../../crates/efr32mg2x-hal/src/cores/i2c/sim.rs"]
pub mod sim;
