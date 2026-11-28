//! EUSART0 in **asynchronous** (UART) mode.
//!
//! The crate's serial story was USART0 only, and the EUSART blocks were reachable only as `spi::Spi1`/`Spi2` in
//! synchronous mode. An EUSART is not a USART with extra registers: the frame lives in `FRAMECFG` instead of `FRAME`,
//! the data bits are encoded differently (`2` is eight bits here, `5` on the USART), the asynchronous divider is
//! `CLKDIV.DIV` while the synchronous one is `CFG2.SDIV`, and the block is clocked from `CMU_EUSART0CLKCTRL.CLKSEL`
//! rather than from the bus clock alone. A driver that reused the USART's register writes would have compiled and
//! produced no frames.
//!
//! ## What this driver is measured to do
//!
//! `hal_eusart_loopback` writes eight distinct bytes through `CFG0.LOOPBK` and reads them back; a control run with the
//! same writes, the same routing and only the loopback bit cleared reads zero bytes. The record is in
//! `docs/04-reference/EUSART-ASYNC-2026-11-27.md` and in the example's header.
//!
//! ## The clock, stated plainly
//!
//! Asynchronous baud is derived from `EM01GRPCCLK`, which `CMU_EUSART0CLKCTRL` selects at reset and which
//! `Board::eusart0_async` pins explicitly. On this board that branch is `HFRCODPLL` — the core clock — so the board
//! passes its own `sysclk_hz`. A product that moves SYSCLK to the crystal without moving `EM01GRPCCLK` must pass the
//! frequency that branch actually runs at; the constructor takes a number rather than reading one, so a wrong
//! assumption is a caller's visible choice and not a driver's guess.
//!
//! The divider arithmetic is `crate::cores::usart::eusart_clkdiv`, and it is **not** the USART's: the EUSART splits its
//! `CLKDIV` field into five fraction bits (`1 + DIV/32`) where the USART uses eight (`1 + DIV/256`). Measured, not
//! assumed — the first version of this driver reused the USART's function and produced 15 650 baud where 115 200 was
//! asked for (`docs/04-reference/EUSART-ASYNC-2026-11-27.md`). Both functions do their multiplication in `u64`, which is
//! what the wrap that once wrote `52` where `2382` belonged was about.

use crate::usart::{Config, DataBits, Error, Parity, RxProblem, StopBits};
use efr32mg2x_pac as pac;

/// `IF.RXOF` — the receiver ran out of room and at least one byte is gone.
pub const IF_RXOF: u32 = 1 << 4;
/// `IF.PERR` — a byte arrived with a parity error.
pub const IF_PERR: u32 = 1 << 8;
/// `IF.FERR` — a frame had no stop bit where one belonged.
pub const IF_FERR: u32 = 1 << 9;

/// The three flags that make a received byte untrustworthy.
const ERROR_FLAGS: u32 = IF_RXOF | IF_PERR | IF_FERR;

/// Oversampling the divider is computed for. `CFG0.OVS` resets to 16x and this driver leaves it there.
const OVERSAMPLE: u32 = 16;

