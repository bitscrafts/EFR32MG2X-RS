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
    /// The requested baud rate cannot be produced by this clock.
    ///
    /// Added 2026-11-17 with the divider fix: the driver used to program a wrapped number instead of refusing, so a rate
    /// this clock cannot reach produced a console at the wrong speed and no error anywhere.
    InvalidBaudrate,
    /// The transmitter never made room for the next byte, or never finished the last one.
    ///
    /// Added 2026-11-28 with the bounded waits: `write_byte`, `write`, `flush` and `flush_receiver` used to spin
    /// without a limit, so a clock nobody enabled, a mis-routed pad or an unset `TXENS` left the program there for ever
    /// and the caller could not tell -- `write` returned `()`. A hang is not an error a caller can handle.
    Timeout,
}

impl embedded_hal_nb::serial::Error for Error {
    fn kind(&self) -> embedded_hal_nb::serial::ErrorKind {
        match self {
            Error::Framing => embedded_hal_nb::serial::ErrorKind::FrameFormat,
            Error::Parity => embedded_hal_nb::serial::ErrorKind::Parity,
            Error::Overrun => embedded_hal_nb::serial::ErrorKind::Overrun,
            // `embedded-hal-nb` has no kind for "this rate is impossible"; the driver's `Display` says which one it is.
            Error::InvalidBaudrate | Error::Timeout => embedded_hal_nb::serial::ErrorKind::Other,
        }
    }
}
