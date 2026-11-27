//! I2C Master Mode Driver
//!
//! # What works on this die, and what used to be written here
//!
//! This header used to say "this driver does not work on this die", from the fourteen measurements in
//! `docs/04-reference/I2C0-READ-DEFECT.md`. Two of those measurements were a driver defect, not a silicon one: the
//! last byte of a reception was read with ACK instead of NACK, so the follower never stopped driving and the master
//! never closed the reception; and the error flags were read through the non-secure view, which reads zero. With both
//! fixed, `hal_i2c_hardware_imu` reads `WHO_AM_I = 0x6a` from the IMU over I2C0 with no help from the bit-banged bus,
//! and `hal_i2c_recovery` completes a read-after-write transaction on the same path. Until 2026-11-17 nothing here
//! could reach a pin at all (`I2c0::new` configured no routing) and no measurement of this driver was possible.
//!
//! What is still open is named where it lives: a bus that a *follower* holds low needs clocks on SCL to be released,
//! and no instrument in this repository produces that condition yet (`docs/04-reference/I2C-RECOVERY-STATE.md`).
//!
//! # Hardware Registers
//!
//! - **EN**: Enable register
//! - **CTRL**: Control register (master/slave mode, clock settings)
//! - **CMD**: Command register (START, STOP, ACK, NACK, ABORT, CLEARTX, CLEARPC)
//! - **STATE**: state machine, `BUSY`/`BUSHOLD`, used to wait for the bus to be ours
//! - **STATUS**: handshake status (PSTART, PSTOP, TXBL, TXC, RXDATAV)
//! - **IF**: sticky interrupt flags -- read through the **secure** window, cleared through **`IF_CLR`**
//! - **CLKDIV**: Clock divider for SCL frequency
//! - **TXDATA** / **RXDATA**: the byte registers
//!
//! # Features
//!
//! - I2C master mode with 7-bit addressing
//! - Configurable SCL frequency (standard 100 kHz, fast 400 kHz)
//! - `write`, `read` and `write_read`, the last one as **one** transaction with a repeated START
//! - embedded-hal v1.0 I2C traits; `embedded-hal-async` for I2C0
//!
//! # Example
//!
//! ```no_run
//! use efr32mg2x_hal::{
//!     clock::{Clocks, ClockConfig},
//!     i2c::{I2c0, Config, Speed},
//!     pac,
//! };
//!
//! let dp = pac::Peripherals::take().unwrap();
//! let clocks = Clocks::new(dp.cmu_s, ClockConfig { hfxo: None, lfxo: None }).freeze();
//! let mut i2c = I2c0::new(dp.i2c0_ns, Config::new(Speed::Standard100kHz), clocks.hfclk());
//!
//! i2c.write(0x6A, &[0x75]).unwrap();
//! let mut buffer = [0u8; 1];
//! i2c.read(0x6A, &mut buffer).unwrap();
//! ```

mod async_impl;

pub use async_impl::{last_state, on_interrupt, waits};
mod traits;
mod types;

pub use crate::cores::i2c as proto;
pub use types::{Config, Error, Speed};

use crate::clock::FrozenClocks;
use crate::pac;

/// Polls a handshake may take before the driver reports `Timeout`.
///
/// At 19 MHz a poll is a handful of cycles, so this is tens of milliseconds; a 100 kHz byte time is 90 microseconds.
/// Three orders of magnitude of slack: no correct transfer of a few bytes comes close, and nothing on a wedged bus can
/// hang the program for longer than this.
const WAIT_POLLS: u32 = 200_000;

/// The hardware registers, as the transaction layer sees them.
///
/// This is the whole seam: `efr32mg2x-i2c-core` decides *what* a repeated START looks like and is measured on the
/// host; this type only performs the four primitives on the peripheral. A change in one cannot silently change the
/// other, which is what made the old two-transaction `write_read` survive in a driver nobody could exercise.
struct HwBus<'a, P> {
    i2c: &'a P,
}

