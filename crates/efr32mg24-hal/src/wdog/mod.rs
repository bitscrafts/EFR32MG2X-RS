//! Watchdog Timer (WDOG0/WDOG1) Module
//!
//! This module provides a Hardware Abstraction Layer (HAL) for the EFR32MG24's
//! Watchdog Timer peripheral, which monitors system operation and resets the device
//! if software fails to periodically "feed" (clear) the watchdog.
//!
//! # Features
//!
//! - **Configurable Timeout**: 16 timeout periods from 9ms to 4.3 minutes
//! - **Energy Mode Support**: Can run in EM0-EM3 (configurable)
//! - **Lock Mechanism**: Configuration can be locked to prevent malicious changes
//! - **Debug Mode**: Can be frozen during debugging (development-friendly)
//! - **Warning Mode**: Can generate interrupts without reset (for testing)
//!
//! # Current Implementation (C5-WDOG Stage 1)
//!
//! - ✅ Watchdog enable with configuration
//! - ✅ Feed (clear) operation
//! - ✅ Lock/unlock configuration
//! - ✅ Timeout period selection (16 options)
//! - ✅ Energy mode support (EM1/EM2/EM3)
//! - ✅ Debug mode control
//! - ✅ Status queries (enabled, locked)
//!
//! # Not Implemented (Future)
//!
//! - ⏳ Interrupt handling
//! - ⏳ PRS (Peripheral Reflex System) integration
//! - ⏳ Window mode (minimum feed interval)
//!
//! # Usage
//!
//! ```rust,no_run
//! use efr32mg24_hal::{wdog::{Wdog, WdogConfig, TimeoutPeriod}, pac};
//!
//! let dp = pac::Peripherals::take().unwrap();
//!
//! // Create configuration: 2 second timeout, runs in all modes
//! let config = WdogConfig::new(TimeoutPeriod::Cycles2k)
//!     .run_in_all_modes();
//!
//! // Enable watchdog
//! let mut wdog = Wdog::new(dp.wdog0_s, config)
//!     .expect("Failed to enable watchdog");
//!
//! loop {
//!     // Do work
//!     perform_task();
//!
//!     // Feed watchdog to prevent reset
//!     wdog.feed();
//!
//!     // If we don't feed within 2 seconds, device will reset
//! }
//! ```
//!
//! # Safety Considerations
//!
//! **Critical for Production**:
//! - Once locked, watchdog configuration CANNOT be changed without reset
//! - Ensure feed interval is shorter than timeout period
//! - Test thoroughly before locking in production
//! - Consider using warning mode first for validation
//!
//! **Lock Key**: 0xABE8 (44008) - required to unlock configuration

mod types;
pub use types::*;

use crate::pac;

/// Watchdog Timer
///
/// Owns the WDOG peripheral and provides watchdog functionality.
///
/// **Important**: The watchdog peripheral can only be configured once after reset
/// unless explicitly unlocked. Once `lock()` is called, configuration is permanent
/// until device reset.
pub struct Wdog {
    _wdog: pac::Wdog0S,
}

/// Unlock key for WDOG configuration registers
const UNLOCK_KEY: u16 = 0xABE8; // 44008 decimal

impl Wdog {
    /// Creates a new Watchdog instance and enables it with the given configuration
    ///
    /// This function:
    /// 1. Enables WDOG0 clock in CMU
    /// 2. Unlocks WDOG configuration (if locked)
    /// 3. Configures watchdog per the config parameter
    /// 4. Enables the watchdog peripheral
    /// 5. Starts the watchdog timer
    ///
    /// **Note**: The watchdog starts counting immediately after this call.
    /// You must call `feed()` before the timeout period expires.
    ///
    /// # Arguments
    ///
    /// * `wdog` - The WDOG0_S peripheral from PAC
    /// * `config` - Watchdog configuration
    ///
    /// # Returns
    ///
    /// * `Ok(Wdog)` - Successfully enabled watchdog
    /// * `Err(Error::Locked)` - Configuration is locked (shouldn't happen on fresh enable)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let dp = pac::Peripherals::take().unwrap();
    /// let config = WdogConfig::new(TimeoutPeriod::Cycles2k);
    /// let wdog = Wdog::new(dp.wdog0_s, config)?;
    /// ```
    pub fn new(wdog: pac::Wdog0S, config: WdogConfig) -> Result<Self, Error> {
        // Enable WDOG0 clock in CMU
        // SAFETY: We access CMU in a critical section to ensure atomicity.
        // WDOG0 clock enable is a safe operation.
        critical_section::with(|_cs| {
            let cmu = unsafe { &(*pac::CmuS::ptr()) };
            cmu.clken0().modify(|_, w| w.wdog0().set_bit());
        });

        // Unlock configuration (in case it was locked before)
        Self::unlock_config();

        // Wait for synchronization
        Self::wait_syncbusy()?;

        // Configure watchdog
        // SAFETY: We own the WDOG peripheral. CFG register configuration is safe.
        // All field values are validated by the type system (bool/enum).
        critical_section::with(|_cs| {
            let wdog_ptr = unsafe { &(*pac::Wdog0S::ptr()) };

            wdog_ptr.cfg().write(|w| unsafe {
                w.persel()
                    .bits(config.period.to_bits())
                    .em1run()
                    .bit(config.em1_run)
                    .em2run()
                    .bit(config.em2_run)
                    .em3run()
                    .bit(config.em3_run)
                    .debugrun()
                    .bit(config.debug_run)
                    .wdogrstdis()
                    .bit(config.reset_disable)
                    .clrsrc()
                    .sw() // Software clear source
            });
        });

        // Wait for configuration to synchronize
        Self::wait_syncbusy()?;

        // Enable the watchdog peripheral
        // SAFETY: EN register enables the peripheral. Safe operation.
        unsafe {
            let wdog_ptr = pac::Wdog0S::ptr();
            (*wdog_ptr).en().write(|w| w.en().set_bit());
        }

        // Wait for enable to synchronize
        Self::wait_syncbusy()?;

        // Watchdog is now running
        Ok(Self { _wdog: wdog })
    }

