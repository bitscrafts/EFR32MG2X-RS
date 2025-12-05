// ! Type definitions for clock configuration
//!
//! This module contains basic types and configurations for the clock system.

/// Clock frequencies in Hertz
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Hertz(pub u32);

impl From<u32> for Hertz {
    fn from(freq: u32) -> Self {
        Hertz(freq)
    }
}

impl From<Hertz> for u32 {
    fn from(hz: Hertz) -> u32 {
        hz.0
    }
}

/// High Frequency Crystal Oscillator (HFXO) configuration
#[derive(Copy, Clone, Debug)]
pub struct HfxoConfig {
    /// Crystal frequency in Hz (typically 38.4 MHz or 39 MHz for XIAO MG24)
    pub frequency: Hertz,
}

impl HfxoConfig {
    /// Create a new HFXO configuration with the specified frequency
    pub fn new(frequency: u32) -> Self {
        Self {
            frequency: Hertz(frequency),
        }
    }
}

/// Low Frequency Crystal Oscillator (LFXO) configuration
#[derive(Copy, Clone, Debug)]
pub struct LfxoConfig {
    /// Crystal frequency in Hz (typically 32768 Hz)
    pub frequency: Hertz,
}

impl LfxoConfig {
    /// Create a new LFXO configuration with the specified frequency
    pub fn new(frequency: u32) -> Self {
        Self {
            frequency: Hertz(frequency),
        }
    }
}

impl Default for LfxoConfig {
    fn default() -> Self {
        Self::new(32_768)
    }
}

/// Clock configuration
#[derive(Default, Copy, Clone, Debug)]
pub struct ClockConfig {
    /// High Frequency Crystal Oscillator configuration
    pub hfxo: Option<HfxoConfig>,
    /// Low Frequency Crystal Oscillator configuration
    pub lfxo: Option<LfxoConfig>,
}
