//! Host-test harness for the dma logic.
//!
//! The source lives in the HAL, because the HAL is the crate that gets published and a
//! published crate cannot depend on a path outside its own package. This file compiles
//! that same file, so the host tests keep running here.
//! Do not paste the code back: scripts/check-core-split.py fails if you do.

#![no_std]

#[path = "../../../crates/efr32mg2x-hal/src/cores/dma.rs"]
mod dma;
pub use dma::*;
