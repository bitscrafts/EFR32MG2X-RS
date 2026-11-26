//! One of each: the claim that stops the same peripheral being handed out twice.
//!
//! ## The defect this closes
//!
//! `Board` is the crate's singleton — `Board::take` can succeed once — but every hand-out method took `&self` and built
//! a driver from `pac::X::steal()`. A `steal()` is a zero-sized handle to a fixed address, so `board.wdog(cfg)` called
//! twice returned two `Wdog` values over the same watchdog, `board.sysrtc()` twice returned two counters over the same
//! `SYSRTC0`, and nothing anywhere objected. Review finding F10, graded C for exactly this: the type system cannot
//! distinguish two handles to one peripheral, and the crate's own claim to be a singleton was not enforced by anything.
//!
//! ## What a runtime claim can and cannot do
//!
//! It cannot make the compiler refuse the second request — that would need the peripheral *owned* by the driver and
//! moved out of the board, which is a redesign of every `Board::x` signature and of the drivers behind them. It can
//! make the second request fail, loudly, at the point the caller asked for it, and it can be tested on the host because
//! the bookkeeping is arithmetic over ten booleans and nothing else.
//!
//! The alternative — leaving it unenforced and documenting "call it once" — was already tried by this project in other
//! forms. A rule that lives only in a comment is a rule that is broken silently; a refusal is visible.

/// Every peripheral this crate hands out. One value each, for the life of the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// `WDOG0` — reloading a watchdog from two owners is a watchdog nobody can reason about.
    Wdog,
    /// The `EMU` — energy mode transitions from two owners fight each other.
    Emu,
    /// The MVP co-processor — one instruction queue.
    Mvp,
    /// `SYSRTC0` — one counter, one compare channel set.
    Sysrtc,
    /// `USART0` — one set of pins.
    Usart0,
    /// `EUSART0`, the board's SPI.
    Spi,
    /// The general-purpose `IADC0`.
    Adc,
    /// The microphone's converter, which is the same `IADC0` configured for one pin.
    Converter,
    /// `I2C0`.
    I2c,
    /// `TIMER0` — the counter behind every PWM and buzzer path.
    Timer0,
    /// `GPCRC` — one CRC engine, one message being checksummed at a time.
    Gpcrc,
    /// `LETIMER0` — one counter, one pair of compare channels, one wake-up source.
    Letimer,
    /// The PRS — one routing fabric; two owners would select and pulse each other's channels.
    Prs,
}

/// How many slots there are, so the table and the enum cannot drift apart.
pub const SLOT_COUNT: usize = 13;

impl Slot {
    /// The index of this slot in the table.
    fn index(self) -> usize {
        match self {
            Slot::Wdog => 0,
            Slot::Emu => 1,
            Slot::Mvp => 2,
            Slot::Sysrtc => 3,
            Slot::Usart0 => 4,
            Slot::Spi => 5,
            Slot::Adc => 6,
            Slot::Converter => 7,
            Slot::I2c => 8,
            Slot::Timer0 => 9,
            Slot::Gpcrc => 10,
            Slot::Letimer => 11,
            Slot::Prs => 12,
        }
    }
}

/// A peripheral that was already handed out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Taken(pub Slot);

/// Which peripherals have been handed out.
#[derive(Clone, Copy)]
pub struct Slots {
    taken: [bool; SLOT_COUNT],
}

impl Default for Slots {
    fn default() -> Self {
        Self::new()
    }
}

impl Slots {
    /// Nothing handed out yet.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            taken: [false; SLOT_COUNT],
        }
    }

    /// Claim `slot`, or say it is gone.
    ///
    /// The refusal carries the slot, so the caller's message can name what it asked for twice instead of "an error".
    pub fn claim(&mut self, slot: Slot) -> Result<(), Taken> {
        let entrada = &mut self.taken[slot.index()];
        if *entrada {
            return Err(Taken(slot));
        }
        *entrada = true;
        Ok(())
    }

    /// Whether `slot` has been handed out.
    #[must_use]
    pub fn is_taken(&self, slot: Slot) -> bool {
        self.taken[slot.index()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_claim_succeeds() {
        let mut slots = Slots::new();
        assert_eq!(slots.claim(Slot::Wdog), Ok(()));
        assert!(slots.is_taken(Slot::Wdog));
    }

    /// The whole point: the second request for the same peripheral is refused, and the error names it.
    #[test]
    fn the_second_claim_of_the_same_slot_is_refused() {
        let mut slots = Slots::new();
        assert_eq!(slots.claim(Slot::Sysrtc), Ok(()));
        assert_eq!(slots.claim(Slot::Sysrtc), Err(Taken(Slot::Sysrtc)));
    }

    /// A refusal must not be a silent reset: the third attempt is refused too.
    #[test]
    fn a_refusal_does_not_free_the_slot() {
        let mut slots = Slots::new();
        let _ = slots.claim(Slot::Timer0);
        let _ = slots.claim(Slot::Timer0);
        assert_eq!(slots.claim(Slot::Timer0), Err(Taken(Slot::Timer0)));
        assert!(slots.is_taken(Slot::Timer0));
    }

    /// The near-miss that keeps the three tests above honest: a *different* slot still works after a refusal, so this
    /// table is not simply refusing everything.
    #[test]
    fn a_refusal_leaves_the_other_slots_free() {
        let mut slots = Slots::new();
        assert_eq!(slots.claim(Slot::Adc), Ok(()));
        assert_eq!(slots.claim(Slot::Adc), Err(Taken(Slot::Adc)));
        assert_eq!(slots.claim(Slot::Converter), Ok(()));
        assert!(!slots.is_taken(Slot::I2c));
    }

    /// Every slot claims once and only once -- written as a loop so a slot added to the enum without an index is
    /// caught here rather than by two drivers fighting over one peripheral.
    #[test]
    fn every_slot_is_claimable_once() {
        let mut slots = Slots::new();
        for slot in [
            Slot::Wdog,
            Slot::Emu,
            Slot::Mvp,
            Slot::Sysrtc,
            Slot::Usart0,
            Slot::Spi,
            Slot::Adc,
            Slot::Converter,
            Slot::I2c,
            Slot::Timer0,
            Slot::Gpcrc,
            Slot::Letimer,
            Slot::Prs,
        ] {
            assert_eq!(slots.claim(slot), Ok(()), "{slot:?} could not be claimed");
            assert_eq!(
                slots.claim(slot),
                Err(Taken(slot)),
                "{slot:?} was claimable twice"
            );
        }
    }
}
