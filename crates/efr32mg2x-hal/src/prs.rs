//! The peripheral reflex system (PRS), as far as this crate drives it: one asynchronous channel, pulsed by software.
//!
//! ## What is measured, and what this module is for
//!
//! Measured 2026-11-26: a write to `ASYNC_SWPULSE` is XORed onto the channel's output, so a channel whose
//! `ASYNC_CHn_CTRL` selects no source still carries the edge. `ASYNC_CH0_CTRL` read back `0x000c0000` at reset
//! (`FNSEL = A`, `SOURCESEL` and `SIGSEL` zero) and **no write was needed** for eight pulses to become eight IADC
//! conversions (record: `docs/04-reference/IADC-PRS-TRIGGER-2026-11-26.md`).
//!
//! So this module exists for one reason: the IADC's triggered path needs a way to pulse a channel that is not a
//! register poke inside an example. It does **not** configure a channel, select a producer, or route a consumer —
//! those are the next measurement, and a driver that guessed them would be a driver nobody had watched work.
//!
//! ## The clock
//!
//! `CMU.CLKEN0.PRS` is a gate: the block's registers are behind a bus clock, and [`Board::prs`] opens it before handing
//! the block out. (The positive direction is measured — every run had the gate open. Nobody has run the negative one.)

use crate::pac;

/// Why the PRS refused what it was asked to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// There is no asynchronous channel above 15: `ASYNC_SWPULSE` is sixteen bits.
    NoSuchChannel,
}

/// The PRS block, handed out once by [`Board::prs`](crate::board::Board::prs).
pub struct Prs {
    regs: pac::PrsS,
}

impl Prs {
    /// SAFETY: `Board` holds the PAC singletons and hands this out through its claim table, so no second `Prs` exists.
    pub(crate) unsafe fn new(regs: pac::PrsS) -> Self {
        Self { regs }
    }

    /// One pulse on asynchronous `channel`: a single write, and the hardware makes the edge one clock wide.
    ///
    /// # Errors
    ///
    /// [`Error::NoSuchChannel`] for a channel above 15.
    pub fn pulse(&mut self, channel: u8) -> Result<(), Error> {
        if channel > 15 {
            return Err(Error::NoSuchChannel);
        }
        // SAFETY: `ASYNC_SWPULSE` is one bit per channel and `channel` was checked against 15 above.
        let bit = 1u32 << channel;
        self.regs.async_swpulse().write(|w| unsafe { w.bits(bit) });
        Ok(())
    }
}
