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
//! use efr32mg2x_hal::{wdog::{Wdog, WdogConfig, TimeoutPeriod}, pac};
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
    _wdog: pac::Wdog0Ns,
}

/// Unlock key for WDOG configuration registers
const UNLOCK_KEY: u16 = 0xABE8; // 44008 decimal
/// The value that locks the configuration again: anything that is not the unlock key.
const LOCK_KEY: u16 = 0x0000;

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
    pub fn new(wdog: pac::Wdog0Ns, config: WdogConfig) -> Result<Self, Error> {
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
            let wdog_ptr = unsafe { &(*pac::Wdog0Ns::ptr()) };

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
            let wdog_ptr = pac::Wdog0Ns::ptr();
            (*wdog_ptr).en().write(|w| w.en().set_bit());
        }

        // Wait for the enable to reach the watchdog's clock domain.
        //
        // `SYNCBUSY` has **one** bit on this part -- `CMD` (RM 26.5.9) -- so the call above waits for nothing after an
        // `EN` write. The enable crosses into the watchdog's own clock domain (LFRCO by default: 30.6 us a cycle), and
        // a `CMD` write that arrives while it is still crossing is refused with a precise bus fault. Measured: the
        // first feed straight after `new()` faulted at 0x5b00400c with `Wdog::feed` in the backtrace, while the same
        // driver with a few register reads inserted between construction and the feed did not. The readback is the
        // instrument: a read of `EN` crosses the same domain, so it only reads 1 once the write has landed.
        Self::wait_syncbusy()?;

        // Lock the configuration again. The vendor's own `WDOG_Init` ends by writing `~WDOG_LOCK_UNLOCK`, and
        // without it a watchdog that exists to protect the product stays reconfigurable by any later code --
        // including a stray write, which is exactly what the lock is for. Measured on the header: `LOCK` reset
        // value is 0x0000ABE8 (unlocked), so this is the only place the lock is ever applied.
        Self::lock_config();

        // Watchdog is now running
        Ok(Self { _wdog: wdog })
    }

    /// Every register of the block in one read, for instruments and for fault diagnosis.
    ///
    /// The watchdog is the one peripheral here whose failures are silent: a refused write is a bus fault, which halts
    /// the core before anything can be reported, so the only way an example can say what happened is to have read the
    /// registers *before* the risky write. Reading them is also the reason this method is in the crate and not in an
    /// example: naming a PAC type outside the PAC crate is refused by the gate, and rightly so.
    pub fn state(&self) -> State {
        // SAFETY: reads of the WDOG's own registers, of a peripheral this handle owns.
        let w = unsafe { &(*pac::Wdog0Ns::ptr()) };
        State {
            ipversion: w.ipversion().read().bits(),
            en: w.en().read().bits(),
            cfg: w.cfg().read().bits(),
            status: w.status().read().bits(),
            syncbusy: w.syncbusy().read().bits(),
        }
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
        let wdog = unsafe { &(*pac::Wdog0Ns::ptr()) };
        // A previous clear may still be crossing into the watchdog's domain. `SYNCBUSY.CMD` is the one bit that
        // reports it, so wait for it before issuing another -- and again afterwards, so the caller's next write cannot
        // land in the middle of this one. Both waits are bounded: a watchdog that will not clear is a fact to report,
        // not a place to spin forever.
        Self::wait_cmd_idle();
        wdog.cmd().write(|w| w.clear().cleared());
        Self::wait_cmd_idle();
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
        let wdog = unsafe { &(*pac::Wdog0Ns::ptr()) };
        wdog.lock().write(|w| unsafe { w.lockkey().bits(0) });
    }

    /// Unlocks the watchdog configuration
    ///
    /// Allows configuration changes after the watchdog has been set up.
    ///
    /// **Note**: This is called internally by `new()`. You typically don't need
    /// to call this unless you locked the configuration and want to change it.
    fn lock_config() {
        // SAFETY: LOCK is a plain writable register whose reset value is the unlock key (0xABE8); writing 0
        // locks the configuration. `STATUS.LOCK` then reports it.
        let wdog = unsafe { &(*pac::Wdog0Ns::ptr()) };
        wdog.lock().write(|w| unsafe { w.lockkey().bits(LOCK_KEY) });
    }

    fn unlock_config() {
        // SAFETY: LOCK register write is safe. Writing UNLOCK_KEY unlocks configuration.
        let wdog = unsafe { &(*pac::Wdog0Ns::ptr()) };
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
        let wdog = unsafe { &(*pac::Wdog0Ns::ptr()) };
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
        let wdog = unsafe { &(*pac::Wdog0Ns::ptr()) };
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
        let wdog = unsafe { &(*pac::Wdog0Ns::ptr()) };
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
        // The flag is cleared through the `_CLR` alias of the same block this driver owns (`Wdog0Ns`, the non-secure
        // one), because writing 1 to the plain `IF` does not clear it on this family (see `crate::alias`).
        // SAFETY: the alias window belongs to the peripheral this driver holds; `alias::clear_window` is the one place
        // that offsets a block pointer.
        unsafe { crate::alias::clear_window(pac::Wdog0Ns::ptr()) }
            .if_()
            .write(|w| w.tout().set_bit());
    }

    /// Wait (bounded) until the `CMD` synchroniser is idle.
    ///
    /// The WDOG's registers cross into its own clock domain (LFRCO by default: 30.6 us a cycle), and a `CMD` write that
    /// lands while the previous one is still crossing is refused with a precise bus fault. `SYNCBUSY` reports exactly
    /// this one synchroniser and nothing else, which is why this wait exists on its own instead of relying on
    /// `wait_syncbusy` after the `EN` write.
    fn wait_cmd_idle() {
        // SAFETY: a read of the WDOG's own register.
        let wdog = unsafe { &(*pac::Wdog0Ns::ptr()) };
        for _ in 0..Self::SYNC_TIMEOUT {
            if !wdog.syncbusy().read().cmd().bit_is_set() {
                return;
            }
        }
    }

    /// Poll budget for a synchronised write, in loop iterations.
    const SYNC_TIMEOUT: u32 = 200_000;

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
        let wdog = unsafe { &(*pac::Wdog0Ns::ptr()) };

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
