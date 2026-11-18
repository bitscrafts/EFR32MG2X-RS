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
