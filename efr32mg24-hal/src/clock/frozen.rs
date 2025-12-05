//! Frozen clock configuration
//!
//! Once frozen, clock frequencies cannot be changed, ensuring peripheral stability.

use super::types::Hertz;
use super::clocks::Clocks;

/// Frozen clock configuration
///
/// Once frozen, the clock configuration cannot be changed.
/// This ensures that peripherals depending on clock frequencies
/// have a stable reference.
#[derive(Copy, Clone, Debug)]
pub struct FrozenClocks(pub(super) Clocks);

impl FrozenClocks {
    /// Get the high frequency clock (HFCLK) frequency
    #[inline]
    pub fn hfclk(&self) -> Hertz {
        self.0.hfclk
    }

    /// Get the low frequency clock (LFCLK) frequency
    #[inline]
    pub fn lfclk(&self) -> Hertz {
        self.0.lfclk
    }

    /// Get the peripheral clock (PCLK) frequency
    #[inline]
    pub fn pclk(&self) -> Hertz {
        self.0.pclk
    }

    /// Get the system clock (SYSCLK) frequency
    #[inline]
    pub fn sysclk(&self) -> Hertz {
        self.0.sysclk
    }
}
