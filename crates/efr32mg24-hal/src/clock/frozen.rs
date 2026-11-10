//! Frozen clock configuration
//!
//! Once frozen, clock frequencies cannot be changed, ensuring peripheral stability.

use super::clocks::Clocks;
use super::types::Hertz;

/// Frozen clock configuration
///
/// Once frozen, the clock configuration cannot be changed.
/// This ensures that peripherals depending on clock frequencies
/// have a stable reference.
///
/// The FrozenClocks struct retains ownership of the CMU peripheral,
/// providing safe access for peripheral clock enable operations.
pub struct FrozenClocks {
    pub(super) clocks: Clocks,
    pub(super) cmu: crate::pac::CmuS,
}

impl FrozenClocks {
    /// Get the high frequency clock (HFCLK) frequency
    #[inline]
    pub fn hfclk(&self) -> Hertz {
        self.clocks.hfclk
    }

    /// Get the low frequency clock (LFCLK) frequency
    #[inline]
    pub fn lfclk(&self) -> Hertz {
        self.clocks.lfclk
    }

    /// Get the peripheral clock (PCLK) frequency
    #[inline]
    pub fn pclk(&self) -> Hertz {
        self.clocks.pclk
    }

    /// Get the system clock (SYSCLK) frequency
    #[inline]
    pub fn sysclk(&self) -> Hertz {
        self.clocks.sysclk
    }

    /// Enable peripheral clock with safe CMU access
    ///
    /// This method provides safe access to the CMU peripheral for
    /// enabling peripheral clocks. It takes a closure that receives
    /// a reference to the CMU peripheral.
    ///
    /// This method is public to allow peripheral drivers to safely
    /// enable their clocks without requiring unsafe pointer access.
    ///
    /// # Arguments
    ///
    /// * `f` - Closure that performs the clock enable operation
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use efr32mg24_hal::{clock::FrozenClocks, pac};
    /// # let clocks: FrozenClocks = unimplemented!();
    /// clocks.enable_peripheral_clock(|cmu| {
    ///     cmu.clken0().modify(|_, w| w.gpio().set_bit());
    /// });
    /// ```
    #[inline]
    pub fn enable_peripheral_clock<F>(&self, f: F)
    where
        F: FnOnce(&crate::pac::CmuS),
    {
        critical_section::with(|_cs| {
            f(&self.cmu);
        });
    }
}
