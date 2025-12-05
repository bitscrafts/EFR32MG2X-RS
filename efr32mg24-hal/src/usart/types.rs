//! USART type definitions and configuration structures
//!
//! This module defines the types used for configuring USART peripherals.

/// USART data bits configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataBits {
    /// 8 data bits (most common)
    Eight,
    /// 9 data bits
    Nine,
}

/// USART parity configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parity {
    /// No parity
    None,
    /// Even parity
    Even,
    /// Odd parity
    Odd,
}

/// USART stop bits configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopBits {
    /// 1 stop bit
    One,
    /// 2 stop bits
    Two,
}

/// USART configuration structure
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// Baud rate in bits per second
    pub baudrate: u32,
    /// Number of data bits
    pub data_bits: DataBits,
    /// Parity configuration
    pub parity: Parity,
    /// Number of stop bits
    pub stop_bits: StopBits,
}

impl Config {
    /// Create a new USART configuration with common settings
    ///
    /// Default: 8 data bits, no parity, 1 stop bit
    pub const fn new(baudrate: u32) -> Self {
        Self {
            baudrate,
            data_bits: DataBits::Eight,
            parity: Parity::None,
            stop_bits: StopBits::One,
        }
    }

    /// Set the number of data bits
    pub const fn data_bits(mut self, data_bits: DataBits) -> Self {
        self.data_bits = data_bits;
        self
    }

    /// Set the parity configuration
    pub const fn parity(mut self, parity: Parity) -> Self {
        self.parity = parity;
        self
    }

    /// Set the number of stop bits
    pub const fn stop_bits(mut self, stop_bits: StopBits) -> Self {
        self.stop_bits = stop_bits;
        self
    }
}

impl Default for Config {
    /// Default configuration: 115200 baud, 8N1
    fn default() -> Self {
        Self::new(115200)
    }
}

/// USART error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Framing error
    Framing,
    /// Parity error
    Parity,
    /// Overrun error (data lost)
    Overrun,
}

impl embedded_hal_nb::serial::Error for Error {
    fn kind(&self) -> embedded_hal_nb::serial::ErrorKind {
        match self {
            Error::Framing => embedded_hal_nb::serial::ErrorKind::FrameFormat,
            Error::Parity => embedded_hal_nb::serial::ErrorKind::Parity,
            Error::Overrun => embedded_hal_nb::serial::ErrorKind::Overrun,
        }
    }
}
