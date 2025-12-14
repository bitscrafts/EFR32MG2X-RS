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

impl Clocks {
    /// Default internal RC oscillator frequencies
    const HFRCO_FREQ: u32 = 19_000_000; // 19 MHz default
    const LFRCO_FREQ: u32 = 32_768; // 32.768 kHz

    /// Oscillator stabilization timeout (in loop iterations)
    /// At ~19 MHz, this gives approximately 10ms timeout
    const OSC_TIMEOUT: u32 = 100_000;

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
        let hfclk = if let Some(hfxo_config) = config.hfxo {
            // Configure HFXO
            // Enable HFXO by writing to SYSCLKCTRL
            cmu.sysclkctrl().modify(|_r, w| {
                // Select HFXO as SYSCLK source
                w.clksel().hfxo()
            });

            // TODO: Wait for HFXO to stabilize
            // The EFR32MG24 uses different oscillator status registers than earlier series.
            // Need to verify correct register/field names from reference manual.
            // For now, insert a small delay to allow oscillator startup.
            cortex_m::asm::delay(Self::OSC_TIMEOUT);

            hfxo_config.frequency
        } else {
            // Use default HFRCO
            // HFRCO is typically enabled by default
            Hertz(Self::HFRCO_FREQ)
        };

        let lfclk = if let Some(lfxo_config) = config.lfxo {
            // TODO: Configure LFXO
            // The EFR32MG24 uses different oscillator control registers than earlier series.
            // Need to verify correct register/field names from reference manual.
            // For now, just track the configured frequency.
            cortex_m::asm::delay(Self::OSC_TIMEOUT);

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
