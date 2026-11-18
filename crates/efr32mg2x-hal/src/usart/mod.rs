//! USART (Universal Synchronous/Asynchronous Receiver/Transmitter) driver
//!
//! This module provides a safe interface to the EFR32MG24 USART peripherals for
//! serial communication. It supports:
//!
//! - Configurable baud rates
//! - 8 or 9 data bits
//! - None, even, or odd parity
//! - 1 or 2 stop bits
//! - Blocking TX/RX operations
//!
//! ## Hardware Registers
//!
//! The driver directly manipulates these USART hardware registers:
//!
//! - **EN**: Enable register (USART peripheral enable)
//! - **FRAME**: Frame format (data bits, parity, stop bits)
//! - **CLKDIV**: Clock divider for baud rate generation
//! - **CMD**: Command register (enable TX/RX, clear FIFOs)
//! - **STATUS**: Status flags (TXBL, RXDATAV, TXC, etc.)
//! - **TXDATA**: Transmit data register (8-bit write-only)
//! - **RXDATA**: Receive data register (8-bit read-only)
//!
//! ## Example
//!
//! ```no_run
//! use efr32mg2x_hal::usart::{Usart0, Config};
//! use efr32mg2x_hal::clock::Clocks;
//!
//! let dp = pac::Peripherals::take().unwrap();
//! let clocks = Clocks::new(dp.CMU_S);
//!
//! // Configure USART0 for 115200 baud, 8N1
//! let mut usart = Usart0::new(dp.USART0_S, Config::default(), &clocks);
//!
//! // Transmit a byte
//! usart.write_byte(b'H');
//!
//! // Receive a byte
//! if let Some(data) = usart.read_byte() {
//!     // Process received data
//! }
//! ```

mod traits;
mod types;

pub use types::{Config, DataBits, Error, Parity, StopBits};

use crate::clock::FrozenClocks;
use efr32mg2x_pac as pac;

/// USART0 peripheral
pub struct Usart0 {
    pub(crate) usart: pac::Usart0S,
}

impl Usart0 {
    /// Create a new USART0 instance
    ///
    /// Takes ownership of the USART0_S peripheral and configures it according
    /// to the provided configuration. Requires clock information for accurate
    /// baud rate calculation.
    ///
    /// # Arguments
    ///
    /// * `usart` - USART0_S peripheral from PAC
    /// * `config` - USART configuration (baud rate, data bits, parity, stop bits)
    /// * `clocks` - Frozen clock configuration for baud rate calculation
    ///
    /// # Example
    ///
    /// ```no_run
    /// let config = Config::new(115200)
    ///     .data_bits(DataBits::Eight)
    ///     .parity(Parity::None)
    ///     .stop_bits(StopBits::One);
    ///
    /// let usart = Usart0::new(dp.USART0_S, config, &clocks);
    /// ```
    pub fn new(usart: pac::Usart0S, config: Config, clocks: &FrozenClocks) -> Result<Self, Error> {
        // Enable USART0 clock in CMU using safe accessor
        // Typed accessor, not a bit number. The previous code set bit 1 and commented it "USART0 clock enable";
        // `CLKEN0` bit 1 is `LDMAXBAR` and USART0 is bit 9 (both read from the generated PAC's field documentation), so
        // the peripheral was never clocked and the bus it needs was. With the accessor the bit cannot drift from the
        // name.
        clocks.enable_peripheral_clock(|cmu| {
            cmu.clken0().modify(|_r, w| w.usart0().set_bit());
        });

        // Enable USART peripheral
        usart.en().write(|w| w.en().set_bit());

        // Configure frame format
        usart.frame().write(|w| {
            // Set data bits
            let databits = match config.data_bits {
                DataBits::Eight => 5, // FRAME.DATABITS = 5 means 8 bits
                DataBits::Nine => 6,  // FRAME.DATABITS = 6 means 9 bits
            };
            unsafe { w.databits().bits(databits) };

            // Set parity
            match config.parity {
                Parity::None => w.parity().none(),
                Parity::Even => w.parity().even(),
                Parity::Odd => w.parity().odd(),
            };

            // Set stop bits
            match config.stop_bits {
                StopBits::One => w.stopbits().one(),
                StopBits::Two => w.stopbits().two(),
            }
        });

        // The divider comes from `efr32mg2x-usart-core`, where it is measured on the host. The expression that used to be
        // here multiplied in `u32`: at 19 MHz `256 * 19 000 000` is 4 864 000 000, which wraps, and the value written to
        // `DIV` was 52 where the correct divider is 2382 -- a console at the wrong speed and no error anywhere.
        let pclk: u32 = clocks.pclk().into();
        let oversample = 16;
        let clkdiv = match crate::cores::usart::clkdiv(pclk, config.baudrate, oversample) {
            Ok(div) => div,
            Err(_) => {
                // A rate this clock cannot produce is refused rather than programmed as a wrapped number.
                return Err(crate::usart::Error::InvalidBaudrate);
            }
        };
        usart.clkdiv().write(|w| unsafe { w.div().bits(clkdiv) });

        // Enable TX and RX
        usart.cmd().write(|w| {
            w.txen().set_bit();
            w.rxen().set_bit()
        });

        Ok(Self { usart })
    }

    /// Write a single byte (blocking)
    ///
    /// Waits until the TX buffer has space, then writes the byte.
    ///
    /// # Hardware Operation
    ///
    /// 1. Polls STATUS.TXBL (TX buffer level) until set
    /// 2. Writes byte to TXDATA register
    ///
    /// # Example
    ///
    /// ```no_run
    /// usart.write_byte(b'A');
    /// ```
    pub fn write_byte(&mut self, byte: u8) {
        // Wait for TX buffer to be ready
        while !self.usart.status().read().txbl().bit_is_set() {
            core::hint::spin_loop();
        }

        // Write byte to TXDATA
        self.usart
            .txdata()
            .write(|w| unsafe { w.txdata().bits(byte) });
    }

    /// Read a single byte (non-blocking)
    ///
    /// Returns `Some(byte)` if data is available, `None` otherwise.
    ///
    /// # Hardware Operation
    ///
    /// 1. Checks STATUS.RXDATAV (RX data valid) flag
    /// 2. If set, reads byte from RXDATA register
    ///
    /// # Example
    ///
    /// ```no_run
    /// if let Some(byte) = usart.read_byte() {
    ///     // Process received byte
    /// }
    /// ```
    pub fn read_byte(&mut self) -> Option<u8> {
        if self.usart.status().read().rxdatav().bit_is_set() {
            Some(self.usart.rxdata().read().rxdata().bits())
        } else {
            None
        }
    }

    /// Write a slice of bytes (blocking)
    ///
    /// Writes all bytes in the slice sequentially.
    ///
    /// # Example
    ///
    /// ```no_run
    /// usart.write(b"Hello, world!\r\n");
    /// ```
    pub fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_byte(byte);
        }
    }

    /// Flush the TX buffer (blocking)
    ///
    /// Waits until all transmitted data has been sent (TX complete).
    ///
    /// # Hardware Operation
    ///
    /// Polls STATUS.TXC (TX complete) until set.
    pub fn flush(&mut self) {
        while !self.usart.status().read().txc().bit_is_set() {
            core::hint::spin_loop();
        }
    }
}
