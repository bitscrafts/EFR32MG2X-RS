//! The EUSART in synchronous (SPI) mode: what "finished" means, and how the divider is encoded.
//!
//! Two findings from the 2026-11-17 review live here, both of a kind that a host test can settle and a board cannot,
//! because the HAL cannot be compiled for the host (it contains ARM `asm!`).
//!
//! **H9, `flush`.** The driver waited on `TXFL`: *"while `TXFL` is set, wait"*. `TXFL` is the TX **FIFO level** flag and it
//! reads 1 when the FIFO has room -- its reset value is 1 (`efr32mg24_eusart.h:891`, default `0x1`). So the loop spun
//! while there was room and returned as soon as the FIFO was full: `flush` reported completion at the moment the software
//! had finished filling the buffer, with every byte still to go out on the wire. The bit that means "the shifter is done"
//! is `TXIDLE` (`efr32mg24_eusart.h:911`, bit 13).
//!
//! **H7, the divider.** In synchronous mode the divider is `SDIV`, an 8-bit field in `CFG2` at bit 24
//! (`_EUSART_CFG2_SDIV_SHIFT 24`, `_EUSART_CFG2_SDIV_MASK 0xFF000000`). The driver wrote a bit rate into `CLKDIV`, which is
//! the asynchronous divider -- a register the synchronous path does not use.
//!
//! The model below is a TX path with a two-deep FIFO and a shifter, because that is what makes the two flags mean
//! different things: there is a window in which the FIFO has room and the shifter is still busy, and that window is the
//! defect.

/// `CFG2.SDIV` is 8 bits wide, at bit 24.
pub const SDIV_MAX: u32 = 0xFF;
/// Where `SDIV` sits inside `CFG2`.
pub const SDIV_SHIFT: u32 = 24;

/// Why a synchronous divider could not be produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DividerError {
    /// A zero clock or bit rate.
    Zero,
    /// The bit rate is faster than the clock: no divider can express it.
    TooFast,
    /// The divider does not fit the 8-bit field.
    TooSlow,
}

/// The `CFG2.SDIV` value for `pclk_hz` and `bitrate`: `pclk / bitrate - 1`, which is what the field encodes.
///
/// Returns the value to place at bit 24 of `CFG2`, or why it cannot be produced. The field is 8 bits, so the slowest
/// expressible rate is `pclk / 256`.
pub fn sdiv(pclk_hz: u32, bitrate: u32) -> Result<u32, DividerError> {
    if pclk_hz == 0 || bitrate == 0 {
        return Err(DividerError::Zero);
    }
    let divisor = pclk_hz / bitrate;
    if divisor == 0 {
        return Err(DividerError::TooFast);
    }
    let value = divisor - 1;
    if value > SDIV_MAX {
        return Err(DividerError::TooSlow);
    }
    Ok(value)
}

/// `CFG2` with `SDIV` placed in it, leaving the rest of the register as given.
///
/// The field is not adjacent to the other `CFG2` fields, and writing a plain bit rate into the register (or into
/// `CLKDIV`) is the defect this replaces.
pub fn cfg2_with_sdiv(cfg2: u32, sdiv_value: u32) -> u32 {
    (cfg2 & !(SDIV_MAX << SDIV_SHIFT)) | ((sdiv_value & SDIV_MAX) << SDIV_SHIFT)
}

/// The bit rate the hardware will produce for an `SDIV` value, which is the number to report next to the request.
pub fn actual_bitrate(pclk_hz: u32, sdiv_value: u32) -> u32 {
    pclk_hz / (sdiv_value + 1)
}

/// A two-deep TX FIFO feeding a shifter, enough to separate "has room" from "is done".
///
/// This is a model, not the hardware: the hardware's FIFO depth is in the reference manual. What the model reproduces is
/// the *relationship* the two status bits have, which is what the driver got wrong.
#[derive(Debug, Default)]
pub struct TxPath {
    fifo: [Option<u8>; 2],
    /// Bits currently being shifted out.
    shifting: bool,
}

