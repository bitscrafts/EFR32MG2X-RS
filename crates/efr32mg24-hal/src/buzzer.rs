//! Piezo buzzer on TIMER0 compare output — the Expansion Base's `D3` (PC03).
//!
//! ## Why a timer and not a delay loop
//!
//! A passive buzzer needs a square wave at the wanted pitch. Producing it by
//! toggling a pin in a NOP loop ties the CPU up and makes the pitch a function of
//! whatever else the program is doing. TIMER0's compare channel generates it in
//! hardware: the CPU sets a frequency and a duty, and is free.
//!
//! ## The sequence this die demands
//!
//! Measured, and in this order — every deviation is a bus fault or a silent timer:
//!
//! ```text
//! LOCK = 0xCE80            unlock the timer's configuration registers
//! EN   = 0                 configuration is only writable with the timer disabled
//! CFG  = up, prescem01grpaclk, div1
//! CC0_CFG = output compare  (also needs EN = 0)
//! EN   = 1
//! TOP  = period            (this one needs EN = 1)
//! CC0_OC = half period     (so the output is a 50 % square)
//! CC0_CTRL = toggle on compare
//! ```
//!
//! ## Why the frequency is calibrated instead of assumed
//!
//! `HFRCOEM23` does not start in the band the datasheet suggests: measured on this
//! board it runs at roughly 5 MHz at reset and settles near 19 MHz, so a `TOP`
//! computed from 20 MHz gave a tone 5 % flat. The driver therefore counts timer
//! ticks against a known CPU delay once, then computes `TOP` from what it measured.
//! See `docs/04-reference/PAC-CLOCK-AND-TIMER-NOTES.md`.
//!
//! ## Silence
//!
//! Stopping the timer stops `CNT` too, which breaks any timing that waits on it.
//! Silence is `CC0_CTRL.CMOA = none` with the timer still counting — measured
//! failure of the obvious approach.

use efr32mg24_pac as pac;

use crate::pin::Port;
use crate::pwm::Timer0;

/// The piezo on this board is loudest near 2.7 kHz; the base's own demo uses it.
pub const DEFAULT_TONE_HZ: u32 = 2700;

/// A calibrated buzzer: TIMER0 in toggle mode on one pin.
///
/// All of the timer's bring-up, routing and calibration lives in [`Timer0`], because a PWM
/// channel needs exactly the same machinery and two copies of it would eventually disagree.
pub struct Buzzer<'a> {
    timer: Timer0<'a>,
}

impl<'a> Buzzer<'a> {
    /// Take TIMER0 on the given pin and stay silent.
    ///
    /// The pin is a parameter because it is wiring: which pin the buzzer sits on is the board's
    /// knowledge, not the driver's. The clock, the route and the calibration all live in
    /// [`crate::pwm::Timer0`], which is the same machinery a PWM channel uses — a piezo and an
    /// LED are the same waveform at different frequencies, and maintaining two copies of the
    /// bring-up is how one of them ends up wrong.
    pub(crate) fn new(dp: &'a pac::Peripherals, port: Port, pin: u8) -> Option<Self> {
        let mut timer = Timer0::take(dp, port, pin)?;
        timer.square_wave(DEFAULT_TONE_HZ);
        timer.silence();
        Some(Self { timer })
    }

    /// The measured TIMER0 tick rate, in Hz.
    #[must_use]
    pub fn tick_hz(&self) -> u32 {
        self.timer.tick_hz()
    }

    /// The assumed SYSCLK, in Hz.
    #[must_use]
    pub fn sysclk_hz(&self) -> u32 {
        self.timer.sysclk_hz()
    }

    /// Override the assumed SYSCLK, keeping the measured tick/cycle ratio.
    pub fn set_sysclk_hz(&mut self, hz: u32) {
        self.timer.set_sysclk_hz(hz);
    }

    /// Set the pitch. Values between roughly 200 Hz and 8 kHz are usable.
    ///
    /// Returns the TIMER0 period it used, which is the number worth quoting in a report.
    pub fn set_tone(&mut self, hz: u32) -> u32 {
        self.timer.square_wave(hz)
    }

    /// Sound the tone.
    pub fn sound(&mut self) {
        self.timer.sound();
    }

    /// Silence without stopping the counter, so timings that wait on `CNT` keep working.
    pub fn silence(&mut self) {
        self.timer.silence();
    }

    /// The configured period in timer ticks, for tests and reports.
    #[must_use]
    pub fn period_ticks(&self) -> u32 {
        self.timer.period_ticks()
    }
}