/// The primitives, written once for both I2C instances.
///
/// `$seguro` is the instance's **secure** window. Two things depend on it and both were measured: `IF` read through
/// the non-secure view always reads zero, and the clear-alias window (`+0x2000`) has to be computed from the secure
/// base.
macro_rules! impl_hw_bus {
    ($ty:ty, $seguro:ty) => {
        impl HwBus<'_, $ty> {
            /// Whether this instance has a NACK pending, read through its own secure window.
            fn nack_pendente(&self) -> bool {
                // SAFETY: a read of the block the caller already owns, through its secure window.
                let janela = unsafe { &*<$seguro>::ptr() };
                janela.if_().read().nack().bit_is_set()
            }

            /// Clear `NACK` through the **clear-alias** window.
            ///
            /// `IF` is RW on this part and its flags are sticky: the manual says each one "needs to be manually
            /// cleared by software" (RM 22.3.8.2), and the vendor clears them by writing `IF_CLR`
            /// (`I2C_IntClear`, `em_i2c.h:295-300`). Writing 1 to the plain register does not clear a flag here, so
            /// the NACK of one transaction was still set when the next one began -- and `write` checks the flag
            /// *before* it loads the byte, to tell a refusal from a timeout, so the next transaction was refused by
            /// the previous refusal without touching the bus. Measured: `hal_i2c_recovery`,
            /// `antes=0x6a erro=1 depois=0x00`.
            fn limpar_nack(&self) {
                // SAFETY: the clear window of this instance's block; writing a flag bit has no other effect.
                unsafe { crate::alias::clear_window(<$seguro>::ptr()) }
                    .if_()
                    .write(|w| w.nack().set_bit());
            }

            /// Clear every sticky flag of this instance through the clear window.
            fn limpar_flags(&self) {
                // SAFETY: as above -- one write to the clear-alias window of a block the caller already owns.
                unsafe { crate::alias::clear_window(<$seguro>::ptr()) }
                    .if_()
                    .write(|w| {
                        w.start()
                            .set_bit()
                            .rstart()
                            .set_bit()
                            .addr()
                            .set_bit()
                            .txc()
                            .set_bit()
                            .txbl()
                            .set_bit()
                            .rxdatav()
                            .set_bit()
                            .ack()
                            .set_bit()
                            .nack()
                            .set_bit()
                            .mstop()
                            .set_bit()
                            .arblost()
                            .set_bit()
                            .buserr()
                            .set_bit()
                            .bushold()
                            .set_bit()
                            .txof()
                            .set_bit()
                            .rxuf()
                            .set_bit()
                            .bito()
                            .set_bit()
                            .clto()
                            .set_bit()
                            .sstop()
                            .set_bit()
                            .rxfull()
                            .set_bit()
                            .clerr()
                            .set_bit()
                            .sclerr()
                            .set_bit()
                            .sdaerr()
                            .set_bit()
                    });
            }
        }

        impl crate::cores::i2c::Bus for HwBus<'_, $ty> {
            fn start(&mut self) -> Result<(), crate::cores::i2c::Error> {
                use crate::cores::i2c::Error as Proto;
                self.i2c.cmd().write(|w| w.start().set_bit());
                let mut polls = 0u32;
                while self.i2c.status().read().pstart().bit_is_set() {
                    polls += 1;
                    if polls > WAIT_POLLS {
                        return Err(Proto::Timeout);
                    }
                }
                Ok(())
            }

            fn write(&mut self, byte: u8) -> Result<(), crate::cores::i2c::Error> {
                use crate::cores::i2c::Error as Proto;
                let mut polls = 0u32;
                while self.i2c.status().read().txbl().bit_is_clear() {
                    polls += 1;
                    if polls > WAIT_POLLS {
                        return Err(Proto::Timeout);
                    }
                }
                // SAFETY: `TXDATA` takes the byte being shifted onto the bus; `byte` is a `u8`, so the widening adds no bits.
                self.i2c.txdata().write(|w| unsafe { w.bits(byte as u32) });
                let mut polls = 0u32;
                while self.i2c.status().read().txc().bit_is_clear() {
                    // Uma recusa chega ANTES de `txc`: o mestre para e o byte nao "sai". Esperar so' por `txc`
                    // transforma um NACK em Timeout -- medido, com o dispositivo ausente -- e perde a unica coisa
                    // que distingue "nao ha ninguem ai" de "esta' lento".
                    if self.nack_pendente() {
                        self.limpar_nack();
                        return Err(Proto::Nack);
                    }
                    polls += 1;
                    if polls > WAIT_POLLS {
                        return Err(Proto::Timeout);
                    }
                }
                // Read the flag through the secure window -- through the non-secure view it is always zero, which
                // is how a refusal was reported as a timeout later on. The clear goes through `IF_CLR`.
                if self.nack_pendente() {
                    self.limpar_nack();
                    return Err(Proto::Nack);
                }
                Ok(())
            }

            fn read(&mut self, ack: bool) -> Result<u8, crate::cores::i2c::Error> {
                use crate::cores::i2c::Error as Proto;
                let mut polls = 0u32;
                while self.i2c.status().read().rxdatav().bit_is_clear() {
                    polls += 1;
                    if polls > WAIT_POLLS {
                        return Err(Proto::Timeout);
                    }
                }
                if ack {
                    self.i2c.cmd().write(|w| w.ack().set_bit());
                } else {
                    self.i2c.cmd().write(|w| w.nack().set_bit());
                }
                Ok(self.i2c.rxdata().read().bits() as u8)
            }

            fn stop(&mut self) -> Result<(), crate::cores::i2c::Error> {
                use crate::cores::i2c::Error as Proto;
                self.i2c.cmd().write(|w| w.stop().set_bit());
                let mut polls = 0u32;
                while self.i2c.status().read().pstop().bit_is_set() {
                    polls += 1;
                    if polls > WAIT_POLLS {
                        return Err(Proto::Timeout);
                    }
                }
                Ok(())
            }

            /// Release the bus after a failed transaction.
            ///
            /// The manual gives the order, and each step below has a reason that was read or measured:
            ///
            /// 1. **`ABORT` with `STOP` in one command.** "Abort the current transmission making the bus go idle.
            ///    When used in combination with STOP, a STOP condition is sent as soon as possible before aborting"
            ///    (RM 22.5.4). The previous version wrote `ABORT`, waited, and only then wrote `STOP` -- a different
            ///    sequence, in which the STOP arrives when there is no transfer left to stop.
            /// 2. **Wait for `STATE.BUSY` to clear**, which is what says the module is off the bus.
            /// 3. **`CLEARPC`**: a command left pending by the refused transaction (an ACK, a STOP, a START) is
            ///    otherwise consumed by the next one.
            /// 4. **`CLEARTX`**: "clear transmit buffer and shift register". Without it the address byte the
            ///    follower refused is still queued when the next transaction starts.
            /// 5. **Clear the sticky flags** through `IF_CLR`. Leaving `NACK` set is what made the third transaction
            ///    of `hal_i2c_recovery` fail without touching the bus.
            fn abort(&mut self) {
                self.i2c
                    .cmd()
                    .write(|w| w.abort().set_bit().stop().set_bit());
                let mut polls = 0u32;
                while self.i2c.state().read().busy().bit_is_set() {
                    polls += 1;
                    if polls > WAIT_POLLS {
                        break;
                    }
                }
                self.i2c.cmd().write(|w| w.clearpc().set_bit());
                self.i2c.cmd().write(|w| w.cleartx().set_bit());
                self.limpar_flags();
            }
        }
    };
}
impl_hw_bus!(pac::I2c0Ns, pac::I2c0S);
impl_hw_bus!(pac::I2c1S, pac::I2c1S);

