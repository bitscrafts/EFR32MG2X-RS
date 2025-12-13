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
//! use efr32mg24_hal::{
//!     clock::{Clocks, ClockConfig, HfxoConfig},
//!     spi::{Spi0, Config, Mode},
//!     pac,
//! };
//!
//! let dp = pac::Peripherals::take().unwrap();
//!
//! // Configure clocks
//! let clocks = Clocks::new(dp.cmu_s, ClockConfig {
//!     hfxo: Some(HfxoConfig::new(39_000_000)),
//!     lfxo: Some(Default::default()),
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

mod types;
mod traits;

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
        // Enable USART0 clock in CMU (CLKEN0 bit 1)
        critical_section::with(|_cs| {
            let cmu = unsafe { &(*pac::CmuS::ptr()) };
            cmu.clken0().modify(|_, w| w.usart0().set_bit());
        });

        // Enable USART peripheral
        usart.en().write(|w| w.en().set_bit());

        // Configure USART for SPI master mode
        usart.ctrl().write(|w| {
            w.sync().set_bit() // Synchronous mode (SPI)
                .clkpol().bit(config.mode.polarity() == Polarity::IdleHigh)
                .clkpha().bit(config.mode.phase() == Phase::CaptureOnSecondTransition)
                .msbf().bit(config.bit_order == BitOrder::MsbFirst)
        });

        // Configure frame format
        usart.frame().write(|w| unsafe {
            w.databits().bits(0x5) // 8 data bits (value 0x5 = 8 bits)
        });

        // Calculate and set clock divider for SPI frequency
        let clkdiv = Self::calculate_clkdiv(clocks.hfclk().into(), config.frequency);
        usart.clkdiv().write(|w| unsafe { w.bits(clkdiv) });

        // Enable TX and RX
        usart.cmd().write(|w| {
            w.txen().set_bit()
                .rxen().set_bit()
        });

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
        self.usart.txdata().write(|w| unsafe { w.bits(byte as u32) });

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
        // Enable EUSART0 clock in CMU (CLKEN1)
        critical_section::with(|_cs| {
            let cmu = unsafe { &(*pac::CmuS::ptr()) };
            cmu.clken1().modify(|_, w| w.eusart0().set_bit());
        });

        // Enable EUSART peripheral
        eusart.en().write(|w| w.en().set_bit());

        // Configure EUSART for SPI master mode
        eusart.cfg0().write(|w| {
            w.sync().sync() // Synchronous mode (SPI)
                .msbf().bit(config.bit_order == BitOrder::MsbFirst)
        });

        eusart.cfg2().write(|w| {
            w.clkpol().bit(config.mode.polarity() == Polarity::IdleHigh)
                .clkpha().bit(config.mode.phase() == Phase::CaptureOnSecondTransition)
                .master().set_bit() // Master mode
        });

        // Calculate and set clock divider
        let clkdiv = Self::calculate_clkdiv(clocks.hfclk().into(), config.frequency);
        eusart.clkdiv().write(|w| unsafe { w.bits(clkdiv) });

        // Enable TX and RX
        eusart.cmd().write(|w| {
            w.txen().set_bit()
                .rxen().set_bit()
        });

        Self { eusart }
    }

    /// Calculate clock divider for desired SPI frequency
    fn calculate_clkdiv(pclk_hz: u32, spi_hz: u32) -> u32 {
        let divider = (pclk_hz / (2 * spi_hz)).saturating_sub(1);
        (divider * 256).min(0x1FFFFF)
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

    /// Transfer a single byte (blocking)
    fn transfer_byte(&mut self, byte: u8) -> Result<u8, Error> {
        // Wait for TX buffer to be ready (TXFL > 0 means space available)
        while self.eusart.status().read().txfl().bit_is_clear() {}

        // Write byte to TX buffer
        self.eusart.txdata().write(|w| unsafe { w.bits(byte as u32) });

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
        // Enable EUSART1 clock in CMU (CLKEN1)
        critical_section::with(|_cs| {
            let cmu = unsafe { &(*pac::CmuS::ptr()) };
            cmu.clken1().modify(|_, w| w.eusart1().set_bit());
        });

        // Enable EUSART peripheral
        eusart.en().write(|w| w.en().set_bit());

        // Configure EUSART for SPI master mode
        eusart.cfg0().write(|w| {
            w.sync().sync()
                .msbf().bit(config.bit_order == BitOrder::MsbFirst)
        });

        eusart.cfg2().write(|w| {
            w.clkpol().bit(config.mode.polarity() == Polarity::IdleHigh)
                .clkpha().bit(config.mode.phase() == Phase::CaptureOnSecondTransition)
                .master().set_bit()
        });

        // Calculate and set clock divider
        let clkdiv = Self::calculate_clkdiv(clocks.hfclk().into(), config.frequency);
        eusart.clkdiv().write(|w| unsafe { w.bits(clkdiv) });

        // Enable TX and RX
        eusart.cmd().write(|w| {
            w.txen().set_bit()
                .rxen().set_bit()
        });

        Self { eusart }
    }

    fn calculate_clkdiv(pclk_hz: u32, spi_hz: u32) -> u32 {
        let divider = (pclk_hz / (2 * spi_hz)).saturating_sub(1);
        (divider * 256).min(0x1FFFFF)
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
        self.eusart.txdata().write(|w| unsafe { w.bits(byte as u32) });
        while self.eusart.status().read().rxfl().bit_is_clear() {}
        let received = self.eusart.rxdata().read().bits() as u8;
        Ok(received)
    }
}
