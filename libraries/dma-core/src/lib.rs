//! Host-test harness for the dma logic.
//!
//! The source lives in the HAL, because the HAL is the crate that gets published and a
//! published crate cannot depend on a path outside its own package. This file compiles
//! that same file, so the host tests keep running here.
//! Do not paste the code back: scripts/check-core-split.py fails if you do.
//!
//! # A transfer that stops its channel when it is dropped
//!
//! ```
//! use std::cell::Cell;
//! use std::rc::Rc;
//!
//! use efr32mg2x_dma_core::{Channel, Transfer};
//!
//! /// A channel that records the two calls the type's soundness depends on.
//! struct Counting {
//!     started: Rc<Cell<u32>>,
//!     stopped: Rc<Cell<u32>>,
//! }
//!
//! impl Channel for Counting {
//!     // SAFETY: the trait's contract is that `dst` stays valid until `stop`; this double never dereferences it, and
//!     // refuses the two arguments that mean the caller made a mistake.
//!     unsafe fn start(&mut self, dst: *mut u8, bytes: usize) {
//!         assert!(!dst.is_null() && bytes > 0, "a transfer covers real memory");
//!         self.started.set(self.started.get() + 1);
//!     }
//!     fn stop(&mut self) {
//!         self.stopped.set(self.stopped.get() + 1);
//!     }
//!     fn active(&self) -> bool {
//!         self.started.get() > self.stopped.get()
//!     }
//! }
//!
//! let started = Rc::new(Cell::new(0));
//! let stopped = Rc::new(Cell::new(0));
//! let mut buffer = [0u8; 16];
//! {
//!     let transfer = Transfer::new(
//!         Counting {
//!             started: started.clone(),
//!             stopped: stopped.clone(),
//!         },
//!         &mut buffer,
//!     );
//!     assert_eq!(transfer.len_bytes(), 16, "the whole buffer is covered");
//!     assert_eq!(
//!         transfer.buffer().len(),
//!         16,
//!         "and readable through the transfer, which holds the borrow"
//!     );
//! }
//! assert_eq!(started.get(), 1, "the channel was started once, by `new`");
//! assert_eq!(
//!     stopped.get(),
//!     1,
//!     "Drop stopped it, so the caller cannot forget to and cannot touch the buffer meanwhile"
//! );
//! ```
#![deny(missing_docs)]
#![no_std]

#[path = "../../../crates/efr32mg2x-hal/src/cores/dma.rs"]
mod dma;
pub use dma::*;
