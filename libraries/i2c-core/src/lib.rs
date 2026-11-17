#![no_std]

//! The I2C transaction, separated from the registers that carry it.
//!
//! Why this exists: `write_read` used to be two independent transactions -- START, address+write, register, **STOP**, then
//! START, address+read, data, STOP. The bus is released in the middle of what is one logical transaction, so another
//! master can interleave, and a device that latches its register pointer on the write is entitled to lose it. The
//! bit-banged driver in this crate exists partly because of what that does in practice; its own header says the on-chip
//! master is not reliably "able to complete a read after a write".
//!
//! The defect could not be measured on the board: `I2c0::new` configures no pins, the board crate exposes only the
//! bit-banged bus, and no example reaches the hardware driver. So the transaction is written once here, over a `Bus`
//! trait, and measured on the **host** against a slave model that records every start, byte and stop. What that buys is
//! not a metaphor: the test below fails on any sequence that releases the bus between the write and the read, which is
//! exactly the defect, and it runs in microseconds with no hardware.

pub mod sim;

/// The errors a bus can report. Mirrors the HAL's `i2c::Error`; the HAL maps one to the other so this crate stays
/// independent of any register access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Address or data not acknowledged.
    Nack,
    /// Bus error (arbitration lost, protocol violation).
    Bus,
    /// Empty buffer.
    InvalidData,
    /// A handshake did not complete within the driver's budget.
    Timeout,
}

/// The four things an I2C master does to a bus, with no knowledge of how they are done.
pub trait Bus {
    /// Emit a (repeated) START condition and wait for it to complete.
    fn start(&mut self) -> Result<(), Error>;
    /// Send one byte; `Err(Error::Nack)` when the slave does not acknowledge it.
    fn write(&mut self, byte: u8) -> Result<(), Error>;
    /// Receive one byte, acknowledging it if `ack`.
    fn read(&mut self, ack: bool) -> Result<u8, Error>;
    /// Emit a STOP condition and wait for the bus to become idle.
    fn stop(&mut self) -> Result<(), Error>;
}

/// Write `register`, then read `out` from it, as **one** transaction: a repeated START, never a STOP in between.
pub fn write_read<B: Bus + ?Sized>(
    bus: &mut B,
    addr: u8,
    register: u8,
    out: &mut [u8],
) -> Result<(), Error> {
    if out.is_empty() {
        return Err(Error::InvalidData);
    }
    bus.start()?;
    bus.write((addr << 1) & 0xFE)?; // address, write
    bus.write(register)?;
    // Repeated START: the bus is not released here, which is the whole point of this function.
    bus.start()?;
    bus.write(((addr << 1) & 0xFE) | 1)?; // same address, read
    let last = out.len() - 1;
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = bus.read(i != last)?;
    }
    bus.stop()?;
    Ok(())
}

/// Write `bytes` to `addr` as one transaction.
pub fn write<B: Bus + ?Sized>(bus: &mut B, addr: u8, bytes: &[u8]) -> Result<(), Error> {
    if bytes.is_empty() {
        return Err(Error::InvalidData);
    }
    bus.start()?;
    bus.write((addr << 1) & 0xFE)?;
    for byte in bytes {
        bus.write(*byte)?;
    }
    bus.stop()?;
    Ok(())
}

/// Read `out` from `addr` as one transaction.
pub fn read<B: Bus + ?Sized>(bus: &mut B, addr: u8, out: &mut [u8]) -> Result<(), Error> {
    if out.is_empty() {
        return Err(Error::InvalidData);
    }
    bus.start()?;
    bus.write(((addr << 1) & 0xFE) | 1)?;
    let last = out.len() - 1;
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = bus.read(i != last)?;
    }
    bus.stop()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::sim::{MockBus, Slave};

    #[test]
    fn write_read_is_one_transaction_with_a_repeated_start() {
        let mut slave = Slave::new();
        let mut out = [0u8; 3];
        {
            let mut bus = MockBus { slave: &mut slave };
            write_read(&mut bus, 0x6A, 0x03, &mut out).expect("transfer");
        }
        // The register file starts at 0x10, so register 3 is 0x13 and the next two follow it.
        assert_eq!(
            out,
            [0x13, 0x14, 0x15],
            "read from the register the master asked for"
        );
        let trace = slave.trace_str();
        assert_eq!(
            trace, "SWWSWRRRP",
            "one START, address+write, register, repeated START, address+read, data, STOP -- no STOP in the middle"
        );
        assert!(
            !trace.contains("PWS"),
            "the bus must not be released between the register write and the read"
        );
    }

    #[test]
    fn a_second_transaction_after_the_first_reads_the_right_register() {
        // The old shape (write, STOP, read) leaves the pointer at 0 after the STOP, so the second read would return
        // register 0. This is the same test after a second call, which is when the defect shows in a driver.
        let mut slave = Slave::new();
        let mut first = [0u8; 1];
        let mut second = [0u8; 1];
        {
            let mut bus = MockBus { slave: &mut slave };
            write_read(&mut bus, 0x6A, 0x05, &mut first).unwrap();
        }
        {
            let mut bus = MockBus { slave: &mut slave };
            write_read(&mut bus, 0x6A, 0x06, &mut second).unwrap();
        }
        assert_eq!(first, [0x15]);
        assert_eq!(
            second,
            [0x16],
            "the second transfer must not inherit the first's pointer"
        );
    }

    #[test]
    fn a_nacked_address_is_reported_as_nack() {
        let mut slave = Slave::new();
        slave.nack_address = true;
        let mut out = [0u8; 1];
        let mut bus = MockBus { slave: &mut slave };
        assert_eq!(write_read(&mut bus, 0x6A, 0x00, &mut out), Err(Error::Nack));
    }

    #[test]
    fn an_empty_buffer_is_rejected_before_the_bus_is_touched() {
        let mut slave = Slave::new();
        let mut bus = MockBus { slave: &mut slave };
        assert_eq!(
            write_read(&mut bus, 0x6A, 0x00, &mut []),
            Err(Error::InvalidData)
        );
        assert_eq!(slave.trace_str(), "", "nothing was put on the bus");
    }

    #[test]
    fn plain_write_and_read_are_single_transactions() {
        let mut slave = Slave::new();
        {
            let mut bus = MockBus { slave: &mut slave };
            write(&mut bus, 0x6A, &[0x02, 0xAA]).unwrap();
            assert_eq!(
                slave.trace_str(),
                "SWWWP",
                "start, address, two bytes, stop"
            );
        }
        let mut slave2 = Slave::new();
        let mut out = [0u8; 2];
        {
            let mut bus = MockBus { slave: &mut slave2 };
            read(&mut bus, 0x6A, &mut out).unwrap();
        }
        assert_eq!(slave2.trace_str(), "SWRRP");
        assert_eq!(out, [0x10, 0x11], "a plain read starts at register 0");
    }
}
