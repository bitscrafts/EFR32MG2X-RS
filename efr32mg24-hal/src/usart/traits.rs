//! embedded-hal trait implementations for USART
//!
//! This module implements the embedded-hal v1.0 and embedded-hal-nb traits for USART peripherals.

use super::{Error, Usart0};

/// Error type for USART operations
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Framing => write!(f, "Framing error"),
            Error::Parity => write!(f, "Parity error"),
            Error::Overrun => write!(f, "Overrun error"),
        }
    }
}

/// ErrorType implementation for USART0
impl embedded_hal_nb::serial::ErrorType for Usart0 {
    type Error = Error;
}

/// embedded-hal-nb Write trait implementation for USART0
///
/// Provides non-blocking write operations compatible with embedded-hal-nb.
impl embedded_hal_nb::serial::Write<u8> for Usart0 {
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        // Check if TX buffer is ready
        if self.usart.status().read().txbl().bit_is_set() {
            self.usart
                .txdata()
                .write(|w| unsafe { w.txdata().bits(word) });
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        // Check if transmission is complete
        if self.usart.status().read().txc().bit_is_set() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

/// embedded-hal-nb Read trait implementation for USART0
///
/// Provides non-blocking read operations compatible with embedded-hal-nb.
impl embedded_hal_nb::serial::Read<u8> for Usart0 {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.read_byte().ok_or(nb::Error::WouldBlock)
    }
}
