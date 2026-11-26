//! The peripheral reflex system (PRS), as far as this crate drives it: one asynchronous channel, a source for it, and
//! a software pulse.
//!
//! ## What is measured, and what this module is for
//!
//! Measured 2026-11-26: a write to `ASYNC_SWPULSE` is XORed onto the channel's output, so a channel whose
//! `ASYNC_CHn_CTRL` selects no source still carries the edge. `ASYNC_CH0_CTRL` read back `0x000c0000` at reset
//! (`FNSEL = A`, `SOURCESEL` and `SIGSEL` zero) and **no write was needed** for eight pulses to become eight IADC
//! conversions (record: `docs/04-reference/IADC-PRS-TRIGGER-2026-11-26.md`).
//!
//! Measured 2026-11-26 as well, and the reason [`Source`] exists: with `SOURCESEL = TIMER0` and `SIGSEL = TIMER0UF`
//! (`0x0c2101` in `ASYNC_CH0_CTRL`) and TIMER0 free-running, eight timer periods produced eight IADC conversions with
//! no CPU pulse at all; the same channel with no source, and with a source whose timer is not clocked, produced none
//! (record: `docs/04-reference/PRS-TIMER-PRODUCER-2026-11-26.md`).
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
    /// A GPIO producer names an external-interrupt channel, and there are sixteen of those.
    NoSuchPin,
}

/// A producer a channel can be pointed at, with the encodings of the vendor's `efr32mg24_prs_signals.h`.
///
/// This is not the whole producer table: it holds the entries this crate has a user for. `SOURCESEL` is a seven-bit
/// field and the vendor header lists dozens of blocks, so an absent variant means "not encoded here", not "not
/// possible".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Nothing drives the channel; only [`Prs::pulse`] reaches it. This is the reset state.
    Off,
    /// TIMER0's overflow/underflow event (`SOURCESEL = 0x21`, `SIGSEL = 1`).
    Timer0Overflow,
    /// TIMER0's CC0 compare match (`SOURCESEL = 0x21`, `SIGSEL = 2`).
    Timer0Compare0,
    /// TIMER1's overflow/underflow event (`SOURCESEL = 0x22`, `SIGSEL = 1`). Useful as a producer that is named and
    /// silent: a timer nobody started produces nothing.
    Timer1Overflow,
    /// GPIO external-interrupt channel `n` (`SOURCESEL = 0x04`, `SIGSEL = n`). Which pin feeds channel `n` is the GPIO
    /// block's business (`EXTIPINSELn`), and this crate does not configure it.
    GpioPin(u8),
}

impl Source {
    /// The `SOURCESEL | SIGSEL` part of `ASYNC_CHn_CTRL`, with `FNSEL` left at its reset value (0xC, pass A).
    fn word(self) -> Option<u32> {
        let (source, signal) = match self {
            Source::Off => (0x00u32, 0u32),
            Source::Timer0Overflow => (0x21, 1),
            Source::Timer0Compare0 => (0x21, 2),
            Source::Timer1Overflow => (0x22, 1),
            Source::GpioPin(pin) => {
                if pin > 15 {
                    return None;
                }
                (0x04, u32::from(pin))
            }
        };
        Some((0xC << 16) | (source << 8) | signal)
    }
}

/// Offset of `ASYNC_CH0_CTRL` in the PRS block; the sixteen channel words follow it, four bytes apart.
const ASYNC_CH0_CTRL: usize = 0x018;

/// The address of asynchronous channel `channel`'s control word.
fn channel_ptr(channel: u8) -> *mut u32 {
    (pac::PrsS::ptr() as usize + ASYNC_CH0_CTRL + 4 * channel as usize) as *mut u32
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

    /// Point asynchronous `channel` at `source`, so the channel carries that producer's edges.
    ///
    /// The whole word is written, including `FNSEL = 0xC` (the reset value, which passes the selected signal
    /// unchanged): a read-modify-write would keep whatever a previous user left in `FNSEL`, and a channel that inverts
    /// or gates its producer still looks configured.
    ///
    /// Nothing here starts or clocks the producer. A timer that is not running produces no edges, which is a property
    /// the example relies on for its control phase.
    ///
    /// # Errors
    ///
    /// [`Error::NoSuchChannel`] for a channel above 15, [`Error::NoSuchPin`] for a GPIO channel above 15. Both are
    /// refused before anything is written.
    pub fn connect(&mut self, channel: u8, source: Source) -> Result<(), Error> {
        if channel > 15 {
            return Err(Error::NoSuchChannel);
        }
        let word = source.word().ok_or(Error::NoSuchPin)?;
        // SAFETY: `channel <= 15` addresses one of the sixteen channel words that `channel_ptr` computes inside the
        // block, and the word is the sum of fields whose widths the vendor header fixes (`SOURCESEL` bits 14:8,
        // `SIGSEL` bits 2:0, `FNSEL` bits 19:16).
        unsafe { core::ptr::write_volatile(channel_ptr(channel), word) };
        Ok(())
    }

    /// What asynchronous `channel` is pointed at right now, as the raw word.
    ///
    /// `None` for a channel above 15. This exists for a probe that has to report the word it configured rather than
    /// the word it meant to configure.
    pub fn channel_ctrl(&self, channel: u8) -> Option<u32> {
        if channel > 15 {
            return None;
        }
        // SAFETY: as in `connect`; a read of one of the sixteen channel words inside the block.
        Some(unsafe { core::ptr::read_volatile(channel_ptr(channel)) })
    }
}
