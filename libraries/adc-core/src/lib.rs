//! # Turning a configured input into the word the hardware takes, and back
//!
//! ```
//! use efr32mg2x_adc_core::{bus_for, parse, Bus, Input, Port};
//!
//! let input = Input::new(Port::C, 4).expect("PC4 is a valid analogue input");
//! let word = input.single_word();
//!
//! // The word is what a record prints, and parsing it back must return the same input -- a round trip that fails is a
//! // decode that will silently attribute a reading to the wrong pin.
//! assert_eq!(parse(word).expect("the word this crate produced parses"), input);
//!
//! // What this project knows about routing that pin to the converter. One pin is measured; every other is `Unknown`,
//! // which is a statement about the evidence and not about the silicon.
//! assert_eq!(bus_for(input), Bus::Unknown);
//! ```
//!
//! # A differential word is refused rather than read as single-ended
//!
//! ```
//! use efr32mg2x_adc_core::{parse, AdcError, Input, Port, PORTNEG_SHIFT};
//!
//! // A negative port in the word means the conversion is a differential pair, which this crate does not model: it is
//! // refused rather than read as single-ended, because reading it as single-ended attributes a difference to a pin.
//! let mut word = Input::new(Port::A, 3).expect("PA3 is a valid input").single_word();
//! word |= 9 << PORTNEG_SHIFT; // a negative port, which this crate does not model
//! assert_eq!(parse(word), Err(AdcError::Differential));
//! ```
#![deny(missing_docs)]

//! Host-test harness for the adc logic.
//!
//! The source lives in the HAL, because the HAL is the crate that gets published and a
//! published crate cannot depend on a path outside its own package. This file compiles
//! that same file, so the host tests keep running here.
//! Do not paste the code back: scripts/check-core-split.py fails if you do.

#![no_std]

#[path = "../../../crates/efr32mg2x-hal/src/cores/adc.rs"]
mod adc;
pub use adc::*;