/// I2C0 peripheral instance
pub struct I2c0 {
    i2c: pac::I2c0Ns,
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
    /// * `hfclk` - the HFCLK frequency the divider is computed from
    ///
    /// # Example
    ///
    /// ```no_run
    /// let i2c = I2c0::new(dp.i2c0_ns, Config::new(Speed::Standard100kHz), clocks.hfclk());
    /// ```
    pub fn new(i2c: pac::I2c0Ns, config: Config, hfclk: crate::clock::Hertz) -> Self {
        // The clock itself was enabled by whoever owns the CMU: this driver only needs the number,
        // to divide it down to the bus rate. Taking `&FrozenClocks` here made the driver a second
        // owner of the CMU, and the CMU can only have one -- which is why no `Board` could ever
        // build an `I2c0`, and why the product fell back to bit-banging.

        // Enable I2C peripheral
        i2c.en().write(|w| w.en().set_bit());

        // Configure as master mode
        i2c.ctrl().write(|w| {
            w.slave()
                .clear_bit() // Master mode
                .autoack()
                .clear_bit() // Manual ACK/NACK
                .autosn()
                .set_bit() // Automatic STOP/START
        });

        // Calculate and set clock divider
        let clkdiv = Self::calculate_clkdiv(hfclk.into(), config.speed);
        // SAFETY: `CLKDIV` is the whole divider word, produced by `cores::i2c::clkdiv`, which is host-tested against the
        // manual's formula and refuses a rate below the bus minimum.
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
        // One transaction, from the layer that has host tests over the sequence -- and that releases the bus
        // through `abort()` on every error path. The inline machine this replaced returned with START still
        // asserted, which is how one refusal left the bus unusable (`hal_i2c_recovery`).
        crate::cores::i2c::write(&mut HwBus { i2c: &self.i2c }, addr, bytes).map_err(Error::from)
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
        crate::cores::i2c::read(&mut HwBus { i2c: &self.i2c }, addr, buffer).map_err(Error::from)
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
        // One transaction with a repeated START, from the crate whose host tests measure exactly that. The old body
        // here called `write` and then `read`, which released the bus in the middle and let a device drop the
        // register pointer it had just been given.
        crate::cores::i2c::write_read(&mut HwBus { i2c: &self.i2c }, addr, register, buffer)
            .map_err(Error::from)
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
            w.slave()
                .clear_bit()
                .autoack()
                .clear_bit()
                .autosn()
                .set_bit()
        });

        // Calculate and set clock divider
        let clkdiv = Self::calculate_clkdiv(clocks.hfclk().into(), config.speed);
        // SAFETY: as above -- the same host-tested divider, for the other bus instance.
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
    pub fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
        crate::cores::i2c::write(&mut HwBus { i2c: &self.i2c }, addr, bytes).map_err(Error::from)
    }

    /// Read data from I2C device
    pub fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Error> {
        crate::cores::i2c::read(&mut HwBus { i2c: &self.i2c }, addr, buffer).map_err(Error::from)
    }

    /// Write register then read from I2C device
    ///
    /// This was two transactions (write, STOP, read) until the core took it over: the bus was released in the
    /// middle, and a device that latches its register pointer on the write is entitled to lose it.
    pub fn write_read(&mut self, addr: u8, register: u8, buffer: &mut [u8]) -> Result<(), Error> {
        crate::cores::i2c::write_read(&mut HwBus { i2c: &self.i2c }, addr, register, buffer)
            .map_err(Error::from)
    }
}
