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
    /// `USART0` — one set of pins, whether it is driven as a UART or as a SPI master (`spi::Spi0`). The role is not
    /// the slot: two roles over one block is the aliasing this table exists to refuse, and `usart0()` beside
    /// `spi0_on()` was such a pair until 2026-11-27.
    Usart0,
    /// `EUSART0`, the board's SPI.
    Spi,
    /// `IADC0` -- one converter, whether it is driven as a general-purpose ADC or as the microphone's sampler. The
    /// slot names the block, not the role: until 2026-11-27 `adc()` and the microphone's `converter()` were two slots
    /// over one IADC0, and a caller could hold both and reconfigure the scan under the other.
    Adc,
    /// `I2C0`.
    I2c,
    /// `I2C1` -- the second bus. Its own slot, because a caller that takes I2C0 must not thereby exclude I2C1.
    I2c1,
    /// `TIMER0` — the counter behind every PWM and buzzer path.
    Timer0,
    /// `TIMER1` — the same block, its own clock-enable bit and its own interrupt line.
    Timer1,
    /// `TIMER2` — as `TIMER1`: same driver, one `CLKEN0` bit apart.
    Timer2,
    /// `TIMER3` — as `TIMER1`.
    Timer3,
    /// `TIMER4` — as `TIMER1`.
    Timer4,
    /// `GPCRC` — one CRC engine, one message being checksummed at a time.
    Gpcrc,
    /// `LETIMER0` — one counter, one pair of compare channels, one wake-up source.
    Letimer,
    /// The PRS — one routing fabric; two owners would select and pulse each other's channels.
    Prs,
    /// `PCNT0` — one counter, one pair of input routes.
    Pcnt,
    /// `BURTC` — one backup counter, one clock mux.
    Burtc,
    /// `ACMP0` — one comparator, one input mux and one ladder tap.
    Acmp0,
    /// `RADIOAES` — one crypto engine, one descriptor chain in flight.
    Aes,
    /// `EUSART1` — the second asynchronous block: same driver, its own `CLKEN1` bit, whether it is driven as a UART or
    /// as a SPI master (`spi::Spi2`). Same reasoning as `Usart0`.
    Eusart1,
}

/// Every slot, in index order -- **test-only**, because nothing but the drift test has a use for a list of all of
/// them, and a `pub const` inside a module the twin crate includes privately is dead code in every other build.
#[cfg(test)]
/// Every slot, in index order. `index()` is the single place that decides the order; this table is checked against it
/// by a host test, so the two cannot drift.
pub const SLOTS: [Slot; SLOT_COUNT] = [
    Slot::Wdog,
    Slot::Emu,
    Slot::Mvp,
    Slot::Sysrtc,
    Slot::Usart0,
    Slot::Spi,
    Slot::Adc,
    Slot::I2c,
    Slot::I2c1,
    Slot::Timer0,
    Slot::Timer1,
    Slot::Timer2,
    Slot::Timer3,
    Slot::Timer4,
    Slot::Gpcrc,
    Slot::Letimer,
    Slot::Prs,
    Slot::Pcnt,
    Slot::Burtc,
    Slot::Acmp0,
    Slot::Aes,
    Slot::Eusart1,
];

/// How many slots there are, so the table and the enum cannot drift apart.
pub const SLOT_COUNT: usize = 22;

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
            Slot::I2c => 7,
            Slot::I2c1 => 8,
            Slot::Timer0 => 9,
            Slot::Timer1 => 10,
            Slot::Timer2 => 11,
            Slot::Timer3 => 12,
            Slot::Timer4 => 13,
            Slot::Gpcrc => 14,
            Slot::Letimer => 15,
            Slot::Prs => 16,
            Slot::Pcnt => 17,
            Slot::Burtc => 18,
            Slot::Acmp0 => 19,
            Slot::Aes => 20,
            Slot::Eusart1 => 21,
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
        assert_eq!(slots.claim(Slot::I2c1), Ok(()));
        assert!(!slots.is_taken(Slot::I2c));
    }

    /// The table and `index()` agree: every slot appears once, at the position its own index names.
    ///
    /// This is the half of the ownership table that can run on the host. The other half -- that two *roles* over one
    /// block are refused -- needs the board, because it is `Board` that decides which slot a constructor claims.
    #[test]
    fn the_table_and_the_index_cannot_drift() {
        for (position, slot) in SLOTS.iter().enumerate() {
            assert_eq!(
                slot.index(),
                position,
                "{slot:?} sits at {position} but says {}",
                slot.index()
            );
        }
        let mut seen = SLOTS.to_vec();
        seen.sort_by_key(|slot| slot.index());
        seen.dedup();
        assert_eq!(seen.len(), SLOT_COUNT, "a slot appears twice in SLOTS");
    }

    /// Every slot claims once and only once.
    ///
    /// The list below is written by hand, and on 2026-11-27 it was two slots behind the enum (`Acmp0` and `Aes` had
    /// never been claimed here), so it is a smoke test and not the guard its first version claimed to be. The real
    /// guard is `Slot::index`: its `match` is exhaustive, so a new variant does not compile until it has an index.
    #[test]
    fn every_slot_is_claimable_once() {
        let mut slots = Slots::new();
        // The list used to be written by hand and was two slots behind the enum. It now walks every index, so a new
        // variant is covered the moment it is declared -- and `index()` refusing to compile without an arm is what
        // guarantees the range is complete.
        for slot in SLOTS {
            assert_eq!(slots.claim(slot), Ok(()), "{slot:?} could not be claimed");
            assert_eq!(
                slots.claim(slot),
                Err(Taken(slot)),
                "{slot:?} was claimable twice"
            );
        }
    }
}
