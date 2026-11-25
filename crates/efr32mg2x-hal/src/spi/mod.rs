//! SPI Master Mode Driver
//!
//! This module provides SPI (Serial Peripheral Interface) master mode functionality using
//! USART peripherals in SPI mode for the EFR32MG24 microcontroller.
//!
//! # Hardware Registers
//!
//! - **EN**: Enable register
//! - **CTRL**: Control register (sync mode enable, master/slave)
//! - **FRAME**: Frame format (data bits, clock polarity/phase, bit order)
//! - **CLKDIV**: Clock divider for SCK frequency
//! - **CMD**: Command register (TX/RX enable)
//! - **STATUS**: Status flags (TXBL, RXDATAV, TXC)
//! - **TXDATA**: Transmit data register
//! - **RXDATA**: Receive data register
//!
//! # Features
//!
//! - SPI master mode using USART peripherals
//! - Configurable clock frequency
//! - All 4 SPI modes (Mode 0-3)
//! - MSB-first and LSB-first bit order
//! - Blocking transfer operations
//! - embedded-hal v1.0 SPI traits
//!
//! # Example
//!
//! ```no_run
//! use efr32mg2x_hal::{
//!     clock::{Clocks, ClockConfig, HfxoConfig},
//!     spi::{Spi0, Config, Mode},
//!     pac,
//! };
//!
//! let dp = pac::Peripherals::take().unwrap();
//!
//! // Configure clocks
//! let clocks = Clocks::new(dp.cmu_s, ClockConfig {
//!     hfxo: None,
//!     lfxo: None,
//! }).freeze();
//!
//! // Create SPI instance (1 MHz, Mode 0)
//! let mut spi = Spi0::new(
//!     dp.usart0_s,
//!     Config::new(Mode::Mode0, 1_000_000),
//!     &clocks
//! );
//!
//! // Transfer data
//! let tx_data = [0x01, 0x02, 0x03];
//! let mut rx_data = [0u8; 3];
//! spi.transfer(&mut rx_data, &tx_data).unwrap();
//! ```

mod traits;
mod types;

pub use types::{BitOrder, Config, Error, Mode, Phase, Polarity};

use crate::clock::FrozenClocks;
use crate::pac;

/// SPI0 using USART0 in SPI master mode
pub struct Spi0 {
    usart: pac::Usart0S,
}

/// SPI1 using EUSART0 in SPI master mode
pub struct Spi1 {
    eusart: pac::Eusart0S,
}

/// SPI2 using EUSART1 in SPI master mode
pub struct Spi2 {
    eusart: pac::Eusart1S,
}

impl Spi0 {
    /// Creates a new SPI0 instance using USART0
    ///
    /// # Arguments
    ///
    /// * `usart` - USART0_S peripheral instance
    /// * `config` - SPI configuration (mode, frequency, bit order)
    /// * `clocks` - Frozen clock configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// let spi = Spi0::new(dp.usart0_s, Config::new(Mode::Mode0, 1_000_000), &clocks);
    /// ```
    pub fn new(usart: pac::Usart0S, config: Config, clocks: &FrozenClocks) -> Self {
        // Enable USART0 clock in CMU using safe accessor
        clocks.enable_peripheral_clock(|cmu| {
            cmu.clken0().modify(|_, w| w.usart0().set_bit());
        });

        // Configure first, enable after: this family faults when a peripheral is configured while it is enabled (the
        // project measured exactly that on the TIMER, `board.rs:244`).
        usart.ctrl().write(|w| {
            w.sync()
                .set_bit() // Synchronous mode (SPI)
                .clkpol()
                .bit(config.mode.polarity() == Polarity::IdleHigh)
                .clkpha()
                .bit(config.mode.phase() == Phase::CaptureOnSecondTransition)
                .msbf()
                .bit(config.bit_order == BitOrder::MsbFirst)
        });

        // Configure frame format
        usart.frame().write(|w| unsafe {
            w.databits().bits(0x5) // 8 data bits (value 0x5 = 8 bits)
        });

        // Calculate and set clock divider for SPI frequency
        let clkdiv = Self::calculate_clkdiv(clocks.hfclk().into(), config.frequency);
        usart.clkdiv().write(|w| unsafe { w.bits(clkdiv) });

        // Enable the peripheral, enable TX and RX, then start the master. Without `MASTEREN` (bit 4 of `CMD`,
        // `efr32mg24_usart.h:531`) a configured synchronous USART sits with the bus clock running and never transfers,
        // which is the half of H8 this block was missing.
        usart.en().write(|w| w.en().set_bit());
        usart.cmd().write(|w| w.txen().set_bit().rxen().set_bit());
        usart.cmd().write(|w| w.masteren().set_bit());

        Self { usart }
    }

    /// Calculate clock divider for desired SPI frequency
    ///
    /// CLKDIV = 256 * (PCLK / (2 * fSCK) - 1)
    ///
    /// For SPI mode, the divider is different from UART mode.
    /// The SPI clock (SCK) = PCLK / (2 * (1 + CLKDIV/256))
    fn calculate_clkdiv(pclk_hz: u32, spi_hz: u32) -> u32 {
        // CLKDIV = 256 * (PCLK / (2 * fSCK) - 1)
        let divider = (pclk_hz / (2 * spi_hz)).saturating_sub(1);
        (divider * 256).min(0x1FFFFF) // 21-bit field
    }

