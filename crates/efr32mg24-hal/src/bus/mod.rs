//! Transport-agnostic I2C master.
//!
//! Drivers (display, sensors, RTC) talk to this trait, never to a peripheral.
//! That is deliberate: the on-chip I2C0 master on this die does not complete a
//! read after a write (`docs/04-reference/I2C0-READ-DEFECT.md`), so today the
//! only working transport is the GPIO bit-banger. When I2C0 is understood, a
//! second implementation of this trait is all that changes — the drivers and
//! every example above them stay untouched.
//!
//! The trait is deliberately plain Rust, not `embedded-hal`'s `I2c`: the
//! register-level helpers (`read_reg`, `read_regs`) are what sensor drivers
//! actually need, and an `embedded-hal` adapter can be written on top of this
//! in a few lines when a crate requires the standard traits.

pub mod bitbang;

pub use bitbang::{BitBangI2c, Wiring};

use core::cell::RefCell;

/// A bus shared between drivers.
///
/// A panel and a sensor live on the same two wires, and each driver wants to
/// own "the bus" for its own calls. Rather than make every driver take the bus
/// per call, hand them a `&RefCell<B>`: borrows are checked at runtime (and the
/// drivers never nest them), so two drivers can coexist without the borrow
/// checker forbidding the obvious program.
pub struct SharedBus<'a, B: I2cBus> {
    cell: &'a RefCell<B>,
}

impl<'a, B: I2cBus> SharedBus<'a, B> {
    /// Wrap a bus cell for handing to a driver.
    pub fn new(cell: &'a RefCell<B>) -> Self {
        Self { cell }
    }
}

impl<B: I2cBus> I2cBus for SharedBus<'_, B> {
    fn probe(&mut self, addr: u8) -> bool {
        self.cell.borrow_mut().probe(addr)
    }
    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), BusError> {
        self.cell.borrow_mut().write(addr, bytes)
    }
    fn read(&mut self, addr: u8, out: &mut [u8]) -> Result<(), BusError> {
        self.cell.borrow_mut().read(addr, out)
    }
    fn write_read(&mut self, addr: u8, prefix: &[u8], out: &mut [u8]) -> Result<(), BusError> {
        self.cell.borrow_mut().write_read(addr, prefix, out)
    }
    fn write_reg(&mut self, addr: u8, reg: u8, value: u8) -> Result<(), BusError> {
        self.cell.borrow_mut().write_reg(addr, reg, value)
    }
    fn read_reg(&mut self, addr: u8, reg: u8) -> Result<u8, BusError> {
        self.cell.borrow_mut().read_reg(addr, reg)
    }
    fn read_regs(&mut self, addr: u8, reg: u8, out: &mut [u8]) -> Result<(), BusError> {
        self.cell.borrow_mut().read_regs(addr, reg, out)
    }
}

/// Errors a bus implementation can report.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BusError {
    /// The device did not acknowledge its address.
    NoAck,
    /// The device acknowledged the address but not a data byte.
    DataNack,
    /// The bus did not respond in the expected way (start/stop conditions,
    /// arbitration, timeouts).
    Bus,
    /// The implementation cannot do this operation at all — used by the I2C0
    /// backend while its read path is broken, so a driver fails loudly instead
    /// of returning invented bytes.
    Unsupported,
}

/// A 7-bit-addressed I2C master.
pub trait I2cBus {
    /// Does anything answer at `addr`?
    fn probe(&mut self, addr: u8) -> bool;

    /// Write `bytes` to `addr` (START, address+W, data, STOP).
    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), BusError>;

    /// Read into `out` from `addr` (START, address+R, data, STOP).
    fn read(&mut self, addr: u8, out: &mut [u8]) -> Result<(), BusError>;

    /// Write `prefix`, repeated START, then read into `out`.
    fn write_read(&mut self, addr: u8, prefix: &[u8], out: &mut [u8]) -> Result<(), BusError>;

    /// Write one 8-bit register.
    fn write_reg(&mut self, addr: u8, reg: u8, value: u8) -> Result<(), BusError> {
        self.write(addr, &[reg, value])
    }

    /// Read one 8-bit register.
    fn read_reg(&mut self, addr: u8, reg: u8) -> Result<u8, BusError> {
        let mut b = [0u8; 1];
        self.write_read(addr, &[reg], &mut b)?;
        Ok(b[0])
    }

    /// Read a run of registers starting at `reg`.
    fn read_regs(&mut self, addr: u8, reg: u8, out: &mut [u8]) -> Result<(), BusError> {
        self.write_read(addr, &[reg], out)
    }
}

/// Let a driver take `&mut bus` instead of consuming the bus, so one transport
/// can serve a panel and a sensor in the same loop.
impl<T: I2cBus + ?Sized> I2cBus for &mut T {
    fn probe(&mut self, addr: u8) -> bool {
        (**self).probe(addr)
    }
    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), BusError> {
        (**self).write(addr, bytes)
    }
    fn read(&mut self, addr: u8, out: &mut [u8]) -> Result<(), BusError> {
        (**self).read(addr, out)
    }
    fn write_read(&mut self, addr: u8, prefix: &[u8], out: &mut [u8]) -> Result<(), BusError> {
        (**self).write_read(addr, prefix, out)
    }
    fn write_reg(&mut self, addr: u8, reg: u8, value: u8) -> Result<(), BusError> {
        (**self).write_reg(addr, reg, value)
    }
    fn read_reg(&mut self, addr: u8, reg: u8) -> Result<u8, BusError> {
        (**self).read_reg(addr, reg)
    }
    fn read_regs(&mut self, addr: u8, reg: u8, out: &mut [u8]) -> Result<(), BusError> {
        (**self).read_regs(addr, reg, out)
    }
}
