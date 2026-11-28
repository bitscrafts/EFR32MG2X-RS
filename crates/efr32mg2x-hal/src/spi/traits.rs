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
        // Nothing written, or its response already read: there is no transmission to wait for. `STATUS.TXC` is clear
        // after reset, so a bare wait on it hung a bus that had never transferred -- the same defect the USART `flush`
        // had, found by an outside review on 2026-11-28.
        if !self.tx_pending {
            return Ok(());
        }
        let mut spins = 0u32;
        while self.usart.status().read().txc().bit_is_clear() {
            if spins >= self.tx_spins {
                return Err(Error::Timeout);
            }
            spins += 1;
            core::hint::spin_loop();
        }
        self.tx_pending = false;
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
        // `TXIDLE`, not `TXFL`. `TXFL` reads 1 when the FIFO has **room** (its reset value is 1), so waiting on it
        // returns as soon as the software has filled the buffer, with every byte still to go out -- measured on the host
        // in `efr32mg2x-spi-core`: `flush on TXFL returned after 0 polls`. The bit that means "the shifter is done" is
        // `TXIDLE` (`efr32mg24_eusart.h:911`), and the wait is bounded so a stuck shifter is an error, not a hang.
        // Nothing pending: the frame's response was already read, so there is nothing to wait for.
        if !self.tx_pending {
            return Ok(());
        }
        const BUDGET: u32 = 200_000;
        let mut polls = 0u32;
        while !self.eusart.status().read().txidle().bit_is_set() {
            polls += 1;
            if polls > BUDGET {
                return Err(Error::Timeout);
            }
        }
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
        // `TXIDLE`, not `TXFL`. `TXFL` reads 1 when the FIFO has **room** (its reset value is 1), so waiting on it
        // returns as soon as the software has filled the buffer, with every byte still to go out -- measured on the host
        // in `efr32mg2x-spi-core`: `flush on TXFL returned after 0 polls`. The bit that means "the shifter is done" is
        // `TXIDLE` (`efr32mg24_eusart.h:911`), and the wait is bounded so a stuck shifter is an error, not a hang.
        // Nothing pending: the frame's response was already read, so there is nothing to wait for.
        if !self.tx_pending {
            return Ok(());
        }
        const BUDGET: u32 = 200_000;
        let mut polls = 0u32;
        while !self.eusart.status().read().txidle().bit_is_set() {
            polls += 1;
            if polls > BUDGET {
                return Err(Error::Timeout);
            }
        }
        Ok(())
    }
}
