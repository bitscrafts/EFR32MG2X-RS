//! Real-Time Clock (SYSRTC0) Module
//!
//! This module provides a Hardware Abstraction Layer (HAL) for the EFR32MG24's
//! System Real-Time Clock (SYSRTC0), a 32-bit counter for timekeeping and alarms.
//!
//! # Features
//!
//! - **32-bit Counter**: Up to 136 years at 32.768 kHz
//! - **Low Power**: Runs in EM0-EM3 energy modes
//! - **Compare Match (Alarm)**: Single alarm with interrupt capability
//! - **LFXO/LFRCO Clock**: High-accuracy crystal or RC oscillator
//! - **Simple API**: Counter-based timekeeping (not full calendar)
//!
//! # Current Implementation (C4-RTC Stage 1)
//!
//! - ✅ Counter read/write
//! - ✅ Single compare match (alarm)
//! - ✅ Interrupt flag checking
//! - ✅ LFXO clock source
//! - ✅ Helper functions (counter ↔ seconds)
//!
//! # Not Implemented (Future)
//!
//! - ⏳ Full calendar mode (date/time structures)
//! - ⏳ Multiple alarms (CMP1)
//! - ⏳ Capture mode
//! - ⏳ Output actions
//!
//! # Usage
//!
//! ```rust,no_run
//! use efr32mg24_hal::{rtc::Rtc, pac};
//!
//! let dp = pac::Peripherals::take().unwrap();
//!
//! // Initialize RTC with LFXO (requires LFXO to be enabled in CMU)
//! let mut rtc = Rtc::new(dp.sysrtc0_ns);
//!
//! // Set current time to 0 (reset counter)
//! rtc.set_counter(0);
//!
//! // Set alarm for 10 seconds from now
//! let current = rtc.counter();
//! let alarm_time = current + rtc.seconds_to_ticks(10);
//! rtc.set_alarm(alarm_time).expect("Failed to set alarm");
//!
//! // Wait for alarm
//! while !rtc.alarm_flag() {
//!     // Could enter sleep mode here
//! }
//!
//! rtc.clear_alarm_flag();
//! ```
//!
//! # Clock Source
//!
//! The RTC requires a low-frequency clock (LFCLK) to be configured in the CMU.
//! Typically this is LFXO (32.768 kHz crystal) for accuracy.
//!
//! ```rust,no_run
//! // Configure LFXO before initializing RTC
//! let (clocks, cmu) = Clocks::new(
//!     dp.cmu_s,
//!     ClockConfig {
//!         hfxo: Some(HfxoConfig::new(39_000_000)),
//!         lfxo: Some(LfxoConfig::default()),  // Enable LFXO for RTC
//!     },
//! ).expect("Clock config failed");
//! ```

mod types;
pub use types::*;

use crate::pac;

/// RTC (System Real-Time Clock)
///
/// Owns the SYSRTC0 peripheral and provides timekeeping functionality.
pub struct Rtc {
    _sysrtc: pac::Sysrtc0Ns,
}

impl Rtc {
    /// Creates a new RTC instance
    ///
    /// Takes ownership of the SYSRTC0 peripheral and initializes it with LFXO clock.
    ///
    /// **Note**: LFXO must be enabled in CMU before calling this function.
    ///
    /// # Arguments
    ///
    /// * `sysrtc` - The SYSRTC0_NS peripheral from PAC
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let dp = pac::Peripherals::take().unwrap();
    /// let rtc = Rtc::new(dp.sysrtc0_ns);
    /// ```
    pub fn new(sysrtc: pac::Sysrtc0Ns) -> Self {
        // Enable SYSRTC0 clock in CMU
        // SAFETY: We have exclusive access to CMU via critical section.
        // Setting SYSRTC0 bit enables the RTC clock.
        critical_section::with(|_cs| {
            let cmu = unsafe { &(*pac::CmuS::ptr()) };
            cmu.clken0().modify(|_, w| w.sysrtc0().set_bit());
        });

        // Enable SYSRTC0 peripheral
        // SAFETY: We own the SYSRTC peripheral. EN register enables the counter.
        unsafe {
            let sysrtc_ptr = pac::Sysrtc0Ns::ptr();
            (*sysrtc_ptr).en().write(|w| w.en().set_bit());
        }

        // Start the counter
        // SAFETY: We own the SYSRTC peripheral. CMD.START starts the counter.
        unsafe {
            let sysrtc_ptr = pac::Sysrtc0Ns::ptr();
            (*sysrtc_ptr).cmd().write(|w| w.start().set_bit());
        }

        Self { _sysrtc: sysrtc }
    }

    /// Reads the current counter value
    ///
    /// Returns the 32-bit counter value in ticks (32.768 kHz with LFXO).
    ///
    /// # Returns
    ///
    /// Current counter value in ticks
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let ticks = rtc.counter();
    /// let seconds = rtc.ticks_to_seconds(ticks);
    /// ```
    pub fn counter(&self) -> u32 {
        // SAFETY: Reading CNT register is safe
        let sysrtc = unsafe { &(*pac::Sysrtc0Ns::ptr()) };
        sysrtc.cnt().read().bits()
    }

