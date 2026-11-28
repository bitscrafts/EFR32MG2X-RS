//! Delay implementation using SysTick
//!
//! This module provides blocking delay functions using the ARM Cortex-M SysTick timer.
//!
//! # Examples
//!
//! ```no_run
//! use efr32mg2x_hal::{clock::Clocks, delay::Delay, pac};
//! use embedded_hal::delay::DelayNs;
//!
//! let cp = cortex_m::Peripherals::take().unwrap();
//! let dp = pac::Peripherals::take().unwrap();
//!
//! let clocks = Clocks::new(dp.CMU_NS, Default::default()).freeze();
//! let mut delay = Delay::new(cp.SYST, &clocks);
//!
//! // Delay for 1 second
//! delay.delay_ms(1000);
//!
//! // Delay for 100 microseconds
//! delay.delay_us(100);
//! ```

use core::sync::atomic::{AtomicBool, Ordering};
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;
use embedded_hal::delay::DelayNs;

use crate::clock::{FrozenClocks, Hertz};

/// Delay provider using SysTick
///
/// Provides blocking delays using the Cortex-M SysTick timer.
pub struct Delay {
    syst: SYST,
    sysclk: Hertz,
}

/// Set once, by whoever builds the first `Delay`. A second `with_sysclk` is a verified no-op on the hardware side: it
/// does **not** reprogram `LOAD`, `VAL` or `CTRL`.
static CONFIGURED: AtomicBool = AtomicBool::new(false);

/// The 24-bit reload the free-running tick uses: one millisecond, at the clock the caller states.
fn ticks_per_ms(sysclk_hz: u32) -> u32 {
    (sysclk_hz / 1000).clamp(1, 0x00FF_FFFF)
}

/// Configure the SysTick as a **free-running 1 ms tick**, once in the life of the program.
///
/// Why once: until 2026-11-28 every delay call reprogrammed `LOAD`, cleared `VAL`, enabled the counter and disabled it
/// again. Two `Delay` values in the same interval -- two threads, or a handler that delays -- reset each other's
/// countdown and switched the counter off under each other, so one of them returned early. A delay that returns early
/// looks exactly like a delay that was right. (Outside review, f6.)
///
/// After this, a wait only *observes* the tick; nothing reprograms it. Two handles can then only make each other wait
/// **longer** -- one may consume the wrap flag the other was waiting for -- and never shorter, which is the safe
/// direction and the reason the hazard is closed rather than forbidden.
fn configure_once(syst: &mut SYST, sysclk_hz: u32) {
    if CONFIGURED.swap(true, Ordering::SeqCst) {
        return;
    }
    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(ticks_per_ms(sysclk_hz));
    syst.clear_current();
    syst.enable_counter();
}

impl Delay {
    /// Create a new delay provider
    ///
    /// # Arguments
    ///
    /// * `syst` - The SysTick peripheral
    /// * `clocks` - The frozen clock configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use efr32mg2x_hal::{clock::Clocks, delay::Delay, pac};
    /// # let cp = cortex_m::Peripherals::take().unwrap();
    /// # let dp = pac::Peripherals::take().unwrap();
    /// # let clocks = Clocks::new(dp.CMU_NS, Default::default()).freeze();
    /// let delay = Delay::new(cp.SYST, &clocks);
    /// ```
    pub fn new(mut syst: SYST, clocks: &FrozenClocks) -> Self {
        syst.set_clock_source(SystClkSource::Core);

        Self {
            syst,
            sysclk: clocks.sysclk(),
        }
    }

    /// Create a delay provider from a stated clock frequency.
    ///
    /// For callers that own the board but not its `FrozenClocks` — the BSP facade is one:
    /// the HAL's `Board` keeps no clock handle, so it cannot hand one out.
    pub fn with_sysclk(mut syst: SYST, sysclk_hz: u32) -> Self {
        configure_once(&mut syst, sysclk_hz);

        // KNOWN DEFECT, measured 2026-11-12: delays come out about 2.3x shorter than requested.
        // `emlib_clock` compares `delay_ms(10)` against emlib's two clock readings and the DWT; it
        // reports 4.35 ms where 10 ms was asked for. An attempt to fix this by measuring SysTick's
        // rate against the cycle counter made it worse (3.0x), so it was reverted rather than shipped:
        // a wrong mechanism that looks like a fix is worse than a documented defect.
        //
        // The next step is the experiment written in docs/04-reference/DELAY-CLOCK-DEFECT.md: time a
        // SysTick reload of a known tick count directly against the cycle counter, and only then
        // decide what the correct conversion is.
        Self {
            syst,
            sysclk: Hertz(sysclk_hz),
        }
    }

    /// Wait for `ticks` core cycles on the free-running 1 ms counter, without touching its configuration.
    fn spin_ticks(&mut self, ticks: u32) {
        if ticks == 0 {
            return;
        }
        // Reading the counter does not touch its configuration, which is the whole point: this wait observes the tick
        // another handle configured.
        let value = SYST::get_current;
        let start = value() & 0x00FF_FFFF;
        while (start.wrapping_sub(value()) & 0x00FF_FFFF) < ticks {}
    }

    /// Release the SysTick peripheral
    ///
    /// This consumes the Delay and returns the SysTick peripheral
    /// so it can be used for other purposes.
    pub fn free(self) -> SYST {
        self.syst
    }
}

impl DelayNs for Delay {
    fn delay_ns(&mut self, ns: u32) {
        // Convert nanoseconds to SysTick ticks
        // ticks = (ns * sysclk_hz) / 1_000_000_000

        let sysclk_hz = self.sysclk.0;

        // For very short delays, just use a NOP loop
        if ns < 1000 {
            // At 39 MHz, each instruction is ~25ns
            // This is approximate but good enough for very short delays
            let cycles = (ns * (sysclk_hz / 1_000_000)) / 1000;
            cortex_m::asm::delay(cycles);
            return;
        }

        // One microsecond up to a millisecond: spin on the running tick, without reprogramming it.
        // Calculate ticks, being careful with overflow
        // ticks = (ns / 1_000_000_000) * sysclk_hz
        //       = (ns * sysclk_hz) / 1_000_000_000

        // To avoid overflow, we'll process in chunks if needed
        let ticks = if ns < 1_000_000 {
            // For delays < 1ms, calculate directly
            ((ns as u64) * (sysclk_hz as u64) / 1_000_000_000) as u32
        } else {
            // For delays >= 1ms, use microsecond precision
            self.delay_us(ns / 1000);
            return;
        };

        if ticks == 0 {
            return;
        }

        // One to a thousand microseconds: spin on the free-running tick instead of reprogramming it. The window is
        // under one millisecond, so the 24-bit counter cannot wrap twice and the wrap-around comparison is exact.
        self.spin_ticks(ticks);
    }

    fn delay_us(&mut self, us: u32) {
        let sysclk_hz = self.sysclk.0;
        if us >= 1000 {
            self.delay_ms(us / 1000);
            let rest = us % 1000;
            if rest > 0 {
                self.spin_ticks(((rest as u64) * (sysclk_hz as u64) / 1_000_000) as u32);
            }
            return;
        }
        self.spin_ticks(((us as u64) * (sysclk_hz as u64) / 1_000_000) as u32);
    }

    fn delay_ms(&mut self, ms: u32) {
        // Count wraps of a tick that is already running. Nothing here reprograms the counter, so a second `Delay`
        // waiting at the same time can only make this one wait longer -- never shorter.
        for _ in 0..ms {
            while !self.syst.has_wrapped() {}
        }
    }
}
