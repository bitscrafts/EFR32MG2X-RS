//! Entering EM2 -- and leaving it.
//!
//! ## Why this is an API and not a `wfi` in an example
//!
//! Entering EM2 is not sleeping: it turns the high-frequency clock off and leaves the part dependent on an interrupt
//! that is already armed. A loose `wfi` with `SLEEPDEEP` set and no wake-up source leaves the device asleep until
//! someone touches its reset -- and the symptom, seen from outside, is a board that died.
//!
//! On this series the `EMU` has `CTRL.EM2DBGEN` and does **not** have an EM2 lock: what prevents entry is the state of
//! the rest of the system, not a bit. The two this module deals with:
//!
//! - **the debugger**: with `EM2DBGEN = 0` the debug connection dies in EM2 and does not come back with the board
//!   running -- [`Sleep::keep_debugger`] sets it before sleeping;
//! - **`SLEEPDEEP`**: it is left set during the `wfi` and cleared afterwards, so that the next `wfi` someone writes by
//!   hand does not enter EM2 without knowing it.
//!
//! ## What this module does not do
//!
//! It does not arm wake-up sources. The caller has to have the interrupt enabled in the peripheral **and** in the
//! `NVIC` before entering, or the `wfi` does not return. That is deliberate: choosing the wake-up source is the
//! product's decision, and a `sleep_ms` that decided for us would be the same class of assumption the `19_000_000`
//! was hiding in the timer driver.

use crate::pac;

/// Entering EM2 and leaving it.
pub struct Sleep<'a> {
    emu: &'a pac::emu_ns::RegisterBlock,
}

impl<'a> Sleep<'a> {
    pub(crate) fn new(emu: &'a pac::emu_ns::RegisterBlock) -> Self {
        Self { emu }
    }

    /// Keeps the debug connection alive while the core is in EM2.
    ///
    /// Without it `probe-rs` loses the target on the first sleep and what you observe is a silent board -- which is
    /// indistinguishable from a broken one. Keeping it costs microamps and buys the ability to look.
    pub fn keep_debugger(&self) {
        self.emu.ctrl().modify(|_, w| w.em2dbgen().set_bit());
    }

    /// Sleeps in EM2 until an armed interrupt wakes the core.
    ///
    /// It does not return without an interrupt: the caller has to have one enabled in the peripheral and in the
    /// `NVIC`.
    pub fn enter(&self) {
        // `cortex_m::Peripherals::steal` is what `pwm` already does for the DWT: the `SCB` does not come in this chip's
        // PAC, and threading a `&mut SCB` through the whole call chain to set one bit does not make it safer.
        // SAFETY: `Board` took `cortex_m::Peripherals` once and keeps it; the `SCB` is not part of this chip's PAC, so
        // it is borrowed here and only `SCR.SLEEPDEEP` is touched -- the one bit this crate writes on that block.
        let mut scb = unsafe { cortex_m::Peripherals::steal() }.SCB;
        scb.set_sleepdeep();
        cortex_m::asm::wfi();
        // Cleared afterwards: an accidental entry into EM2 later would be a silent defect.
        scb.clear_sleepdeep();
    }
}
