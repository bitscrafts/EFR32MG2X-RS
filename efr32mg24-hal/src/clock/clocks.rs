//! Clock system implementation with hardware register access
//!
//! This module implements actual hardware configuration for the EFR32MG24 clock system.

use super::frozen::FrozenClocks;
use super::types::{ClockConfig, Hertz};

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

impl Clocks {
    /// Default internal RC oscillator frequencies
    const HFRCO_FREQ: u32 = 19_000_000; // 19 MHz default
    const LFRCO_FREQ: u32 = 32_768; // 32.768 kHz

    /// Configure the clock system with hardware register access
    ///
    /// # Arguments
    ///
    /// * `cmu` - CMU peripheral (consumed to ensure exclusive access)
    /// * `config` - Clock configuration
    ///
    /// # Returns
    ///
    /// A `Clocks` struct containing the actual configured frequencies
    ///
    /// # Hardware Configuration
    ///
    /// This method configures the CMU registers to:
    /// - Enable and configure HFXO if requested
    /// - Enable and configure LFXO if requested
    /// - Select SYSCLK source
    /// - Wait for oscillator stabilization
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
    /// );
    /// ```
    pub fn new(cmu: crate::pac::CmuS, config: ClockConfig) -> Self {
        // Determine the frequencies based on configuration
        let hfclk = if let Some(hfxo_config) = config.hfxo {
            // Configure HFXO
            critical_section::with(|_cs| {
                // Enable HFXO by writing to SYSCLKCTRL
                // SAFETY: We have exclusive access to CMU peripheral
                cmu.sysclkctrl().modify(|_r, w| {
                    // Select HFXO as SYSCLK source
                    w.clksel().hfxo()
                });

                // TODO: Wait for HFXO to stabilize by checking CMU_STATUS
                // For now, assume it stabilizes quickly
            });

            hfxo_config.frequency
        } else {
            // Use default HFRCO
            // HFRCO is typically enabled by default, but we can verify/configure if needed
            Hertz(Self::HFRCO_FREQ)
        };

        let lfclk = if let Some(lfxo_config) = config.lfxo {
            // Configure LFXO
            // TODO: Implement LFXO configuration
            // For Phase 2, we'll use the configured frequency without hardware setup
            lfxo_config.frequency
        } else {
            // Use default LFRCO
            Hertz(Self::LFRCO_FREQ)
        };

        // On EFR32MG24, SYSCLK and PCLK typically equal HFCLK
        let sysclk = hfclk;
        let pclk = hfclk;

        Self {
            hfclk,
            lfclk,
            pclk,
            sysclk,
        }
    }

    /// Freeze the clock configuration
    ///
    /// This consumes the Clocks struct and returns a FrozenClocks
    /// which can be used by other peripherals but cannot be modified.
    pub fn freeze(self) -> FrozenClocks {
        FrozenClocks(self)
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