    /// Feeds (clears) the watchdog timer
    ///
    /// This resets the watchdog counter to 0, preventing a timeout reset.
    /// Must be called periodically before the timeout period expires.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// loop {
    ///     do_work();
    ///     wdog.feed();  // Prevent reset
    /// }
    /// ```
    pub fn feed(&mut self) {
        // SAFETY: CMD register write is safe. CLEAR bit is write-only command.
        let wdog = unsafe { &(*pac::Wdog0S::ptr()) };
        wdog.cmd().write(|w| w.clear().cleared());
    }

    /// Locks the watchdog configuration
    ///
    /// Once locked, the watchdog configuration CANNOT be changed without a device reset.
    /// This prevents malicious or accidental changes to watchdog settings.
    ///
    /// **Warning**: This is a one-way operation until device reset!
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// // Configure and lock watchdog (production use)
    /// let mut wdog = Wdog::new(dp.wdog0_s, config)?;
    /// wdog.lock();  // Now locked until reset
    /// ```
    pub fn lock(&mut self) {
        // SAFETY: LOCK register write is safe. Writing 0 locks the configuration.
        let wdog = unsafe { &(*pac::Wdog0S::ptr()) };
        wdog.lock().write(|w| unsafe { w.lockkey().bits(0) });
    }

    /// Unlocks the watchdog configuration
    ///
    /// Allows configuration changes after the watchdog has been set up.
    ///
    /// **Note**: This is called internally by `new()`. You typically don't need
    /// to call this unless you locked the configuration and want to change it.
    fn unlock_config() {
        // SAFETY: LOCK register write is safe. Writing UNLOCK_KEY unlocks configuration.
        let wdog = unsafe { &(*pac::Wdog0S::ptr()) };
        wdog.lock()
            .write(|w| unsafe { w.lockkey().bits(UNLOCK_KEY) });
    }

    /// Checks if the watchdog configuration is locked
    ///
    /// # Returns
    ///
    /// `true` if configuration is locked, `false` if unlocked
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// if wdog.is_locked() {
    ///     println!("Watchdog is locked - cannot modify config");
    /// }
    /// ```
    pub fn is_locked(&self) -> bool {
        // SAFETY: STATUS register read is safe and non-destructive.
        let wdog = unsafe { &(*pac::Wdog0S::ptr()) };
        wdog.status().read().lock().bit_is_set()
    }

    /// Checks if the watchdog is enabled
    ///
    /// # Returns
    ///
    /// `true` if watchdog is enabled and running, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// if wdog.is_enabled() {
    ///     println!("Watchdog is active");
    /// }
    /// ```
    pub fn is_enabled(&self) -> bool {
        // SAFETY: EN register read is safe and non-destructive.
        let wdog = unsafe { &(*pac::Wdog0S::ptr()) };
        wdog.en().read().en().bit_is_set()
    }

    /// Checks if a watchdog timeout warning occurred
    ///
    /// When `reset_disable` is true (warning-only mode), this flag is set
    /// when timeout occurs instead of resetting the device.
    ///
    /// # Returns
    ///
    /// `true` if timeout warning flag is set, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// if wdog.timeout_flag() {
    ///     println!("Watchdog timeout occurred!");
    ///     wdog.clear_timeout_flag();
    /// }
    /// ```
    pub fn timeout_flag(&self) -> bool {
        // SAFETY: IF register read is safe and non-destructive.
        let wdog = unsafe { &(*pac::Wdog0S::ptr()) };
        wdog.if_().read().tout().bit_is_set()
    }

    /// Clears the watchdog timeout warning flag
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// if wdog.timeout_flag() {
    ///     wdog.clear_timeout_flag();
    /// }
    /// ```
    pub fn clear_timeout_flag(&mut self) {
        // SAFETY: IF register write to clear flags is safe.
        let wdog = unsafe { &(*pac::Wdog0S::ptr()) };
        wdog.if_().write(|w| w.tout().set_bit());
    }

    /// Waits for SYNCBUSY to clear
    ///
    /// The watchdog uses a slow clock domain, so writes need time to synchronize.
    /// This function waits (with timeout) for synchronization to complete.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Synchronization complete
    /// * `Err(Error::Locked)` - Timeout waiting for sync (unlikely)
    fn wait_syncbusy() -> Result<(), Error> {
        // SAFETY: SYNCBUSY register read is safe and non-destructive.
        let wdog = unsafe { &(*pac::Wdog0S::ptr()) };

        // Timeout: ~100k cycles at 78 MHz = ~1.3ms
        const TIMEOUT_CYCLES: u32 = 100_000;
        let mut cycles = 0;

        while wdog.syncbusy().read().bits() != 0 {
            cycles += 1;
            if cycles > TIMEOUT_CYCLES {
                return Err(Error::Locked); // Timeout (shouldn't happen)
            }
        }

        Ok(())
    }
}

// Note: Watchdog does not implement Drop
// Intentionally! Disabling the watchdog would defeat its purpose.
// Once enabled, it stays enabled until device reset.
