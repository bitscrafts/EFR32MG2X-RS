//! `LETIMER0`: the timer that keeps counting while the core is asleep.
//!
//! Every other timer in this crate needs the bus clock, which means the core is awake or the count stops. `LETIMER0` runs
//! from the low-frequency domain and keeps counting in EM2 and EM3, so it is the thing a low-power design wakes on: a
//! compare match every second, every minute, or once an hour, with the core asleep in between.
//!
//! # What this driver encodes, and why
//!
//! Eight facts were measured on the part (2026-11-26, records in `docs/04-reference/LETIMER-TOP-2026-11-26.md` and
//! `docs/04-reference/LETIMER-COMPARE-2026-11-26.md`) and each one is a line of the code below:
//!
//! 1. **`EN` is the gate.** `CLKEN0` alone leaves the module answering on the bus and never counting.
//! 2. **`CTRL.CNTTOPEN` is the switch that makes `TOP` mean anything.** At its reset value the top is the whole 24-bit
//!    counter (`0xFFFFFF`, not the 65535 the manual's bit table claims) and `TOP` is ignored; set, the period is
//!    `TOP`+1 ticks and `TOP` is the top. This driver sets it, always, before returning.
//! 3. **The counter is 24 bits.** It is exposed here as [`Letimer::counter`] returning `u32`; the `u16` this crate used
//!    before truncated it and made a real top of `0xFFFFFF` look like a top of 65535 -- the whole defect that hid the
//!    switch in the first place.
//! 4. **A compare at or above `TOP` never fires.** Both `set_top` and `set_compare` refuse that combination instead of
//!    programming a flag that cannot be reached.
//! 5. **`SYNCBUSY` is never polled.** Reading it on this peripheral stalls the bus -- measured three times, with the
//!    branch on LFXO, on LFRCO, and before a branch was selected at all. The registers still cross into the low-frequency
//!    domain, so the caller settles with its own clock: [`SETTLE_US`] is how long that takes.
//! 6. **Writing to `IF` sets a flag; clearing goes through `IF + 0x2000`.** A write of `COMP0|COMP1|UF` to `IF` left
//!    `0x7` behind, and the run's flags never cleared until the write went to the alias. [`Letimer::clear_compares`] and
//!    [`Letimer::clear_flags`] use the alias.
//! 7. **The counter counts down, so `COMP1` fires after `COMP0`** when its value is lower. Measured: `COMP0` = 500 fired
//!    501 ticks after the reload and `COMP1` = 250 fired 250 ticks after that. And a compare flag set while the counter
//!    is still below the compare value cannot be cleared -- it is the comparison, not a latch -- so a caller that wants
//!    to see a clear takes it after the wrap.
//! 8. **`TOP` read back is not trustworthy while the timer is stopped.** Two of four runs returned `0` for a `TOP` that
//!    had been written as 1000 and settled; the same register read once the counter was running returned 1000 every
//!    time. That is why the compare validation uses [`Letimer::programmed_top`] and the register is reported separately.
//!
//! # What is deliberately absent
//!
//! The prescaler stays at its reset value (divide by one) and only the free-running count and the two compare channels
//! are exposed. Repetition (`REP0`/`REP1`), the PWM outputs and the PRS trigger are in the PAC and are not driven here
//! yet: none of them has an instrument in this crate, and a driver written without one is a claim.

use efr32mg2x_pac as pac;

/// How many ticks a second holds with the prescaler at its reset value and LFRCO (32.768 kHz) as the source.
pub const TICKS_PER_SECOND: u32 = 32_768;

/// The counter's width: `CNT` is bits 23:0, and that is the number the board reported at reset (`16777215`), not 65535.
const COUNTER_MASK: u32 = 0x00FF_FFFF;

/// How long a caller must let pass after a write that crosses into the low-frequency domain: two clock cycles at
/// 32.768 kHz, rounded up, and nothing more. The driver cannot wait for itself -- polling `SYNCBUSY` stalls this
/// peripheral -- so this is the contract the caller keeps.
pub const SETTLE_US: u32 = 62;

/// Which compare channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// `COMP0`
    Ch0,
    /// `COMP1`
    Ch1,
}

/// Why a `LETIMER0` operation was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// A compare at or above `TOP` is unreachable: the counter never reaches it, so the flag never sets.
    CompareAtOrAboveTop {
        /// The compare that was asked for.
        compare: u16,
        /// The top in force when it was asked for.
        top: u16,
    },
    /// A `TOP` at or below a compare already programmed would orphan that compare exactly as above.
    TopBelowCompare {
        /// The top that was asked for.
        top: u16,
        /// The compare already in the register.
        compare: u16,
    },
    /// A `TOP` of zero is a one-tick period, which no caller of this API can use and every caller would misread as
    /// "unset".
    TopTooSmall,
}

/// `IF` bits.
const IF_COMP0: u32 = 1 << 0;
const IF_COMP1: u32 = 1 << 1;
/// Every flag the PAC documents: `COMP0`, `COMP1`, `UF`, `REP0`, `REP1`.
const IF_ALL: u32 = 0x1F;

