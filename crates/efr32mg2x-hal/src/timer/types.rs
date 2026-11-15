//! Timer and PWM Types
//!
//! This module defines configuration types, modes, and error handling for
//! the EFR32MG24 TIMER peripherals.

/// Timer configuration
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// Timer frequency in Hz
    pub frequency: u32,
    /// PWM mode configuration (None for basic timer mode)
    pub pwm_mode: Option<PwmMode>,
}

impl Config {
    /// Create a new timer configuration
    ///
    /// # Arguments
    ///
    /// * `frequency` - Timer frequency in Hz
    ///
    /// # Example
    ///
    /// ```no_run
    /// let config = Config::new(1_000_000); // 1 MHz timer
    /// ```
    pub const fn new(frequency: u32) -> Self {
        Self {
            frequency,
            pwm_mode: None,
        }
    }

    /// Configure timer for PWM mode
    ///
    /// # Arguments
    ///
    /// * `mode` - PWM mode configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// let config = Config::new(1_000_000).with_pwm(PwmMode::EdgeAligned);
    /// ```
    pub const fn with_pwm(mut self, mode: PwmMode) -> Self {
        self.pwm_mode = Some(mode);
        self
    }
}

/// PWM operating modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PwmMode {
    /// Edge-aligned PWM (standard PWM)
    EdgeAligned,
    /// Center-aligned PWM (symmetric PWM)
    CenterAligned,
}

/// PWM channel identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PwmChannel {
    /// PWM Channel 0
    Channel0,
    /// PWM Channel 1
    Channel1,
    /// PWM Channel 2
    Channel2,
}

impl PwmChannel {
    /// Get channel index (0, 1, or 2)
    pub const fn index(&self) -> u8 {
        match self {
            PwmChannel::Channel0 => 0,
            PwmChannel::Channel1 => 1,
            PwmChannel::Channel2 => 2,
        }
    }
}

/// Timer/PWM errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Invalid frequency configuration
    InvalidFrequency,
    /// Invalid duty cycle value
    InvalidDutyCycle,
    /// Timer is locked
    Locked,
    /// Invalid channel
    InvalidChannel,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidFrequency => write!(f, "Invalid frequency configuration"),
            Error::InvalidDutyCycle => write!(f, "Invalid duty cycle value"),
            Error::Locked => write!(f, "Timer is locked"),
            Error::InvalidChannel => write!(f, "Invalid PWM channel"),
        }
    }
}

// Note: defmt::Format implementation removed to avoid feature flag warnings.
// To re-enable, add 'defmt' to Cargo.toml [features] and [dependencies]
