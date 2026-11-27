//! Host-test harness for the sysrtc logic.
//!
//! The source lives in the HAL, because the HAL is the crate that gets published and a
//! published crate cannot depend on a path outside its own package. This file compiles
//! that same file, so the host tests keep running here.
//! Do not paste the code back: scripts/check-core-split.py fails if you do.
//!
//! # Deadlines across the wrap, and a counter widened without one
//!
//! ```
//! use efr32mg2x_sysrtc_core::{
//!     compare_target, has_passed, is_future, ticks_for_ms, ticks_until, WideTick,
//! };
//!
//! // A deadline on the far side of the 32-bit wrap is 301 ticks away, not 2^32 - 301: the subtraction is wrapping, and
//! // that is the whole correction -- a plain `<` calls this deadline "in the future" for the next 36.4 hours.
//! let now = u32::MAX - 100;
//! assert!(is_future(now, 200));
//! assert_eq!(ticks_until(now, 200), 301);
//!
//! // A deadline that has already gone -- and one exactly equal to now, which the compare would never match because the
//! // counter is already sitting on it -- is reprogrammed one tick ahead and reported late, instead of hanging the waiter
//! // until the counter wraps.
//! assert_eq!(compare_target(10_000, 9_000), (10_001, true));
//! assert_eq!(compare_target(500, 500), (501, true));
//! assert_eq!(compare_target(500, 1_500), (1_500, false));
//! assert!(has_passed(2_000, 1_000));
//!
//! // Widening by polling: a reading smaller than the last one can only be the wrap, so the high word counts it and the
//! // tick never goes backwards.
//! let mut tick = WideTick::new();
//! assert_eq!(tick.update(0xFFFF_FFF0), 0xFFFF_FFF0);
//! assert_eq!(tick.update(5), (1u64 << 32) + 5, "the wrap appears in the high word");
//!
//! // 32.768 kHz: a second is 32768 ticks, and half a second is 16384 of them.
//! assert_eq!(ticks_for_ms(1_000), 32_768);
//! assert_eq!(ticks_for_ms(500), 16_384);
//! ```
#![deny(missing_docs)]
#![no_std]

#[path = "../../../crates/efr32mg2x-hal/src/cores/sysrtc.rs"]
mod sysrtc;
pub use sysrtc::*;
