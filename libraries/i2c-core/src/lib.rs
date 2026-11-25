//! # One transaction, with the repeated START spelled out
//!
//! The `Bus` trait is four operations, so a host test can be a mock and a real device can be the peripheral. This is the
//! whole point of the split: the sequence -- address, register, **repeated** START, address again, read, STOP -- is
//! arithmetic on a trait, and it is where the 2026-11-17 defect lived.
//!
//! ```
//! use efr32mg2x_i2c_core::{write_read, Bus, Error};
//!
//! /// A device with four registers, behind the four bus operations.
//! struct Mock {
//!     regs: [u8; 4],
//!     ptr: u8,
//!     next: usize,
//!     starts: u32,
//!     stopped: bool,
//! }
//!
//! impl Bus for Mock {
//!     fn start(&mut self) -> Result<(), Error> {
//!         self.starts += 1;
//!         Ok(())
//!     }
//!     fn write(&mut self, byte: u8) -> Result<(), Error> {
//!         // The calls alternate address, register, address: the two odd ones carry the address in bit 0.
//!         if byte & 1 == 0 && self.starts == 1 {
//!             self.ptr = 0; // address byte, write
//!         } else if self.starts == 1 {
//!             self.ptr = byte; // register byte
//!         }
//!         Ok(())
//!     }
//!     fn read(&mut self, _ack: bool) -> Result<u8, Error> {
//!         let v = self.regs[self.ptr as usize + self.next];
//!         self.next += 1;
//!         Ok(v)
//!     }
//!     fn stop(&mut self) -> Result<(), Error> {
//!         self.stopped = true;
//!         Ok(())
//!     }
//! }
//!
//! let mut bus = Mock { regs: [0x6a, 0x01, 0x02, 0x03], ptr: 0, next: 0, starts: 0, stopped: false };
//! let mut out = [0u8; 2];
//! write_read(&mut bus, 0x6a, 0x00, &mut out).expect("the mock never nacks");
//!
//! assert_eq!(out, [0x6a, 0x01]);
//! assert_eq!(bus.starts, 2, "the second START is the repeated one -- the bus is never released in between");
//! assert!(bus.stopped);
//! ```
//!
//! # A refusal releases the bus, because a NACK is not a tidy failure
//!
//! ```
//! use efr32mg2x_i2c_core::{write_read, Bus, Error};
//!
//! struct Nacking {
//!     aborted: bool,
//! }
//!
//! impl Bus for Nacking {
//!     fn start(&mut self) -> Result<(), Error> { Ok(()) }
//!     fn write(&mut self, _byte: u8) -> Result<(), Error> { Err(Error::Nack) }
//!     fn read(&mut self, _ack: bool) -> Result<u8, Error> { unreachable!("never reached") }
//!     fn stop(&mut self) -> Result<(), Error> { Ok(()) }
//!     fn abort(&mut self) { self.aborted = true; }
//! }
//!
//! let mut bus = Nacking { aborted: false };
//! let mut out = [0u8; 1];
//! assert_eq!(write_read(&mut bus, 0x6a, 0x00, &mut out), Err(Error::Nack));
//! assert!(bus.aborted, "a NACK with no STOP leaves the bus held; abort is what releases it");
//! ```
//!
//! # An empty read is refused before the bus is touched
//!
//! ```
//! # use efr32mg2x_i2c_core::{write_read, Bus, Error};
//! # struct Quiet;
//! # impl Bus for Quiet {
//! #     fn start(&mut self) -> Result<(), Error> { unreachable!("the bus must not be touched") }
//! #     fn write(&mut self, _b: u8) -> Result<(), Error> { unreachable!() }
//! #     fn read(&mut self, _a: bool) -> Result<u8, Error> { unreachable!() }
//! #     fn stop(&mut self) -> Result<(), Error> { unreachable!() }
//! # }
//! assert_eq!(write_read(&mut Quiet, 0x6a, 0x00, &mut []), Err(Error::InvalidData));
//! ```

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
