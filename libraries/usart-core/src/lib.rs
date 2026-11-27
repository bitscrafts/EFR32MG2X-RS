//! # The divider, and what it actually produces
//!
//! ```
//! use efr32mg2x_usart_core::{actual_baud, clkdiv};
//!
//! // The board runs at 19 MHz; the request is 115200; asynchronous mode oversamples by 16.
//! let div = clkdiv(19_000_000, 115_200, 16).expect("115200 is reachable at 19 MHz");
//! let got = actual_baud(19_000_000, div, 16);
//!
//! // The rate the caller asked for and the rate it got are not always the same number, which is why the driver reports
//! // both.
//! assert!((got as i64 - 115_200).abs() < 600, "got {got}");
//! ```
//!
//! # A baud rate the clock cannot produce is refused, not rounded
//!
//! ```
//! use efr32mg2x_usart_core::{clkdiv, DividerError};
//!
//! // 4 Mbaud out of a 19 MHz clock would need a negative divider.
//! assert_eq!(clkdiv(19_000_000, 4_000_000, 16), Err(DividerError::TooFast));
//! ```
#![deny(missing_docs)]

//! Host-test harness for the usart logic.
//!
//! The source lives in the HAL, because the HAL is the crate that gets published and a
//! published crate cannot depend on a path outside its own package. This file compiles
//! that same file, so the host tests keep running here.
//! Do not paste the code back: scripts/check-core-split.py fails if you do.

#![no_std]

#[path = "../../../crates/efr32mg2x-hal/src/cores/usart.rs"]
mod usart;
pub use usart::*;
