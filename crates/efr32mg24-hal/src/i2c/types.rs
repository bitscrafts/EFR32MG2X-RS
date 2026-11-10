//! I2C Types and Configuration

/// I2C configuration
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// I2C bus speed
    pub speed: Speed,
}

impl Config {
    /// Create new I2C configuration with specified speed
    pub const fn new(speed: Speed) -> Self {
        Self { speed }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            speed: Speed::Standard100kHz,
        }
    }
}

/// I2C bus speed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speed {
    /// Standard mode: 100 kHz
    Standard100kHz,
    /// Fast mode: 400 kHz
    Fast400kHz,
}

/// I2C errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Address or data not acknowledged (NACK received)
    Nack,
    /// Bus error (arbitration lost, bus timeout, etc.)
    Bus,
    /// Invalid data (empty buffer)
    InvalidData,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Nack => write!(f, "I2C NACK received"),
            Error::Bus => write!(f, "I2C bus error"),
            Error::InvalidData => write!(f, "I2C invalid data"),
        }
    }
}

// Implement embedded-hal Error trait
impl embedded_hal::i2c::Error for Error {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        match self {
            Error::Nack => embedded_hal::i2c::ErrorKind::NoAcknowledge(
                embedded_hal::i2c::NoAcknowledgeSource::Unknown,
            ),
            Error::Bus => embedded_hal::i2c::ErrorKind::Bus,
            Error::InvalidData => embedded_hal::i2c::ErrorKind::Other,
        }
    }
}