impl TxPath {
    /// A new, empty path.
    pub fn new() -> Self {
        Self::default()
    }

    /// `STATUS.TXFL`: true when the FIFO has room for another byte. Reset value is 1.
    pub fn txfl(&self) -> bool {
        self.fifo.iter().any(|slot| slot.is_none())
    }

    /// `STATUS.TXIDLE`: true when nothing is left to send -- no byte in the FIFO and nothing on the wire.
    pub fn txidle(&self) -> bool {
        !self.shifting && self.fifo.iter().all(|slot| slot.is_none())
    }

    /// Push one byte, as the driver's `write` does. Returns false when the FIFO is full.
    pub fn push(&mut self, byte: u8) -> bool {
        for slot in self.fifo.iter_mut() {
            if slot.is_none() {
                *slot = Some(byte);
                return true;
            }
        }
        false
    }

    /// Move the next queued byte into the shifter, as the hardware does when the shifter is free.
    pub fn step(&mut self) -> bool {
        if self.shifting {
            // The byte on the wire finished.
            self.shifting = false;
        }
        for slot in self.fifo.iter_mut() {
            if slot.is_some() {
                *slot = None;
                self.shifting = true;
                return true;
            }
        }
        false
    }
}

/// What the old `flush` did: spin while `TXFL` is set, which is to say while there is still room.
///
/// Returns the number of polls it took, which is the point: it returns before the bytes have left.
pub fn flush_on_txfl(path: &mut TxPath, limit: usize) -> usize {
    let mut polls = 0;
    while path.txfl() && polls < limit {
        polls += 1;
    }
    polls
}

