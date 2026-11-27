//! The `ACMP` ladder arithmetic, compiled on the host so its tests run in the gate.
//!
//! The HAL cannot be compiled for an `x86_64` host -- it contains ARM `asm!` -- so `cores/acmp.rs` is the only place the
//! driver's arithmetic can be observed without a board. This crate includes that file by path, which keeps one copy and
//! runs the tests that live next to it (`cargo test -p efr32mg2x-acmp-core`).
//!
//! What is tested is the mapping from a ladder tap to a voltage and the interval a crossing brackets; what is **not**
//! tested, because it cannot be, is whether the part's ladder follows that mapping. That answer is on the board:
//! `docs/04-reference/ACMP-LADDER-2026-11-27.md`.
#![deny(missing_docs)]
#![no_std]
#![forbid(unsafe_code)]

#[path = "../../../crates/efr32mg2x-hal/src/cores/acmp.rs"]
mod driver;

pub use driver::{ladder_mv, overlap, window_mv, LADDER_FULL};
