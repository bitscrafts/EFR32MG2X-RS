//! I2C Master Mode Driver
//!
//! This module provides a hardware abstraction layer for the EFR32MG24 I2C peripherals
//! in master mode with support for 7-bit addressing.
//!
//! # Hardware Registers
//!
//! - **EN**: Enable register
//! - **CTRL**: Control register (master/slave mode, clock settings)
//! - **CMD**: Command register (START, STOP, ACK, NACK, etc.)
//! - **STATUS**: Status register (TXBL, RXDATAV, TXC, etc.)
//! - **CLKDIV**: Clock divider for SCL frequency
//! - **TXDATA**: Transmit data register
//! - **RXDATA**: Receive data register
//!
//! # Features
//!
//! - I2C master mode with 7-bit addressing
//! - Configurable SCL frequency (standard 100 kHz, fast 400 kHz)
//! - Blocking write and read operations
//! - embedded-hal v1.0 I2C traits
//!
//! # Example
//!
//! ```no_run
//! use efr32mg24_hal::{
//!     clock::{Clocks, ClockConfig, HfxoConfig},
//!     i2c::{I2c0, Config, Speed},
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
//! // Create I2C instance
//! let mut i2c = I2c0::new(
//!     dp.i2c0_s,
//!     Config::new(Speed::Standard100kHz),
//!     &clocks
//! );
//!
//! // Write to device at address 0x6B
//! i2c.write(0x6B, &[0x75]).unwrap();
//!
//! // Read from device
//! let mut buffer = [0u8; 1];
//! i2c.read(0x6B, &mut buffer).unwrap();
//! ```

mod types;
mod traits;

pub use types::{Config, Error, Speed};

use crate::clock::FrozenClocks;
use crate::pac;

/// I2C0 peripheral instance
pub struct I2c0 {
    i2c: pac::I2c0S,
}

/// I2C1 peripheral instance
pub struct I2c1 {
    i2c: pac::I2c1S,
}

impl I2c0 {
    /// Creates a new I2C0 instance
    ///
    /// # Arguments
    ///
    /// * `i2c` - I2C0_S peripheral instance
    /// * `config` - I2C configuration
    /// * `clocks` - Frozen clock configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// let i2c = I2c0::new(dp.i2c0_s, Config::new(Speed::Standard100kHz), &clocks);
    /// ```
    pub fn new(i2c: pac::I2c0S, config: Config, clocks: &FrozenClocks) -> Self {
        // Enable I2C0 clock in CMU using safe accessor
        clocks.enable_peripheral_clock(|cmu| {
            cmu.clken0().modify(|_, w| w.i2c0().set_bit());
        });

        // Enable I2C peripheral
        i2c.en().write(|w| w.en().set_bit());

        // Configure as master mode
        i2c.ctrl().write(|w| {
            w.slave().clear_bit() // Master mode
                .autoack().clear_bit() // Manual ACK/NACK
                .autosn().set_bit() // Automatic STOP/START
        });

        // Calculate and set clock divider
        let clkdiv = Self::calculate_clkdiv(clocks.hfclk().into(), config.speed);
        i2c.clkdiv().write(|w| unsafe { w.bits(clkdiv) });

        Self { i2c }
    }

    /// Calculate clock divider value for desired I2C speed
    ///
    /// CLKDIV = (fHCLK / (8 * fSCL)) - 1
    fn calculate_clkdiv(hclk_hz: u32, speed: Speed) -> u32 {
        let scl_hz = match speed {
            Speed::Standard100kHz => 100_000,
            Speed::Fast400kHz => 400_000,
        };

        // CLKDIV = (fHCLK / (8 * fSCL)) - 1
        let clkdiv = (hclk_hz / (8 * scl_hz)).saturating_sub(1);
        clkdiv.min(0x1FF) // 9-bit field
    }

    /// Write data to I2C device
    ///
    /// # Arguments
    ///
    /// * `addr` - 7-bit device address
    /// * `bytes` - Data to write
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(Error)` on failure
    pub fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
        if bytes.is_empty() {
            return Err(Error::InvalidData);
        }

        // Send START condition
        self.i2c.cmd().write(|w| w.start().set_bit());

        // Wait for START to complete
        while self.i2c.status().read().pstart().bit_is_set() {}

        // Send address with write bit (R/W = 0)
        let addr_byte = (addr << 1) & 0xFE;
        self.write_byte(addr_byte)?;

        // Write data bytes
        for &byte in bytes {
            self.write_byte(byte)?;
        }

        // Send STOP condition
        self.i2c.cmd().write(|w| w.stop().set_bit());

        // Wait for STOP to complete
        while self.i2c.status().read().pstop().bit_is_set() {}

        Ok(())
    }

    /// Read data from I2C device
    ///
    /// # Arguments
    ///
    /// * `addr` - 7-bit device address
    /// * `buffer` - Buffer to store read data
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(Error)` on failure
    pub fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Error> {
        if buffer.is_empty() {
            return Err(Error::InvalidData);
        }

        // Send START condition
        self.i2c.cmd().write(|w| w.start().set_bit());

        // Wait for START to complete
        while self.i2c.status().read().pstart().bit_is_set() {}

        // Send address with read bit (R/W = 1)
        let addr_byte = (addr << 1) | 0x01;
        self.write_byte(addr_byte)?;

        // Read data bytes
        let last_idx = buffer.len() - 1;
        for (i, byte) in buffer.iter_mut().enumerate() {
            // Wait for RX data valid
            while self.i2c.status().read().rxdatav().bit_is_clear() {}

            // Read byte
            *byte = self.i2c.rxdata().read().bits() as u8;

            // Send ACK for all bytes except last, NACK for last byte
            if i == last_idx {
                self.i2c.cmd().write(|w| w.nack().set_bit());
            } else {
                self.i2c.cmd().write(|w| w.ack().set_bit());
            }
        }

        // Send STOP condition
        self.i2c.cmd().write(|w| w.stop().set_bit());

        // Wait for STOP to complete
        while self.i2c.status().read().pstop().bit_is_set() {}

        Ok(())
    }

