//! embedded-hal v1.0 SPI trait implementations

use super::{Error, Spi0, Spi1, Spi2};
use embedded_hal::spi::{ErrorType, SpiBus};

// Implement embedded-hal ErrorType for Spi0
impl ErrorType for Spi0 {
    type Error = Error;
}

// Implement embedded-hal SpiBus trait for Spi0
impl SpiBus for Spi0 {
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        Spi0::read(self, words)
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        Spi0::write(self, words)
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        Spi0::transfer(self, read, write)
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        for word in words.iter_mut() {
            *word = self.transfer_byte(*word)?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // Wait for transmission to complete
        while self.usart.status().read().txc().bit_is_clear() {}
        Ok(())
    }
}

// Implement embedded-hal ErrorType for Spi1
impl ErrorType for Spi1 {
    type Error = Error;
}

// Implement embedded-hal SpiBus trait for Spi1
impl SpiBus for Spi1 {
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        Spi1::read(self, words)
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        Spi1::write(self, words)
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        Spi1::transfer(self, read, write)
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        for word in words.iter_mut() {
            *word = self.transfer_byte(*word)?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // Wait for TX FIFO to drain (EUSART uses TXFL instead of TXC)
        // TXFL bit is set when FIFO has space, clear when empty
        while self.eusart.status().read().txfl().bit_is_set() {}
        Ok(())
    }
}

// Implement embedded-hal ErrorType for Spi2
impl ErrorType for Spi2 {
    type Error = Error;
}

// Implement embedded-hal SpiBus trait for Spi2
impl SpiBus for Spi2 {
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        Spi2::read(self, words)
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        Spi2::write(self, words)
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        Spi2::transfer(self, read, write)
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        for word in words.iter_mut() {
            *word = self.transfer_byte(*word)?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // Wait for TX FIFO to drain (EUSART uses TXFL instead of TXC)
        // TXFL bit is set when FIFO has space, clear when empty
        while self.eusart.status().read().txfl().bit_is_set() {}
        Ok(())
    }
}
