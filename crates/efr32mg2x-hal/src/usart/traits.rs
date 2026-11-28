//! embedded-hal trait implementations for USART
//!
//! This module implements the embedded-hal v1.0 and embedded-hal-nb traits for USART peripherals.

use super::{Error, RxProblem, Usart0};

/// Error type for USART operations
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Framing => write!(f, "Framing error"),
            Error::Parity => write!(f, "Parity error"),
            Error::Overrun => write!(f, "Overrun error"),
            Error::InvalidBaudrate => write!(f, "Baud rate cannot be produced by this clock"),
            Error::Timeout => write!(f, "The transmitter never made room or never finished"),
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
            // SAFETY: `TXDATA` is as wide as the frame the driver configured; `word` is the `u8` this trait carries, so
            // it always fits an 8-bit frame, and `TXBL` was just seen set, which is what makes the write legal.
            self.usart
                .txdata()
                .write(|w| unsafe { w.txdata().bits(word) });
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        // `TXC` is clear after reset, so a port that has sent nothing reads as "not finished" and this returned
        // `WouldBlock` for ever: `nb::block!` on a fresh port hung. Ask the driver whether anything is pending -- the
        // same question the blocking `flush` asks -- and only then look at the flag.
        if !self.tx_pending {
            return Ok(());
        }
        if self.usart.status().read().txc().bit_is_set() {
            self.tx_pending = false;
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
        match self.read_byte() {
            Some(Ok(byte)) => Ok(byte),
            // A frame the hardware says is broken is not data: the trait has no way to say "consumed but untrustworthy",
            // so it is reported as an error. The byte is gone either way -- refusing to read it would stall the receiver.
            Some(Err(problem)) => Err(nb::Error::Other(match problem {
                RxProblem::Framing => Error::Framing,
                RxProblem::Parity => Error::Parity,
                RxProblem::Overrun => Error::Overrun,
            })),
            None => Err(nb::Error::WouldBlock),
        }
    }
}