    /// Write register then read from I2C device
    ///
    /// # Arguments
    ///
    /// * `addr` - 7-bit device address
    /// * `register` - Register address to read from
    /// * `buffer` - Buffer to store read data
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(Error)` on failure
    pub fn write_read(&mut self, addr: u8, register: u8, buffer: &mut [u8]) -> Result<(), Error> {
        if buffer.is_empty() {
            return Err(Error::InvalidData);
        }

        // Write register address
        self.write(addr, &[register])?;

        // Read data
        self.read(addr, buffer)?;

        Ok(())
    }

    /// Write a single byte to TX buffer and wait for ACK/NACK
    fn write_byte(&mut self, byte: u8) -> Result<(), Error> {
        // Wait for TX buffer ready
        while self.i2c.status().read().txbl().bit_is_clear() {}

        // Write byte to TX buffer
        self.i2c.txdata().write(|w| unsafe { w.bits(byte as u32) });

        // Wait for transmission complete
        while self.i2c.status().read().txc().bit_is_clear() {}

        // Check for NACK (address or data not acknowledged)
        if self.i2c.if_().read().nack().bit_is_set() {
            // Clear NACK flag
            self.i2c.if_().write(|w| w.nack().set_bit());
            return Err(Error::Nack);
        }

        Ok(())
    }
}

impl I2c1 {
    /// Creates a new I2C1 instance
    pub fn new(i2c: pac::I2c1S, config: Config, clocks: &FrozenClocks) -> Self {
        // Enable I2C1 clock in CMU using safe accessor
        clocks.enable_peripheral_clock(|cmu| {
            cmu.clken0().modify(|_, w| w.i2c1().set_bit());
        });

        // Enable I2C peripheral
        i2c.en().write(|w| w.en().set_bit());

        // Configure as master mode
        i2c.ctrl().write(|w| {
            w.slave().clear_bit()
                .autoack().clear_bit()
                .autosn().set_bit()
        });

        // Calculate and set clock divider
        let clkdiv = Self::calculate_clkdiv(clocks.hfclk().into(), config.speed);
        i2c.clkdiv().write(|w| unsafe { w.bits(clkdiv) });

        Self { i2c }
    }

    fn calculate_clkdiv(hclk_hz: u32, speed: Speed) -> u32 {
        let scl_hz = match speed {
            Speed::Standard100kHz => 100_000,
            Speed::Fast400kHz => 400_000,
        };
        let clkdiv = (hclk_hz / (8 * scl_hz)).saturating_sub(1);
        clkdiv.min(0x1FF)
    }

    /// Write data to I2C device
    ///
    /// # Arguments
    ///
    /// * `addr` - 7-bit device address
    /// * `bytes` - Data to write
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(Error)` on failure
    pub fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
        if bytes.is_empty() {
            return Err(Error::InvalidData);
        }

        self.i2c.cmd().write(|w| w.start().set_bit());
        while self.i2c.status().read().pstart().bit_is_set() {}

        let addr_byte = (addr << 1) & 0xFE;
        self.write_byte(addr_byte)?;

        for &byte in bytes {
            self.write_byte(byte)?;
        }

        self.i2c.cmd().write(|w| w.stop().set_bit());
        while self.i2c.status().read().pstop().bit_is_set() {}

        Ok(())
    }

    /// Read data from I2C device
    ///
    /// # Arguments
    ///
    /// * `addr` - 7-bit device address
    /// * `buffer` - Buffer to store read data
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(Error)` on failure
    pub fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Error> {
        if buffer.is_empty() {
            return Err(Error::InvalidData);
        }

        self.i2c.cmd().write(|w| w.start().set_bit());
        while self.i2c.status().read().pstart().bit_is_set() {}

        let addr_byte = (addr << 1) | 0x01;
        self.write_byte(addr_byte)?;

        let last_idx = buffer.len() - 1;
        for (i, byte) in buffer.iter_mut().enumerate() {
            while self.i2c.status().read().rxdatav().bit_is_clear() {}
            *byte = self.i2c.rxdata().read().bits() as u8;

            if i == last_idx {
                self.i2c.cmd().write(|w| w.nack().set_bit());
            } else {
                self.i2c.cmd().write(|w| w.ack().set_bit());
            }
        }

        self.i2c.cmd().write(|w| w.stop().set_bit());
        while self.i2c.status().read().pstop().bit_is_set() {}

        Ok(())
    }

    /// Write register then read from I2C device
    ///
    /// # Arguments
    ///
    /// * `addr` - 7-bit device address
    /// * `register` - Register address to read from
    /// * `buffer` - Buffer to store read data
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(Error)` on failure
    pub fn write_read(&mut self, addr: u8, register: u8, buffer: &mut [u8]) -> Result<(), Error> {
        if buffer.is_empty() {
            return Err(Error::InvalidData);
        }
        self.write(addr, &[register])?;
        self.read(addr, buffer)?;
        Ok(())
    }

    fn write_byte(&mut self, byte: u8) -> Result<(), Error> {
        while self.i2c.status().read().txbl().bit_is_clear() {}
        self.i2c.txdata().write(|w| unsafe { w.bits(byte as u32) });
        while self.i2c.status().read().txc().bit_is_clear() {}

        if self.i2c.if_().read().nack().bit_is_set() {
            self.i2c.if_().write(|w| w.nack().set_bit());
            return Err(Error::Nack);
        }

        Ok(())
    }
}
