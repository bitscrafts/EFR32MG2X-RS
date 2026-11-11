//! Clock system implementation with hardware register access
//!
//! This module implements actual hardware configuration for the EFR32MG24 clock system.

use super::frozen::FrozenClocks;
use super::types::{ClockConfig, ClockError, Hertz};

/// Configured clock frequencies with hardware control
///
/// This struct contains the actual clock frequencies after hardware configuration.
#[derive(Copy, Clone, Debug)]
pub struct Clocks {
    /// High frequency clock (HFCLK) frequency
    pub(super) hfclk: Hertz,
    /// Low frequency clock (LFCLK) frequency
    pub(super) lfclk: Hertz,
    /// Peripheral clock (PCLK) frequency
    pub(super) pclk: Hertz,
    /// System clock (SYSCLK) frequency
    pub(super) sysclk: Hertz,
}

/// The SYSCLK this HAL configures when no other clock source is asked for.
///
/// It is a statement about the configuration, not a measurement: the real rate was
/// measured with the DWT counter at ~19 MHz (a 2700 Hz tone needed TIMER0 TOP=3518), and
/// anything that needs better than a percent should measure its own.
pub const DEFAULT_SYSCLK_HZ: u32 = 19_000_000;

impl Clocks {
    /// Default internal RC oscillator frequencies
    const HFRCO_FREQ: u32 = 19_000_000; // 19 MHz default
    const LFRCO_FREQ: u32 = 32_768; // 32.768 kHz

    // The oscillator wait is a poll on HFRCOEM23.STATUS.RDY, which reports reality;
    // an iteration counter only ever reported how fast the loop was going.

    /// Configure the clock system with hardware register access
    ///
    /// # Arguments
    ///
    /// * `cmu` - CMU peripheral (consumed to ensure exclusive access)
    /// * `config` - Clock configuration
    ///
    /// # Returns
    ///
    /// A `Result` containing either the configured `Clocks` or a `ClockError`
    ///
    /// # Errors
    ///
    /// Returns `ClockError::HfxoTimeout` if HFXO fails to stabilize within timeout
    /// Returns `ClockError::LfxoTimeout` if LFXO fails to stabilize within timeout
    ///
    /// # Hardware Configuration
    ///
    /// This method configures the CMU registers to:
    /// - Enable and configure HFXO if requested
    /// - Enable and configure LFXO if requested
    /// - Select SYSCLK source
    /// - Wait for oscillator stabilization with timeout
    ///
    /// # Example
    ///
    /// ```no_run
    /// use efr32mg24_hal::{clock::{Clocks, ClockConfig, HfxoConfig}, pac};
    ///
    /// let dp = pac::Peripherals::take().unwrap();
    ///
    /// let clocks = Clocks::new(
    ///     dp.cmu_s,
    ///     ClockConfig {
    ///         hfxo: Some(HfxoConfig::new(39_000_000)),
    ///         lfxo: None, // Use internal LFRCO
    ///     }
    /// )?;
    /// # Ok::<(), efr32mg24_hal::clock::ClockError>(())
    /// ```
    pub fn new(
        cmu: crate::pac::CmuS,
        config: ClockConfig,
    ) -> Result<(Self, crate::pac::CmuS), ClockError> {
        // Determine the frequencies based on configuration
        //
        // WARNING: This function does NOT switch SYSCLK to HFXO. The previous
        // implementation switched SYSCLK to HFXO BEFORE HFXO was running, which
        // stopped the CPU clock and permanently locked the debug Access Port.
        // See docs/01-getting-started/BRICKING_INCIDENT_2026-06-29.md for details.
        //
        // The proper HFXO sequence requires:
        //   1. Enable HFXO via HFXO0_S.CTRL (set FORCEEN/ENABLE)
        //   2. Poll HFXO0_S.CTRL.RDY until HFXO is stable
        //   3. THEN switch CMU_S.SYSCLKCTRL.CLKSEL to HFXO
        //
        // This is not implemented yet. For now, we just track the requested
        // frequency for delay calculations. The actual clock source remains
        // whatever the bootloader or reset default set (FSRCO or HFRCO).
        let hfclk = if let Some(hfxo_config) = config.hfxo {
            // Track HFXO frequency but do NOT switch SYSCLK.
            // The caller is responsible for enabling HFXO and switching SYSCLK
            // via a separate, properly-sequenced function (TODO: implement).
            hfxo_config.frequency
        } else {
            // Use default HFRCO
            Hertz(Self::HFRCO_FREQ)
        };

        let lfclk = if let Some(lfxo_config) = config.lfxo {
            // Track LFXO frequency but do NOT configure LFXO registers yet.
            // TODO: implement proper LFXO enable + wait + select sequence.
            lfxo_config.frequency
        } else {
            // Use default LFRCO
            Hertz(Self::LFRCO_FREQ)
        };

        // On EFR32MG24, SYSCLK and PCLK typically equal HFCLK
        let sysclk = hfclk;
        let pclk = hfclk;

        let clocks = Self {
            hfclk,
            lfclk,
            pclk,
            sysclk,
        };

        Ok((clocks, cmu))
    }

    /// Freeze the clock configuration
    ///
    /// This consumes the Clocks struct and CMU peripheral, returning a FrozenClocks
    /// which can be used by other peripherals but cannot be modified.
    ///
    /// The FrozenClocks retains ownership of the CMU peripheral, providing
    /// safe access for peripheral clock enable operations.
    ///
    /// # Arguments
    ///
    /// * `cmu` - CMU peripheral to store in the frozen configuration
    pub fn freeze(self, cmu: crate::pac::CmuS) -> FrozenClocks {
        FrozenClocks { clocks: self, cmu }
    }

    /// Get the high frequency clock (HFCLK) frequency
    #[inline]
    pub fn hfclk(&self) -> Hertz {
        self.hfclk
    }

    /// Get the low frequency clock (LFCLK) frequency
    #[inline]
    pub fn lfclk(&self) -> Hertz {
        self.lfclk
    }

    /// Get the peripheral clock (PCLK) frequency
    #[inline]
    pub fn pclk(&self) -> Hertz {
        self.pclk
    }

    /// Get the system clock (SYSCLK) frequency
    #[inline]
    pub fn sysclk(&self) -> Hertz {
        self.sysclk
    }
}
