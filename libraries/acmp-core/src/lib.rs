//! The `ACMP` ladder arithmetic, compiled on the host so its tests run in the gate.
//!
//! The HAL cannot be compiled for an `x86_64` host -- it contains ARM `asm!` -- so `cores/acmp.rs` is the only place the
//! driver's arithmetic can be observed without a board. This crate includes that file by path, which keeps one copy and
//! runs the tests that live next to it (`cargo test -p efr32mg2x-acmp-core`).
//!
//! What is tested is the mapping from a ladder tap to a voltage and the interval a crossing brackets; what is **not**
//! tested, because it cannot be, is whether the part's ladder follows that mapping. That answer is on the board:
//! `docs/04-reference/ACMP-LADDER-2026-11-27.md`.
//!
//! # The sweep, as arithmetic
//!
//! ```
//! use efr32mg2x_acmp_core::{ladder_mv, overlap, window_mv, LADDER_FULL};
//!
//! // The pass-through tap divides by one, so 63 is the reference itself.
//! assert_eq!(ladder_mv(1_250, LADDER_FULL), 1_250);
//!
//! // A sweep of a 1.25 V input reads low from tap 42 on, so the signal lies between taps 41 and 42: 19.8 mV per step on
//! // this reference, and the interval is open at both ends.
//! let first = window_mv(1_250, 42).expect("tap 42 is inside the ladder");
//! assert_eq!(first, (813, 833));
//!
//! // A second sweep of the same input against a 2.5 V reference is an independent measurement of one voltage. The two
//! // can only both be right if their intervals intersect, which is what makes a disagreement visible instead of silent.
//! let second = window_mv(2_500, 21).expect("tap 21 is inside the ladder");
//! assert_eq!(overlap(first, second), Some((813, 833)));
//! assert_eq!(overlap(first, (900, 950)), None, "windows that miss cannot both be right");
//!
//! // A sweep that never turned is a failed measurement, not an interval.
//! assert_eq!(window_mv(1_250, LADDER_FULL + 1), None);
//! ```
#![deny(missing_docs)]
#![no_std]
#![forbid(unsafe_code)]

#[path = "../../../crates/efr32mg2x-hal/src/cores/acmp.rs"]
mod driver;

pub use driver::{ladder_mv, overlap, window_mv, LADDER_FULL};
