//! Delay implementation using SysTick
//!
//! This module provides blocking delay functions using the ARM Cortex-M SysTick timer.
//!
//! # Examples
//!
//! ```no_run
//! use efr32mg24_hal::{clock::Clocks, delay::Delay, pac};
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
    /// # use efr32mg24_hal::{clock::Clocks, delay::Delay, pac};
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

        // For longer delays, use SysTick
        // SysTick is a 24-bit down counter
        const MAX_RVR: u32 = 0x00FF_FFFF;

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

        // If the delay fits in one SysTick cycle
        if ticks <= MAX_RVR {
            self.syst.set_reload(ticks);
            self.syst.clear_current();
            self.syst.enable_counter();

            while !self.syst.has_wrapped() {}

            self.syst.disable_counter();
        } else {
            // Split into multiple cycles
            let mut remaining = ticks;

            while remaining > 0 {
                let current = remaining.min(MAX_RVR);

                self.syst.set_reload(current);
                self.syst.clear_current();
                self.syst.enable_counter();

                while !self.syst.has_wrapped() {}

                self.syst.disable_counter();

                remaining -= current;
            }
        }
    }

    fn delay_us(&mut self, us: u32) {
        // Convert microseconds to ticks
        // ticks = (us * sysclk_hz) / 1_000_000

        let sysclk_hz = self.sysclk.0;
        const MAX_RVR: u32 = 0x00FF_FFFF;

        // Calculate ticks
        let ticks = ((us as u64) * (sysclk_hz as u64) / 1_000_000) as u32;

        if ticks == 0 {
            return;
        }

        if ticks <= MAX_RVR {
            self.syst.set_reload(ticks);
            self.syst.clear_current();
            self.syst.enable_counter();

            while !self.syst.has_wrapped() {}

            self.syst.disable_counter();
        } else {
            // Split into multiple cycles
            let mut remaining = ticks;

            while remaining > 0 {
                let current = remaining.min(MAX_RVR);

                self.syst.set_reload(current);
                self.syst.clear_current();
                self.syst.enable_counter();

                while !self.syst.has_wrapped() {}

                self.syst.disable_counter();

                remaining -= current;
            }
        }
    }

    fn delay_ms(&mut self, ms: u32) {
        // For millisecond delays, call delay_us in chunks to avoid overflow
        for _ in 0..ms {
            self.delay_us(1000);
        }
    }
}