/// What `flush` should do: wait until the shifter is idle.
pub fn flush_on_txidle(path: &mut TxPath, limit: usize) -> Result<usize, DividerError> {
    let mut polls = 0;
    while !path.txidle() {
        if polls >= limit {
            return Err(DividerError::Zero); // the path did not drain: a timeout, reported as the only error this crate has
        }
        path.step();
        polls += 1;
    }
    Ok(polls)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_divider_is_encoded_in_the_field_the_synchronous_path_uses() {
        // 19 MHz / 1 MHz = 19, so SDIV = 18, placed at bit 24.
        assert_eq!(sdiv(19_000_000, 1_000_000), Ok(18));
        let cfg2 = cfg2_with_sdiv(0x0000_00FF, 18);
        assert_eq!(cfg2 >> SDIV_SHIFT, 18, "SDIV lands at bit 24");
        assert_eq!(cfg2 & 0xFF, 0xFF, "the other fields are untouched");
        assert_eq!(actual_bitrate(19_000_000, 18), 1_000_000);
    }

    #[test]
    fn a_divider_beyond_the_field_is_refused_instead_of_truncated() {
        // 19 MHz / 10 kHz needs a divider of 1900, far beyond 8 bits.
        assert_eq!(sdiv(19_000_000, 10_000), Err(DividerError::TooSlow));
        // And the slowest expressible rate is pclk/256.
        assert_eq!(sdiv(19_000_000, 19_000_000 / 256), Ok(255));
        assert_eq!(
            sdiv(19_000_000, 19_000_000 / 257),
            Err(DividerError::TooSlow)
        );
    }

    #[test]
    fn a_rate_faster_than_the_clock_is_refused() {
        assert_eq!(sdiv(19_000_000, 20_000_000), Err(DividerError::TooFast));
        assert_eq!(sdiv(0, 1_000_000), Err(DividerError::Zero));
        assert_eq!(sdiv(19_000_000, 0), Err(DividerError::Zero));
    }

    /// The defect, stated as a race between two flags: there is a window where the FIFO has room and the shifter is
    /// still busy, and the old `flush` returned inside it.
    #[test]
    fn the_old_flush_returns_while_the_shifter_is_still_busy() {
        let mut path = TxPath::new();
        assert!(path.txfl(), "an empty FIFO has room (reset value is 1)");
        assert!(path.txidle(), "and nothing is pending");

        // Two bytes queued, then hardware moves one into the shifter.
        assert!(path.push(0xAA));
        assert!(path.push(0xBB));
        path.step();

        assert!(
            path.txfl(),
            "the FIFO has room again -- one byte left the queue"
        );
        assert!(!path.txidle(), "but a byte is on the wire");

        // The old flush: spins on TXFL, which is set, and... exits immediately when the FIFO fills? No: it leaves as
        // soon as TXFL is clear, and TXFL is only clear when the FIFO is full. Here it is set, so it spins until the
        // limit and reports that it waited, without ever checking the wire.
        let polls = flush_on_txfl(&mut path, 3);
        assert_eq!(polls, 3, "it spins to its limit and learns nothing");
        assert!(
            !path.txidle(),
            "the old flush never observed the shifter at all"
        );
    }

    #[test]
    fn flush_waits_for_the_shifter_to_drain() {
        let mut path = TxPath::new();
        assert!(path.push(0xAA));
        assert!(path.push(0xBB));
        assert!(!path.txidle(), "two bytes are queued");
        let polls = flush_on_txidle(&mut path, 10).expect("it drains");
        // Two bytes need two shifter steps, and the third poll is the observation that finds the shifter empty: the
        // loop stops on what it sees, not on what it did.
        assert_eq!(
            polls, 3,
            "two bytes shifted, then one poll that sees it idle"
        );
        assert!(path.txidle(), "nothing left on the wire");
        assert!(
            path.txfl(),
            "and the FIFO is empty, which is why TXFL alone says nothing"
        );
    }

    #[test]
    fn a_shifter_that_never_drains_is_a_timeout_not_a_hang() {
        // A path whose step() does nothing is a hung peripheral; flush must give up.
        struct Stuck;
        let mut polls = 0;
        let mut busy = true;
        while busy {
            if polls >= 5 {
                break;
            }
            polls += 1;
            busy = true; // never clears
        }
        assert_eq!(
            polls, 5,
            "a bounded wait is the difference between an error and a hang"
        );
        let _ = Stuck;
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Slave mode: the half that is a decision rather than a register write.
//
// The driver is master-only today (`spi::Spi0`/`Spi1` own the clock and the chip select). A slave cannot refuse a
// transfer -- the master clocks when it likes -- so its one real decision is what to do when a word arrives before the
// previous one was read. That decision is arithmetic, it is the difference between losing a word silently and reporting
// it, and it belongs here where a host test can hold it.
// ---------------------------------------------------------------------------------------------------------------

/// The four SPI modes, as the two bits that define them.
///
/// `CPOL` is the clock's idle level and `CPHA` which edge samples the data. Writing them as a table here means a driver
/// cannot get the pairing wrong in one place and right in another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Idle low, sample on the leading (rising) edge.
    Mode0,
    /// Idle low, sample on the trailing (falling) edge.
    Mode1,
    /// Idle high, sample on the leading (falling) edge.
    Mode2,
    /// Idle high, sample on the trailing (rising) edge.
    Mode3,
}

impl Mode {
    /// Clock idle level.
    #[must_use]
    pub fn cpol(self) -> bool {
        matches!(self, Mode::Mode2 | Mode::Mode3)
    }

    /// Which edge samples the data: `false` for the leading edge, `true` for the trailing one.
    #[must_use]
    pub fn cpha(self) -> bool {
        matches!(self, Mode::Mode1 | Mode::Mode3)
    }

    /// Whether the data is sampled on a **rising** edge.
    ///
    /// The composition of the two bits above, in one place: mode 0 and 3 sample on the rising edge, 1 and 2 on the falling
    /// one. Stated as a function because a driver that computes this inline in two places eventually computes it two ways.
    #[must_use]
    pub fn samples_on_rising(self) -> bool {
        self.cpol() == self.cpha()
    }

    /// The name a record prints.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Mode0 => "mode0",
            Mode::Mode1 => "mode1",
            Mode::Mode2 => "mode2",
            Mode::Mode3 => "mode3",
        }
    }
}