    /// Transfer data (full-duplex)
    ///
    /// Simultaneously sends and receives data. The received data is stored in `read`,
    /// and data from `write` is transmitted.
    ///
    /// # Arguments
    ///
    /// * `read` - Buffer to store received data
    /// * `write` - Data to transmit
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(Error)` on failure
    pub fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        if read.len() != write.len() {
            return Err(Error::InvalidConfig);
        }

        for (rx, tx) in read.iter_mut().zip(write.iter()) {
            *rx = self.transfer_byte(*tx)?;
        }

        Ok(())
    }

    /// Write data (half-duplex, discard received data)
    ///
    /// # Arguments
    ///
    /// * `words` - Data to write
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(Error)` on failure
    pub fn write(&mut self, words: &[u8]) -> Result<(), Error> {
        for &word in words {
            self.transfer_byte(word)?;
        }
        Ok(())
    }

    /// Read data (half-duplex, send zeros)
    ///
    /// # Arguments
    ///
    /// * `words` - Buffer to store read data
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(Error)` on failure
    pub fn read(&mut self, words: &mut [u8]) -> Result<(), Error> {
        for word in words.iter_mut() {
            *word = self.transfer_byte(0x00)?;
        }
        Ok(())
    }

    /// Transfer a single byte (blocking)
    fn transfer_byte(&mut self, byte: u8) -> Result<u8, Error> {
        // Wait for TX buffer to be ready
        while self.usart.status().read().txbl().bit_is_clear() {}

        // Write byte to TX buffer
        self.usart
            .txdata()
            .write(|w| unsafe { w.bits(byte as u32) });

        // Wait for RX data to be valid
        while self.usart.status().read().rxdatav().bit_is_clear() {}

        // Read received byte
        let received = self.usart.rxdata().read().bits() as u8;

        Ok(received)
    }
}

impl Spi1 {
    /// Creates a new SPI1 instance using EUSART0
    ///
    /// # Arguments
    ///
    /// * `eusart` - EUSART0_S peripheral instance
    /// * `config` - SPI configuration (mode, frequency, bit order)
    /// * `clocks` - Frozen clock configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// let spi = Spi1::new(dp.eusart0_s, Config::new(Mode::Mode0, 1_000_000), &clocks);
    /// ```
    pub fn new(eusart: pac::Eusart0S, config: Config, clocks: &FrozenClocks) -> Self {
        // Enable EUSART0 clock in CMU using safe accessor
        clocks.enable_peripheral_clock(|cmu| {
            cmu.clken1().modify(|_, w| w.eusart0().set_bit());
        });
        Self::new_at(eusart, config, clocks.hfclk().into())
    }

    /// The same driver, built from the HFCLK in hertz instead of a `FrozenClocks`.
    ///
    /// `FrozenClocks` owns the CMU and the `Board` owns the CMU, so a caller that took the board can never build
    /// one — the same knot `Board::i2c0` and `Board::usart0` untie by taking the number. The caller must have
    /// enabled `CMU_CLKEN1.EUSART0`; `Board::spi_on` does.
    pub fn new_at(eusart: pac::Eusart0S, config: Config, hfclk_hz: u32) -> Self {
        // Configure **before** enabling, and put the divider where the synchronous path reads it.
        //
        // The previous order was: enable the peripheral, then write CFG0/CFG2. The project has already measured that
        // this family faults when a peripheral is configured after it is enabled (the TIMER at `board.rs:244`), and the
        // divider was written into `CLKDIV` -- the *asynchronous* divider -- while in synchronous mode the divider is
        // `CFG2.SDIV`, an 8-bit field at bit 24 (`efr32mg24_eusart.h:565`). Both halves are measured on the host in
        // `efr32mg2x-spi-core`.
        eusart.cfg0().write(|w| {
            w.sync()
                .sync() // Synchronous mode (SPI)
                .msbf()
                .bit(config.bit_order == BitOrder::MsbFirst)
        });

        let sdiv = match crate::cores::spi::sdiv(hfclk_hz, config.frequency) {
            Ok(value) => value as u8,
            Err(_) => 0xFF, // the slowest expressible rate: reported by `actual_frequency`, never silent
        };
        eusart.cfg2().write(|w| unsafe {
            w.clkpol()
                .bit(config.mode.polarity() == Polarity::IdleHigh)
                .clkpha()
                .bit(config.mode.phase() == Phase::CaptureOnSecondTransition)
                .master()
                .set_bit() // Master mode
                .sdiv()
                .bits(sdiv)
        });

        // Enable TX and RX, then enable the peripheral. The EUSART starts transferring once `CFG2.MASTER` is set and a
        // byte is queued; `CMD.MASTEREN` does not exist on this peripheral (it is a USART bit, bit 4 of CMD --
        // `efr32mg24_usart.h:531`), which is worth stating because the review named it for the wrong block.
        eusart.cmd().write(|w| w.txen().set_bit().rxen().set_bit());
        eusart.en().write(|w| w.en().set_bit());

        Self { eusart }
    }