/// `IF`, relative to the peripheral base (the PAC's own offset).
const IF_OFFSET: usize = 0x034;

/// Where the write-1-to-clear alias of every register in this family lives: base + 0x2000 + the register's offset.
const CLEAR_ALIAS: usize = 0x2000;

/// `LETIMER0`, owned by one caller.
pub struct Letimer {
    regs: pac::Letimer0Ns,
    /// The top this driver last programmed. `TOP` crosses into the low-frequency domain, so reading it back immediately
    /// after a write can return the old value -- and validating a compare against that stale number refuses a compare
    /// that is perfectly reachable. The driver therefore keeps what it wrote.
    top_ticks: u16,
}

/// What the clock tree says about this peripheral, read back rather than assumed.
///
/// Bit 0 is `CMU_CLKEN0.LETIMER0`; bits 8..11 are `CMU_EM23GRPACLKCTRL.CLKSEL`. An instrument that records this next to
/// the counter can tell a branch that was never selected from a module that was enabled and did not run -- two different
/// defects that look the same from the counter alone.
#[must_use]
pub fn clock_readback() -> u32 {
    // SAFETY: two CMU registers, read only. `Board` owns the CMU for the life of the program and a read changes nothing.
    let cmu = unsafe { pac::CmuS::steal() };
    let clken = u32::from(cmu.clken0().read().letimer0().bit_is_set());
    let sel = cmu.em23grpaclkctrl().read().clksel().bits();
    clken | ((u32::from(sel) & 0xF) << 8)
}

impl Letimer {
    /// Take `LETIMER0`, unlock its configuration registers, enable it and make `TOP` the top, left stopped.
    ///
    /// The three writes are in a measured order: `LOCK` first (the configuration registers are locked out of reset and a
    /// locked write is silently ignored), then `EN` (the module answers on the bus while disabled and never counts), then
    /// `CTRL.CNTTOPEN` (without it `TOP` is ignored and the counter runs 0xFFFFFF ticks per period).
    ///
    /// # Safety
    ///
    /// The caller must own `LETIMER0` for the life of this value. Two drivers writing `COMP0` interleave into one period
    /// that neither of them asked for.
    pub(crate) unsafe fn new(regs: pac::Letimer0Ns) -> Self {
        regs.lock().write(|w| w.letimerlockkey().unlock());
        regs.en().write(|w| w.en().set_bit());
        // `REPMODE = FREE`: a repeat counter would stop the compares after a programmed number of matches, and this
        // driver has no way to say how many.
        regs.ctrl()
            .modify(|_, w| w.cnttopen().bit(true).repmode().free());
        // Read after enabling, and twice: the first read asks for the crossing, the second returns a settled value.
        let _ = regs.top().read().top().bits();
        let top_ticks = regs.top().read().top().bits() as u16;
        Self { regs, top_ticks }
    }

    /// Set the top, which is where the counter reloads and how long one period lasts: `TOP`+1 ticks.
    ///
    /// Refused when the new top is at or below a compare already programmed, because that compare would become
    /// unreachable -- `IF` would then only ever carry the underflow flag.
    pub fn set_top(&mut self, ticks: u16) -> Result<(), Error> {
        if ticks == 0 {
            return Err(Error::TopTooSmall);
        }
        let compare = self.compare(Channel::Ch0).max(self.compare(Channel::Ch1));
        if compare >= ticks {
            return Err(Error::TopBelowCompare {
                top: ticks,
                compare,
            });
        }
        // SAFETY: `TOP` is 16 bits and `ticks` is a `u16`, so no value can exceed the field.
        self.regs
            .top()
            .write(|w| unsafe { w.top().bits(u32::from(ticks)) });
        self.top_ticks = ticks;
        Ok(())
    }

    /// The top in the register, read back.
    ///
    /// Read twice and keep the second, for the same reason [`Letimer::counter`] does: `TOP` is a synchronised register and
    /// the first read requests the crossing. Even so, read it at least [`SETTLE_US`] after the write if the value has to
    /// be believed -- the crossing is why the driver validates compares against [`Letimer::programmed_top`] rather than
    /// against this.
    ///
    /// Measured 2026-11-26: read while the counter was running this returned 1000 every time; read before `start` it
    /// returned 0 in two runs out of four, after the same settle. A `0` from this method on a stopped timer is therefore
    /// not evidence that the write did not land.
    #[must_use]
    pub fn top(&self) -> u16 {
        let _ = self.regs.top().read().top().bits();
        self.regs.top().read().top().bits() as u16
    }

    /// The top this driver programmed, as the driver holds it.
    ///
    /// This is not a register read: it is what the caller asked for and the driver wrote, which is the number a compare
    /// has to stay below. [`Letimer::top`] is the register.
    #[must_use]
    pub fn programmed_top(&self) -> u16 {
        self.top_ticks
    }