    /// Sets the counter value
    ///
    /// Writes a new value to the 32-bit counter.
    ///
    /// # Arguments
    ///
    /// * `value` - New counter value in ticks
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// // Reset counter to 0
    /// rtc.set_counter(0);
    ///
    /// // Set to specific time (e.g., 1 hour = 3600 seconds)
    /// rtc.set_counter(rtc.seconds_to_ticks(3600));
    /// ```
    pub fn set_counter(&mut self, value: u32) {
        // SAFETY: Writing to CNT register is safe. We own the peripheral.
        critical_section::with(|_cs| {
            let sysrtc = unsafe { &(*pac::Sysrtc0Ns::ptr()) };
            sysrtc.cnt().write(|w| unsafe { w.bits(value) });
        });
    }

    /// Sets the alarm (compare match 0) value
    ///
    /// Configures the compare match 0 register to trigger an interrupt flag
    /// when the counter reaches the specified value.
    ///
    /// # Arguments
    ///
    /// * `value` - Counter value at which to trigger alarm
    ///
    /// # Errors
    ///
    /// Returns `Error::Overflow` if value exceeds 32-bit range (shouldn't happen with u32).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// // Set alarm for 60 seconds from now
    /// let current = rtc.counter();
    /// let alarm_time = current + rtc.seconds_to_ticks(60);
    /// rtc.set_alarm(alarm_time).expect("Failed to set alarm");
    /// ```
    pub fn set_alarm(&mut self, value: u32) -> Result<(), Error> {
        // SAFETY: Writing to compare register is safe. We own the peripheral.
        critical_section::with(|_cs| {
            let sysrtc = unsafe { &(*pac::Sysrtc0Ns::ptr()) };

            // Set compare 0 value
            sysrtc.grp0_cmp0value().write(|w| unsafe { w.bits(value) });

            // Enable compare 0
            sysrtc.grp0_ctrl().modify(|_, w| w.cmp0en().set_bit());
        });

        Ok(())
    }

    /// Checks if the alarm flag is set
    ///
    /// Returns true if the counter has reached the compare match 0 value.
    ///
    /// # Returns
    ///
    /// `true` if alarm triggered, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// if rtc.alarm_flag() {
    ///     // Alarm triggered!
    ///     rtc.clear_alarm_flag();
    /// }
    /// ```
    pub fn alarm_flag(&self) -> bool {
        // SAFETY: Reading interrupt flag register is safe
        let sysrtc = unsafe { &(*pac::Sysrtc0Ns::ptr()) };
        sysrtc.grp0_if().read().cmp0().bit_is_set()
    }

    /// Clears the alarm flag
    ///
    /// Clears the compare match 0 interrupt flag.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// if rtc.alarm_flag() {
    ///     rtc.clear_alarm_flag();
    ///     // Handle alarm event
    /// }
    /// ```
    pub fn clear_alarm_flag(&mut self) {
        // SAFETY: Writing to interrupt flag clear register is safe
        let sysrtc = unsafe { &(*pac::Sysrtc0Ns::ptr()) };
        sysrtc.grp0_if().write(|w| w.cmp0().set_bit());
    }

    /// Disables the alarm
    ///
    /// Disables compare match 0, preventing alarm triggers.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// rtc.disable_alarm();
    /// ```
    pub fn disable_alarm(&mut self) {
        // SAFETY: Writing to control register is safe
        critical_section::with(|_cs| {
            let sysrtc = unsafe { &(*pac::Sysrtc0Ns::ptr()) };
            sysrtc.grp0_ctrl().modify(|_, w| w.cmp0en().clear_bit());
        });
    }

    /// Converts ticks to seconds
    ///
    /// Converts RTC counter ticks to seconds, assuming 32.768 kHz LFXO.
    ///
    /// # Arguments
    ///
    /// * `ticks` - Counter value in ticks
    ///
    /// # Returns
    ///
    /// Approximate seconds (rounded down)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let ticks = rtc.counter();
    /// let seconds = rtc.ticks_to_seconds(ticks);
    /// println!("Uptime: {} seconds", seconds);
    /// ```
    pub fn ticks_to_seconds(&self, ticks: u32) -> u32 {
        // 32768 ticks per second with LFXO
        ticks / 32768
    }

    /// Converts seconds to ticks
    ///
    /// Converts seconds to RTC counter ticks, assuming 32.768 kHz LFXO.
    ///
    /// # Arguments
    ///
    /// * `seconds` - Time in seconds
    ///
    /// # Returns
    ///
    /// Corresponding tick count
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// // Set alarm for 5 minutes (300 seconds)
    /// let alarm_ticks = rtc.seconds_to_ticks(300);
    /// rtc.set_alarm(alarm_ticks).unwrap();
    /// ```
    pub fn seconds_to_ticks(&self, seconds: u32) -> u32 {
        // 32768 ticks per second with LFXO
        seconds.saturating_mul(32768)
    }

    /// Converts ticks to milliseconds
    ///
    /// Converts RTC counter ticks to milliseconds.
    ///
    /// # Arguments
    ///
    /// * `ticks` - Counter value in ticks
    ///
    /// # Returns
    ///
    /// Approximate milliseconds (rounded down)
    pub fn ticks_to_millis(&self, ticks: u32) -> u32 {
        // (ticks * 1000) / 32768 = (ticks * 125) / 4096
        (ticks as u64 * 125 / 4096) as u32
    }

    /// Converts milliseconds to ticks
    ///
    /// Converts milliseconds to RTC counter ticks.
    ///
    /// # Arguments
    ///
    /// * `millis` - Time in milliseconds
    ///
    /// # Returns
    ///
    /// Corresponding tick count
    pub fn millis_to_ticks(&self, millis: u32) -> u32 {
        // (millis * 32768) / 1000 = (millis * 4096) / 125
        ((millis as u64 * 4096) / 125) as u32
    }
}