    /// Calculate clock divider for desired SPI frequency
    /// Transfer data (full-duplex)
    ///
    /// # Arguments
    ///
    /// * `read` - Buffer to store received data
    /// * `write` - Data to transmit
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(Error)` on failure
    pub fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        if read.len() != write.len() {
            return Err(Error::InvalidConfig);
        }

        for (rx, tx) in read.iter_mut().zip(write.iter()) {
            *rx = self.transfer_byte(*tx)?;
        }

        Ok(())
    }

    /// Write data (half-duplex)
    ///
    /// # Arguments
    ///
    /// * `words` - Data to write
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(Error)` on failure
    pub fn write(&mut self, words: &[u8]) -> Result<(), Error> {
        for &word in words {
            self.transfer_byte(word)?;
        }
        Ok(())
    }

    /// Read data (half-duplex)
    ///
    /// # Arguments
    ///
    /// * `words` - Buffer to store read data
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(Error)` on failure
    pub fn read(&mut self, words: &mut [u8]) -> Result<(), Error> {
        for word in words.iter_mut() {
            *word = self.transfer_byte(0x00)?;
        }
        Ok(())
    }

    /// Transfer a single byte (blocking)
    fn transfer_byte(&mut self, byte: u8) -> Result<u8, Error> {
        // Wait for TX buffer to be ready (TXFL > 0 means space available)
        while self.eusart.status().read().txfl().bit_is_clear() {}

        // Write byte to TX buffer
        self.eusart
            .txdata()
            .write(|w| unsafe { w.bits(byte as u32) });

        // Wait for RX data to be valid (RXFL > 0 means data available)
        while self.eusart.status().read().rxfl().bit_is_clear() {}

        // Read received byte
        let received = self.eusart.rxdata().read().bits() as u8;

        Ok(received)
    }
}

impl Spi2 {
    /// Creates a new SPI2 instance using EUSART1
    ///
    /// # Arguments
    ///
    /// * `eusart` - EUSART1_S peripheral instance
    /// * `config` - SPI configuration (mode, frequency, bit order)
    /// * `clocks` - Frozen clock configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// let spi = Spi2::new(dp.eusart1_s, Config::new(Mode::Mode0, 1_000_000), &clocks);
    /// ```
    pub fn new(eusart: pac::Eusart1S, config: Config, clocks: &FrozenClocks) -> Self {
        // Enable EUSART1 clock in CMU using safe accessor
        clocks.enable_peripheral_clock(|cmu| {
            cmu.clken1().modify(|_, w| w.eusart1().set_bit());
        });

        // Same order and the same divider register as `Spi1`: configure, then enable, with the rate in `CFG2.SDIV`.
        eusart.cfg0().write(|w| {
            w.sync()
                .sync()
                .msbf()
                .bit(config.bit_order == BitOrder::MsbFirst)
        });

        let sdiv = match crate::cores::spi::sdiv(clocks.hfclk().into(), config.frequency) {
            Ok(value) => value as u8,
            Err(_) => 0xFF,
        };
        eusart.cfg2().write(|w| unsafe {
            w.clkpol()
                .bit(config.mode.polarity() == Polarity::IdleHigh)
                .clkpha()
                .bit(config.mode.phase() == Phase::CaptureOnSecondTransition)
                .master()
                .set_bit()
                .sdiv()
                .bits(sdiv)
        });

        eusart.cmd().write(|w| w.txen().set_bit().rxen().set_bit());
        eusart.en().write(|w| w.en().set_bit());

        Self { eusart }
    }

    /// Transfer data (full-duplex)
    ///
    /// # Arguments
    ///
    /// * `read` - Buffer to store received data
    /// * `write` - Data to transmit
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(Error)` on failure
    pub fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        if read.len() != write.len() {
            return Err(Error::InvalidConfig);
        }

        for (rx, tx) in read.iter_mut().zip(write.iter()) {
            *rx = self.transfer_byte(*tx)?;
        }

        Ok(())
    }

    /// Write data (half-duplex)
    ///
    /// # Arguments
    ///
    /// * `words` - Data to write
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(Error)` on failure
    pub fn write(&mut self, words: &[u8]) -> Result<(), Error> {
        for &word in words {
            self.transfer_byte(word)?;
        }
        Ok(())
    }

    /// Read data (half-duplex)
    ///
    /// # Arguments
    ///
    /// * `words` - Buffer to store read data
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(Error)` on failure
    pub fn read(&mut self, words: &mut [u8]) -> Result<(), Error> {
        for word in words.iter_mut() {
            *word = self.transfer_byte(0x00)?;
        }
        Ok(())
    }

    fn transfer_byte(&mut self, byte: u8) -> Result<u8, Error> {
        while self.eusart.status().read().txfl().bit_is_clear() {}
        self.eusart
            .txdata()
            .write(|w| unsafe { w.bits(byte as u32) });
        while self.eusart.status().read().rxfl().bit_is_clear() {}
        let received = self.eusart.rxdata().read().bits() as u8;
        Ok(received)
    }
}
