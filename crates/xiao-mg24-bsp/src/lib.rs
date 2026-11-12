#![no_std]
#![doc = include_str!("../README.md")]

//! Board support package for the Seeed XIAO MG24 family.
//!
//! ## What this crate knows, and what it refuses to know
//!
//! It knows the **module**: the user LED on `PA7` (active low), and — behind the `sense` feature —
//! the LSM6DS3TR-C on `PB2`/`PB3` with its power gate on `PD5`, and the analog microphone on
//! `PC8` (enable) and `PC9` (signal).
//!
//! It refuses to know the Seeed Expansion Base. The OLED, the BME68x, the RTC, the buzzer and the
//! button are on the carrier, not on this board, and they live in `xiao-expansion-base-bsp` — a
//! crate that depends on this one. A user who owns only the XIAO never compiles that code.
//!
//! ## The layering, stated once
//!
//! ```text
//! efr32mg24-pac    registers, generated from the SVD
//!      ↑
//! efr32mg24-hal    drivers and pin primitives; never names a board's wiring
//!      ↑
//! this crate       which pin the LED is on, which bus the IMU is on, which gate powers it
//!      ↑
//! application      asks for `led()`, `imu()`, `microphone()`; never sees an address or a pin
//! ```
//!
//! ## The Sense is a feature, not a crate
//!
//! The XIAO MG24 and the XIAO MG24 Sense are the same PCB; the Sense has more components fitted.
//! Two crates would duplicate the boot, the clock tree and the LED, and would let one of them
//! drift. `--features sense` adds the sensors and nothing else.
//!
//! ## Quick start
//!
//! ```no_run
//! use xiao_mg24_bsp::Module;
//!
//! # fn main() -> ! {
//! let module = Module::take().expect("peripherals already taken");
//! let mut led = module.led();
//! loop {
//!     led.toggle();
//! }
//! # }
//! ```

pub mod module;

pub use module::Module;

// The HAL is re-exported because a BSP that hides the chip entirely is a BSP that cannot be
// used for anything its author did not anticipate. The PAC deliberately is not: an application
// holding the PAC can reconfigure what this crate claims to own.
pub use efr32mg24_hal as hal;

// What an example needs to start a binary, re-exported so that an example names only this crate.
// (`hal` is re-exported whole below; these two are the pieces every example uses.)
pub use efr32mg24_hal::{entry, prelude};
