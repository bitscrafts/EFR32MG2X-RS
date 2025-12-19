//! ADC Configuration Types

/// ADC resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    /// 12-bit resolution (default)
    Bits12,
}

/// ADC reference voltage source
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reference {
    /// Internal 1.21V bandgap reference
    Vbgr,
    /// VDD supply voltage
    Vdd,
}

/// ADC input channel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// Channel 0 - Port A Pin 0
    Ch0 = 0,
    /// Channel 1 - Port A Pin 1
    Ch1 = 1,
    /// Channel 2 - Port A Pin 2
    Ch2 = 2,
    /// Channel 3 - Port A Pin 3
    Ch3 = 3,
    /// Channel 4 - Port A Pin 4
    Ch4 = 4,
    /// Channel 5 - Port A Pin 5
    Ch5 = 5,
    /// Ground reference (for testing)
    Gnd = 15,
}

/// ADC configuration
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// ADC resolution
    pub resolution: Resolution,
    /// Reference voltage source
    pub reference: Reference,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            resolution: Resolution::Bits12,
            reference: Reference::Vbgr,
        }
    }
}

impl Config {
    /// Create a new ADC configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the ADC resolution
    pub fn with_resolution(mut self, resolution: Resolution) -> Self {
        self.resolution = resolution;
        self
    }

    /// Set the reference voltage source
    pub fn with_reference(mut self, reference: Reference) -> Self {
        self.reference = reference;
        self
    }
}

/// ADC errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Conversion timeout
    Timeout,
    /// Invalid channel
    InvalidChannel,
}