    /// Set a compare value, in ticks from the reload.
    ///
    /// The counter loads `TOP` and counts **down**, so a compare matches that many ticks after the period starts. Refused
    /// at or above `TOP`: the counter never passes that value, and the flag would never set.
    ///
    /// Call it while the timer is stopped. Nothing in the hardware stops a write while it runs, and the result of one is
    /// a period whose first match is neither the old value nor the new.
    pub fn set_compare(&mut self, channel: Channel, ticks: u16) -> Result<(), Error> {
        let top = self.programmed_top();
        if ticks >= top {
            return Err(Error::CompareAtOrAboveTop {
                compare: ticks,
                top,
            });
        }
        match channel {
            Channel::Ch0 => {
                // SAFETY: `COMP0` is a 16-bit register and the field is written from a `u16`, so no value can exceed it.
                self.regs
                    .comp0()
                    .write(|w| unsafe { w.comp0().bits(u32::from(ticks)) });
            }
            Channel::Ch1 => {
                // SAFETY: `COMP1` is a 16-bit register and the field is written from a `u16`, so no value can exceed it.
                self.regs
                    .comp1()
                    .write(|w| unsafe { w.comp1().bits(u32::from(ticks)) });
            }
        }
        Ok(())
    }

    /// The value a compare channel currently holds.
    #[must_use]
    pub fn compare(&self, channel: Channel) -> u16 {
        match channel {
            Channel::Ch0 => self.regs.comp0().read().comp0().bits() as u16,
            Channel::Ch1 => self.regs.comp1().read().comp1().bits() as u16,
        }
    }

    /// Start counting from `TOP` downwards.
    ///
    /// Let at least [`SETTLE_US`] pass after the last configuration write and before this one, or the start can cross
    /// ahead of the configuration it was meant to use.
    pub fn start(&mut self) {
        self.regs.cmd().write(|w| w.start().set_bit());
    }

    /// Stop counting. The counter keeps its value; [`Letimer::clear`] puts it back to the top.
    pub fn stop(&mut self) {
        self.regs.cmd().write(|w| w.stop().set_bit());
    }

    /// Reload the counter from `TOP`.
    pub fn clear(&mut self) {
        self.regs.cmd().write(|w| w.clear().set_bit());
    }

    /// The counter's value, 24 bits, read through the low-frequency boundary.
    ///
    /// Read twice and keep the second: the first read requests the crossing, the second returns a settled value. The pair
    /// is not claimed to be a synchronisation primitive -- `SYNCBUSY` cannot be polled on this part -- and the instrument
    /// that uses it records the counter next to an independent clock.
    pub fn counter(&mut self) -> u32 {
        let _ = self.regs.cnt().read().cnt().bits();
        self.regs.cnt().read().cnt().bits() & COUNTER_MASK
    }

    /// Every flag in `IF`, as the register returns it: `COMP0` bit 0, `COMP1` bit 1, `UF` bit 2, `REP0` bit 3, `REP1`
    /// bit 4.
    ///
    /// The whole word rather than one bit at a time, because "the compare did not fire" and "the flag I read was not the
    /// one that fired" look identical through a single-bit accessor.
    #[must_use]
    pub fn flags(&self) -> u32 {
        self.regs.if_().read().bits()
    }

    /// Whether a compare channel has matched since it was last cleared.
    #[must_use]
    pub fn compare_fired(&self, channel: Channel) -> bool {
        let bit = match channel {
            Channel::Ch0 => IF_COMP0,
            Channel::Ch1 => IF_COMP1,
        };
        self.regs.if_().read().bits() & bit != 0
    }

    /// Whether the counter has wrapped since the flags were last cleared.
    #[must_use]
    pub fn wrapped(&self) -> bool {
        // `IF.UF`, bit 2. The probe read `0x4` with the top in force and `0x0` at the reset top, which is the same
        // observation from the other side.
        self.regs.if_().read().bits() & (1 << 2) != 0
    }

    /// Clear the compare flags.
    ///
    /// Clearing is what makes a match a repeated event: a flag left set reads as "it fired" for ever, and an instrument
    /// that never clears it cannot tell one match from ten.
    pub fn clear_compares(&mut self) {
        self.clear_if(IF_COMP0 | IF_COMP1);
    }

    /// Clear every flag, including the underflow that a wrap sets.
    pub fn clear_flags(&mut self) {
        self.clear_if(IF_ALL);
    }

    /// Clear `IF` bits through the write-1-to-clear alias.
    ///
    /// **Writing one to `IF` does not clear a flag on this family: it sets one.** Measured 2026-11-26: after a run that
    /// left `IF = 0x1`, a write of `COMP0|COMP1|UF` to `IF` read back `0x7` -- the flags it was supposed to clear had been
    /// added instead. The clear lives at `IF + 0x2000`, the alias window every peripheral in this family has (the I2C
    /// driver learned the same lesson on 2026-11-24: `docs/04-reference/I2C0-READ-DEFECT.md`).
    fn clear_if(&mut self, bits: u32) {
        let base = &*self.regs as *const pac::letimer0_ns::RegisterBlock as usize;
        // SAFETY: `base + IF + 0x2000` is the clear alias of a register the caller owns through this value; the write is
        // 32 bits wide, which is the register's own width. It has no effect on any other register.
        unsafe {
            core::ptr::write_volatile((base + IF_OFFSET + CLEAR_ALIAS) as *mut u32, bits);
        }
    }
}