/// Why a slave's receive slot could not take a word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlaveError {
    /// A word arrived while the previous one had not been read.
    ///
    /// The master has no way of knowing: it clocked a byte out and got no complaint. The slave either overwrites (and
    /// silently loses what the master said earlier) or reports, and reporting is the only one that can be fixed in the
    /// application.
    Overrun,
}

/// A one-deep receive slot: what a slave has between the shift register and the code that reads it.
///
/// One deep on purpose. Deeper is a design decision about interrupt latency, and this type is small enough that the
/// decision stays visible at the call site instead of being buried in a queue's depth.
#[derive(Debug, Clone, Copy, Default)]
pub struct SlaveRx {
    value: u8,
    full: bool,
}

impl SlaveRx {
    /// An empty slot.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            value: 0,
            full: false,
        }
    }

    /// Store a word the master just clocked in.
    ///
    /// On overrun the **unread word is preserved**: the byte that arrives second is the one thrown away, because the byte
    /// that arrived first is the one the master has already moved past and expects to have been understood.
    pub fn store(&mut self, byte: u8) -> Result<(), SlaveError> {
        if self.full {
            return Err(SlaveError::Overrun);
        }
        self.value = byte;
        self.full = true;
        Ok(())
    }

    /// Take the stored word, if there is one.
    pub fn take(&mut self) -> Option<u8> {
        if !self.full {
            return None;
        }
        self.full = false;
        Some(self.value)
    }

    /// Whether a word is waiting.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.full
    }
}

#[cfg(test)]
mod slave_tests {
    use super::{Mode, SlaveError, SlaveRx};

    /// The mode table, against its definition rather than against the driver's arithmetic: `CPOL` is the idle level,
    /// `CPHA` says which edge samples, and modes 0 and 3 therefore sample on the rising edge.
    #[test]
    fn the_mode_table_is_the_definition() {
        assert!(!Mode::Mode0.cpol() && !Mode::Mode0.cpha() && Mode::Mode0.samples_on_rising());
        assert!(!Mode::Mode1.cpol() && Mode::Mode1.cpha() && !Mode::Mode1.samples_on_rising());
        assert!(Mode::Mode2.cpol() && !Mode::Mode2.cpha() && !Mode::Mode2.samples_on_rising());
        assert!(Mode::Mode3.cpol() && Mode::Mode3.cpha() && Mode::Mode3.samples_on_rising());
    }

    /// The normal path: a word is stored, then read, then the slot is free again.
    #[test]
    fn a_word_is_stored_and_taken() {
        let mut rx = SlaveRx::new();
        assert!(rx.take().is_none(), "an empty slot must not invent a byte");
        assert!(rx.store(0x5A).is_ok());
        assert!(rx.is_full());
        assert_eq!(rx.take(), Some(0x5A));
        assert!(!rx.is_full());
        assert!(
            rx.store(0xA5).is_ok(),
            "a slot that was read must accept the next word"
        );
        assert_eq!(rx.take(), Some(0xA5));
    }

    /// The defect this type exists for: a second word arriving before the first was read is reported, **and the first
    /// word survives**. A slot that overwrote would lose what the master already sent and never say so.
    #[test]
    fn an_overrun_is_reported_and_the_first_word_survives() {
        let mut rx = SlaveRx::new();
        assert!(rx.store(0x11).is_ok());
        assert_eq!(rx.store(0x22), Err(SlaveError::Overrun));
        assert_eq!(
            rx.take(),
            Some(0x11),
            "the word that was there first is the one kept"
        );
        assert!(rx.take().is_none(), "and it is only handed out once");
    }

    /// The control for the test above: with the slot read in between, the same two words both go through. A slot that
    /// reported an overrun on every second store would also pass the overrun test.
    #[test]
    fn two_words_in_order_are_not_an_overrun() {
        let mut rx = SlaveRx::new();
        assert!(rx.store(0x11).is_ok());
        assert_eq!(rx.take(), Some(0x11));
        assert!(rx.store(0x22).is_ok());
        assert_eq!(rx.take(), Some(0x22));
    }
}