/// One asynchronous driver, generated once per EUSART block.
///
/// `EUSART0` and `EUSART1` are the same block at two addresses, each with its own `CLKEN1` bit and clock selector, so a
/// macro is the honest shape: two concrete types, one body. The concrete `Eusart0` this file used to hold was a driver
/// tied to one instance, and `EUSART1` was listed as "compiled, not measured" for exactly that reason -- nothing could
/// build it. Measured 2026-11-27, one instance at a time.
macro_rules! impl_eusart {
    ($Name:ident, $Regs:ident) => {
        /// EUSART0 as an asynchronous serial port.
        pub struct $Name {
            eusart: pac::$Regs,
            divider: u32,
            clock_hz: u32,
            rx_errors: u32,
            last_rx_problem: Option<RxProblem>,
        }

        impl $Name {
            /// Configure EUSART0 from a peripheral clock in hertz.
            ///
            /// `clock_hz` is `EM01GRPCCLK` — the branch `CMU_EUSART0CLKCTRL` points the block at, **not** `HFCLK`. The caller
            /// must have enabled `CMU_CLKEN1.EUSART0` and selected that branch; `Board::eusart0_async` does both.
            ///
            /// Configuration happens **before** `EN` is set, for the reason this crate measured on the TIMER and the SPI path:
            /// a peripheral configured after it is enabled takes the writes on the bus and not in the state machine.
            pub fn new_at(
                eusart: pac::$Regs,
                config: Config,
                clock_hz: u32,
            ) -> Result<Self, Error> {
                let divider =
                    crate::cores::usart::eusart_clkdiv(clock_hz, config.baudrate, OVERSAMPLE)
                        .map_err(|_| Error::InvalidBaudrate)?;

                // Asynchronous mode, loopback off, and nothing else touched: `OVS`, `MSBF` and the inversions keep their reset
                // values (16x oversampling, LSB first, no inversion), which is the 8N1 the frame below asks for.
                eusart
                    .cfg0()
                    .write(|w| w.sync().async_().loopbk().clear_bit());

                eusart.framecfg().write(|w| {
                    match config.data_bits {
                        // The EUSART encodes the width directly: 2 is eight bits, 3 is nine. (The USART's `FRAME.DATABITS`
                        // uses 5 and 6 for the same two widths -- copying that encoding here is the defect this line avoids.)
                        DataBits::Eight => w.databits().eight(),
                        DataBits::Nine => w.databits().nine(),
                    };
                    match config.parity {
                        Parity::None => w.parity().none(),
                        Parity::Even => w.parity().even(),
                        Parity::Odd => w.parity().odd(),
                    };
                    match config.stop_bits {
                        StopBits::One => w.stopbits().one(),
                        StopBits::Two => w.stopbits().two(),
                    }
                });

                // SAFETY: `DIV` is 21 bits wide and `clkdiv` multiplies in `u64` and refuses a value that does not fit, so no
                // truncated number can reach the register.
                eusart.clkdiv().write(|w| unsafe { w.div().bits(divider) });
                Self::wait_divider(&eusart);

                eusart.cmd().write(|w| w.txen().set_bit().rxen().set_bit());
                eusart.en().write(|w| w.en().set_bit());

                Ok(Self {
                    eusart,
                    divider,
                    clock_hz,
                    rx_errors: 0,
                    last_rx_problem: None,
                })
            }

            /// Wait for `CLKDIV` to cross into the EUSART's own clock domain.
            ///
            /// Bounded on purpose: a `SYNCBUSY` that never clears would otherwise hang the caller with no record of where. A
            /// hundred thousand iterations is orders of magnitude more than the transfer takes, and giving up leaves the
            /// divider as written — the caller's first frame then says whether it arrived.
            fn wait_divider(eusart: &pac::$Regs) {
                let mut guard = 100_000u32;
                while eusart.syncbusy().read().div().bit_is_set() {
                    guard = guard.saturating_sub(1);
                    if guard == 0 {
                        break;
                    }
                    core::hint::spin_loop();
                }
            }

            /// Connect the receiver to this peripheral's own transmitter, inside the part.
            ///
            /// `CFG0.LOOPBK`. This is a self-test of the shifter, the frame format and the read path; it says nothing about the
            /// pad or the level on it, because nothing leaves the block.
            ///
            /// ## The enable has to come down around this write, and that is measured
            ///
            /// `CFG0` is not writable while `EN` is set on this block, and dropping `EN` is not instantaneous: `EN.DISABLING`
            /// (bit 1) stays set while the block winds down, exactly like the TIMER's `EN.DISABLING` this crate already waits
            /// on. Measured 2026-11-27: with the enable dropped and `CFG0` written on the very next bus access, the write was a
            /// **precise bus fault** — `CFSR=0x00008200` (BFARVALID | PRECISERR), `HFSR=0x40000000`, `BFAR=0x4b010008`, which
            /// is `EUSART0_S + 0x08`, the `CFG0` register. The first attempt, writing `CFG0` with the block still enabled,
            /// faulted at the same address. So the sequence is: drop `EN`, wait for `DISABLING` to clear, write the bit, raise
            /// `EN`. The wait is bounded so a block that never finishes disabling reports instead of hanging.
            pub fn set_internal_loopback(&mut self, on: bool) {
                self.eusart.en().write(|w| w.en().clear_bit());
                // Bounded: 100 000 iterations is orders of magnitude more than the wind-down takes. Both the enable and the
                // busy status are watched: `EN` can still read one while the block winds down, and the write that follows must
                // not go in until both are clear.
                let mut guard = 100_000u32;
                loop {
                    let en = self.eusart.en().read();
                    if !en.en().bit_is_set() && !en.disabling().bit_is_set() {
                        break;
                    }
                    guard = guard.saturating_sub(1);
                    if guard == 0 {
                        break;
                    }
                    core::hint::spin_loop();
                }
                self.eusart.cfg0().modify(|_, w| w.loopbk().bit(on));
                // `CMD.TXEN`/`RXEN` are re-asserted because they are the block's own enables and an `EN` cycle is exactly the
                // kind of state change that can leave them where the disable put them. Writing them again is idempotent and
                // costs two bus writes; a transmitter left disabled after the cycle would hang `write_byte` waiting for FIFO
                // room that never appears -- measured 2026-11-27, the first version that toggled `EN` without this line.
                self.eusart
                    .cmd()
                    .write(|w| w.txen().set_bit().rxen().set_bit());
                // `CMD.TXEN`/`RXEN` are commands, not configuration: they cross into the block's own clock domain and
                // `SYNCBUSY` says whether they have. Raising `EN` before they land leaves a block that looks enabled, sends
                // nothing and receives nothing -- measured on `EUSART1` on 2026-11-27, where the record read
                // `matched=0 seen=0 cfg0=0x00000002` with `LOOPBK` demonstrably set.
                Self::wait_enables(&self.eusart);
                self.eusart.en().write(|w| w.en().set_bit());
            }

            /// Wait, bounded, for the enable commands to reach the protocol domain.
            fn wait_enables(eusart: &pac::$Regs) {
                let mut guard = 100_000u32;
                while eusart.syncbusy().read().txen().bit_is_set()
                    || eusart.syncbusy().read().rxen().bit_is_set()
                {
                    guard = guard.saturating_sub(1);
                    if guard == 0 {
                        break;
                    }
                    core::hint::spin_loop();
                }
            }

            /// `STATUS` as the block has it: `RXENS`, `TXENS`, `RXFL`, `TXIDLE` and the rest, for an instrument that has to say
            /// whether the receiver was enabled rather than whether it was asked to be.
            #[must_use]
            pub fn status(&self) -> u32 {
                self.eusart.status().read().bits()
            }

            /// Write one byte, waiting until the transmitter has room for it.
            pub fn write_byte(&mut self, byte: u8) -> Result<(), crate::usart::Error> {
                // Bounded, like the USART's: a clock nobody enabled or a pad nobody routed must be an error the caller
                // can handle, not a program that never comes back.
                let mut spins = 0u32;
                while !self.eusart.status().read().txfl().bit_is_set() {
                    if spins >= crate::usart::TX_SPINS {
                        return Err(crate::usart::Error::Timeout);
                    }
                    spins += 1;
                    core::hint::spin_loop();
                }
                // SAFETY: `TXDATA` holds the frame in its low bits and `byte` is a `u8`, so the value fits whatever width the
                // frame was configured for.
                self.eusart
                    .txdata()
                    .write(|w| unsafe { w.txdata().bits(u16::from(byte)) });
                Ok(())
            }

            /// Write every byte in order.
            pub fn write(&mut self, bytes: &[u8]) -> Result<(), crate::usart::Error> {
                for &byte in bytes {
                    self.write_byte(byte)?;
                }
                Ok(())
            }

            /// Read one 8-bit frame, or `None` when the receiver holds nothing.
            ///
            /// With `DataBits::Nine` the ninth bit is dropped; [`read_frame`](Self::read_frame) is the method for that
            /// configuration.
            ///
            /// A byte is only returned when the `IF` error flags are clear at the moment it is read. The flags are latches on
            /// this block rather than a per-byte verdict (measured: see `clear_error_flags`), so a refused byte here means "an
            /// error has been seen since the last clear", never "this byte was broken". Refusing to hand it over while still
            /// counting it is the honest reading: a link that returns bytes and silently counts errors looks healthy.
            pub fn read_byte(&mut self) -> Option<Result<u8, RxProblem>> {
                let frame = self.read_frame()?;
                let flags = self.rx_flags() & ERROR_FLAGS;
                // `rx_problem` decides which flag wins when more than one is set (an overrun outranks a framing error: bytes
                // are missing and no later reading repairs the count).
                let problem = match crate::cores::usart::rx_problem(flags) {
                    Some(problem) => problem,
                    None => return Some(Ok((frame & 0x00FF) as u8)),
                };
                self.rx_errors = self.rx_errors.saturating_add(1);
                self.last_rx_problem = Some(problem);
                Some(Err(problem))
            }

            /// Read one frame as the receiver latched it: up to nine data bits.
            #[must_use]
            pub fn read_frame(&mut self) -> Option<u16> {
                if !self.eusart.status().read().rxfl().bit_is_set() {
                    return None;
                }
                Some(self.eusart.rxdata().read().rxdata().bits())
            }

            /// The receiver's error flags, straight out of `IF`.
            #[must_use]
            pub fn rx_flags(&self) -> u32 {
                self.eusart.if_().read().bits()
            }

            /// Write one to each error flag and report what it read before and after.
            ///
            /// `IF` on this block documents write-one-to-clear; the USART0 measurement (2026-11-25) found that the same write
            /// through the secure window did **not** clear its flags, so whether this block behaves as documented is a fact to
            /// be measured rather than assumed. Returning both readings lets an instrument say which happened.
            pub fn clear_error_flags(&mut self) -> (u32, u32) {
                let before = self.rx_flags();
                // SAFETY: only the three documented error bits are written, each with a one; no data or status bit is touched.
                self.eusart.if_().write(|w| unsafe { w.bits(ERROR_FLAGS) });
                (before, self.rx_flags())
            }

            /// Wait until the last frame has left the shifter.
            pub fn flush(&mut self) {
                while !self.eusart.status().read().txc().bit_is_set() {
                    core::hint::spin_loop();
                }
            }

            /// How many received bytes were refused because an error flag was set.
            #[must_use]
            pub fn rx_errors(&self) -> u32 {
                self.rx_errors
            }

            /// Why the last refused byte was refused.
            #[must_use]
            pub fn last_rx_problem(&self) -> Option<RxProblem> {
                self.last_rx_problem
            }

            /// `CFG0` as the block has it, for an instrument that needs to prove a bit landed.
            ///
            /// `LOOPBK` is the bit that matters here: a run that writes it and then reads back a different value is measuring
            /// the write, not the loopback.
            #[must_use]
            pub fn cfg0(&self) -> u32 {
                self.eusart.cfg0().read().bits()
            }

            /// The value written to `CLKDIV.DIV`.
            #[must_use]
            pub fn divider(&self) -> u32 {
                self.divider
            }

            /// The rate the divider produces from the clock this instance was built with.
            #[must_use]
            pub fn actual_baud(&self) -> u32 {
                crate::cores::usart::eusart_actual_baud(self.clock_hz, self.divider, OVERSAMPLE)
            }
        }
    };
}

impl_eusart!(Eusart0, Eusart0S);
impl_eusart!(Eusart1, Eusart1S);
