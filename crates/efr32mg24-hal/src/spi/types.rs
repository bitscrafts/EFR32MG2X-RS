//! SPI Types and Configuration

/// SPI configuration
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// SPI mode (clock polarity and phase)
    pub mode: Mode,
    /// SPI bit order
    pub bit_order: BitOrder,
    /// SPI frequency in Hz
    pub frequency: u32,
}

impl Config {
    /// Create new SPI configuration
    ///
    /// # Arguments
    ///
    /// * `mode` - SPI mode (CPOL/CPHA)
    /// * `frequency` - SPI clock frequency in Hz
    ///
    /// # Example
    ///
    /// ```no_run
    /// let config = Config::new(Mode::Mode0, 1_000_000); // 1 MHz, Mode 0
    /// ```
    pub const fn new(mode: Mode, frequency: u32) -> Self {
        Self {
            mode,
            bit_order: BitOrder::MsbFirst,
            frequency,
        }
    }

    /// Set bit order
    pub const fn with_bit_order(mut self, bit_order: BitOrder) -> Self {
        self.bit_order = bit_order;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Mode0,
            bit_order: BitOrder::MsbFirst,
            frequency: 1_000_000, // 1 MHz default
        }
    }
}

/// SPI mode (clock polarity and phase)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// CPOL=0, CPHA=0
    Mode0,
    /// CPOL=0, CPHA=1
    Mode1,
    /// CPOL=1, CPHA=0
    Mode2,
    /// CPOL=1, CPHA=1
    Mode3,
}

impl Mode {
    /// Get clock polarity (CPOL)
    pub const fn polarity(&self) -> Polarity {
        match self {
            Mode::Mode0 | Mode::Mode1 => Polarity::IdleLow,
            Mode::Mode2 | Mode::Mode3 => Polarity::IdleHigh,
        }
    }

    /// Get clock phase (CPHA)
    pub const fn phase(&self) -> Phase {
        match self {
            Mode::Mode0 | Mode::Mode2 => Phase::CaptureOnFirstTransition,
            Mode::Mode1 | Mode::Mode3 => Phase::CaptureOnSecondTransition,
        }
    }
}

/// SPI clock polarity (CPOL)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    /// Clock is low when idle
    IdleLow,
    /// Clock is high when idle
    IdleHigh,
}

/// SPI clock phase (CPHA)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Data captured on first clock transition
    CaptureOnFirstTransition,
    /// Data captured on second clock transition
    CaptureOnSecondTransition,
}

/// SPI bit order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitOrder {
    /// MSB transmitted first (most common)
    MsbFirst,
    /// LSB transmitted first
    LsbFirst,
}

/// SPI errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Overrun error (data received before previous data was read)
    Overrun,
    /// Frame error
    FrameError,
    /// Invalid configuration
    InvalidConfig,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Overrun => write!(f, "SPI overrun error"),
            Error::FrameError => write!(f, "SPI frame error"),
            Error::InvalidConfig => write!(f, "SPI invalid configuration"),
        }
    }
}

// Implement embedded-hal Error trait
impl embedded_hal::spi::Error for Error {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        match self {
            Error::Overrun => embedded_hal::spi::ErrorKind::Overrun,
            Error::FrameError => embedded_hal::spi::ErrorKind::FrameFormat,
            Error::InvalidConfig => embedded_hal::spi::ErrorKind::Other,
        }
    }
}
