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

pub use crate::cores::usart::RxProblem;
pub use types::{Config, DataBits, Error, Parity, StopBits};

use crate::clock::FrozenClocks;
use crate::cores::usart::{rx_problem, IF_FERR, IF_PERR, IF_RXOF};
use efr32mg2x_pac as pac;

/// USART0 peripheral
pub struct Usart0 {
    pub(crate) usart: pac::Usart0S,
    /// How many bytes were refused because the receiver said the frame was broken.
    ///
    /// A refused byte is not an error the caller can handle -- there is nothing to hand it -- so it is counted instead,
    /// and an instrument prints the count next to the byte count. A link that reports bytes and refuses some of them is a
    /// link with a problem, and the two numbers together say so; either alone can look like a working link.
    rx_errors: u32,
    /// Why the last one was refused, so a record can say `ferr` instead of leaving a reader to guess.
    last_rx_problem: Option<RxProblem>,
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
        Self::new_at(usart, config, clocks.pclk().into())
    }

    /// The same driver, built from a peripheral clock in hertz instead of a `FrozenClocks`.
    ///
    /// `FrozenClocks` owns the CMU, and the `Board` owns the CMU inside `Peripherals` — so a caller that already
    /// took the board can never build one. That is the same knot `Board::i2c0` unties by taking a `Hertz`, and the
    /// baud divider only needs one number: PCLK. On this part PCLK = HFCLK (the clock tree sets it so, and nothing in
    /// this crate touches the prescaler), which the board tracks as `sysclk_hz`.
    ///
    /// The caller must have enabled `CMU_CLKEN0.USART0`; `Board::usart0` does.
    pub fn new_at(usart: pac::Usart0S, config: Config, pclk_hz: u32) -> Result<Self, Error> {
        // Enable USART peripheral
        usart.en().write(|w| w.en().set_bit());

        // Configure frame format
        usart.frame().write(|w| {
            // Set data bits
            let databits = match config.data_bits {
                DataBits::Eight => 5, // FRAME.DATABITS = 5 means 8 bits
                DataBits::Nine => 6,  // FRAME.DATABITS = 6 means 9 bits
            };
            // SAFETY: `DATABITS` is written with its encoded value (`5` for eight bits, `6` for nine, as the manual defines)
            // and the value comes from the `DataBits` enum, never from a caller's number.
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
        let pclk: u32 = pclk_hz;
        let oversample = 16;
        let clkdiv = match crate::cores::usart::clkdiv(pclk, config.baudrate, oversample) {
            Ok(div) => div,
            Err(_) => {
                // A rate this clock cannot produce is refused rather than programmed as a wrapped number.
                return Err(crate::usart::Error::InvalidBaudrate);
            }
        };
        // SAFETY: `DIV` is 16 bits and `clkdiv` comes from `cores::usart::clkdiv`, which multiplies in `u64` and refuses a
        // rate it cannot represent. The wrap that wrote 52 where 2382 belongs is exactly what this replaced.
        usart.clkdiv().write(|w| unsafe { w.div().bits(clkdiv) });

        // Enable TX and RX
        usart.cmd().write(|w| {
            w.txen().set_bit();
            w.rxen().set_bit()
        });

        Ok(Self {
            usart,
            rx_errors: 0,
            last_rx_problem: None,
        })
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
            // SAFETY: `TXDATA` holds the frame; `byte` is a `u8` and the frame is 8 or 9 bits wide, so the value fits.
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
    /// Read a byte, and say whether the frame it arrived in can be believed.
    ///
    /// The byte is **always consumed**: the receiver only advances when `RXDATA` is read, so refusing to read a broken
    /// frame stalls every byte behind it. What the caller gets is the byte *and* the verdict, which is why this returns
    /// `Result` inside the `Option`: `None` means nothing had arrived, `Some(Err(..))` means something arrived and the
    /// hardware said the frame was broken. Dropping it is the caller's decision, and the right one for most callers -- but
    /// it has to be a decision, not a side effect of the driver keeping the byte to itself.
    ///
    /// Measured on the part, 2026-11-25: the error flags in `IF` do **not** clear by writing one to them through the
    /// secure instance this board hands out (`pre=1f cleared=1f post=1f`), so a driver that refused and expected the next
    /// byte to be clean would refuse for the rest of the program's life. Reporting instead of swallowing is what keeps the
    /// receiver alive.
    pub fn read_byte(&mut self) -> Option<Result<u8, RxProblem>> {
        if !self.usart.status().read().rxdatav().bit_is_set() {
            return None;
        }

        // Read first: this is what advances the receiver.
        let byte = self.usart.rxdata().read().rxdata().bits();

        // This MCU has no per-byte error bit in `STATUS`; the only evidence that the frame was broken is in `IF`. That is
        // the whole reason this check exists: with nothing driving the RX pad, `hal_usart_link` once reported
        // `echo n=271` -- 271 bytes nobody sent -- because a floating line produces start bits out of noise.
        match rx_problem(self.rx_flags()) {
            Some(problem) => {
                self.rx_errors = self.rx_errors.saturating_add(1);
                self.last_rx_problem = Some(problem);
                Some(Err(problem))
            }
            None => Some(Ok(byte)),
        }
    }

    /// Connect the receiver to this peripheral's own transmitter, inside the part.
    ///
    /// The cheat that makes a link provable with one board: the receiver sees the transmitter past the pads, so a byte
    /// written comes back as a valid frame. A **self-test**, not a substitute for a second machine -- it proves the
    /// shifter, the frame format and the read path, and proves nothing about the pad or the level on it.
    pub fn set_internal_loopback(&mut self, on: bool) {
        self.usart.ctrl().modify(|_, w| w.loopbk().bit(on));
    }

    /// The receiver's error flags, straight out of `IF`.
    ///
    /// `RXOF` at bit 4, `PERR` at 8, `FERR` at 9. Exposed because whether they clear, and how, is a fact about this part
    /// that has to be measured rather than assumed.
    #[must_use]
    pub fn rx_flags(&self) -> u32 {
        self.usart.if_().read().bits()
    }

    /// Clear whichever receiver error flags are set, and return what they were.
    ///
    /// The write is a one to each set bit, which is what the reference manual gives as the way to clear `IF`. Measured on
    /// the part 2026-11-25 through the secure instance this board hands out: **it does not clear them** (`pre=1f
    /// cleared=1f post=1f`, from `hal_usart_link`'s selftest record). The method stays because it is the documented
    /// sequence and the next thing to try is the non-secure alias, not because it works today.
    pub fn clear_rx_flags(&mut self) -> u32 {
        let flags = self.rx_flags();
        if flags & (IF_RXOF | IF_PERR | IF_FERR) != 0 {
            self.usart.if_().write(|w| {
                if flags & IF_RXOF != 0 {
                    w.rxof().set_bit();
                }
                if flags & IF_PERR != 0 {
                    w.perr().set_bit();
                }
                if flags & IF_FERR != 0 {
                    w.ferr().set_bit();
                }
                w
            });
        }
        flags
    }

    /// How many received bytes were refused because their frame was broken.
    ///
    /// Print this next to the byte count: an instrument that shows only the bytes it accepted cannot tell a quiet link
    /// from a link reading noise.
    #[must_use]
    pub fn rx_errors(&self) -> u32 {
        self.rx_errors
    }

    /// Why the last refused byte was refused, if any was.
    ///
    /// A record that prints only a count leaves a reader unable to tell an overrun (bytes are missing, the count is
    /// short) from a framing error (nothing was wired, the "data" was noise).
    #[must_use]
    pub fn last_rx_problem(&self) -> Option<RxProblem> {
        self.last_rx_problem
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
