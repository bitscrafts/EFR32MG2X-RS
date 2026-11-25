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

// The mock bus and the simulated follower are for the tests below, not for callers: a test double in a published API
// is a promise nobody wants to keep, and it was the last thing keeping `deny(missing_docs)` off this module.
#[cfg(test)]
mod sim;

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
    /// Release the bus after a failed transaction.
    ///
    /// Every `?` above returns with the master still holding the bus: a NACK on the address leaves START
    /// asserted and no STOP follows. On this peripheral that is not a tidy failure -- it is a bus that no longer
    /// answers, measured on the board (`hal_i2c_recovery`: one refusal and the next read of a device that *is*
    /// there fails). A default no-op keeps the trait usable by a bus that has nothing to abort.
    fn abort(&mut self) {}
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
    let resultado = (|| {
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
        bus.stop()
    })();
    if resultado.is_err() {
        // A refusal is not the end of the transaction, it is the end of the bus -- unless somebody releases it.
        bus.abort();
    }
    resultado
}

/// Write `bytes` to `addr` as one transaction.
pub fn write<B: Bus + ?Sized>(bus: &mut B, addr: u8, bytes: &[u8]) -> Result<(), Error> {
    if bytes.is_empty() {
        return Err(Error::InvalidData);
    }
    let resultado = (|| {
        bus.start()?;
        bus.write((addr << 1) & 0xFE)?;
        for byte in bytes {
            bus.write(*byte)?;
        }
        bus.stop()
    })();
    if resultado.is_err() {
        bus.abort();
    }
    resultado
}

/// Read `out` from `addr` as one transaction.
pub fn read<B: Bus + ?Sized>(bus: &mut B, addr: u8, out: &mut [u8]) -> Result<(), Error> {
    if out.is_empty() {
        return Err(Error::InvalidData);
    }
    let resultado = (|| {
        bus.start()?;
        bus.write(((addr << 1) & 0xFE) | 1)?;
        let last = out.len() - 1;
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = bus.read(i != last)?;
        }
        bus.stop()
    })();
    if resultado.is_err() {
        bus.abort();
    }
    resultado
}

#[cfg(test)]
mod tests {
    extern crate std;
    use self::sim::{MockBus, Slave};
    use super::*;

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

    /// The control: the same mock bus, the old two-transaction sequence, and the wrong bytes.
    ///
    /// This is what makes the test above a check rather than a tautology. If the mock could not tell the two sequences
    /// apart, `write_read_is_one_transaction_with_a_repeated_start` would pass either way and the STOP-in-the-middle
    /// defect would come back unnoticed. Until 2026-11-17 this control was produced by reverting the code by hand, so the
    /// evidence lived in a terminal and could not be re-run.
    #[test]
    fn the_control_takes_the_wrong_path() {
        // The correct sequence: one transaction, repeated START.
        let mut good = Slave::new();
        let mut out_good = [0u8; 3];
        {
            let mut bus = MockBus { slave: &mut good };
            write_read(&mut bus, 0x6A, 0x03, &mut out_good).expect("a good read");
        }

        // The old sequence: write, STOP, read.
        let mut bad = Slave::new();
        let mut out_bad = [0u8; 3];
        {
            let mut bus = MockBus { slave: &mut bad };
            write_read_the_defective_way(&mut bus, 0x6A, 0x03, &mut out_bad)
                .expect("the control runs");
        }

        assert_eq!(
            good.trace_str(),
            "SWWSWRRRP",
            "one transaction, repeated START"
        );
        assert_eq!(
            bad.trace_str(),
            "SWWPSWRRRP",
            "the control emits a STOP between the write and the read"
        );
        assert_ne!(
            out_good, out_bad,
            "if these were equal the mock would not distinguish the defect from the fix"
        );
        assert_eq!(
            out_bad,
            [0x10, 0x11, 0x12],
            "the control reads register 0, because the pointer was lost at the STOP"
        );
    }
}
/// The old `write_read`, kept as a control so the test that catches it can be shown to fail.
///
/// The HAL's `write_read` used to be `write(register)` followed by `read(buffer)` -- two transactions, so the bus was
/// released between them and the slave lost the register pointer it had just been given. Measured on the board before the
/// fix: reading after a write returned register 0's bytes.
///
/// This function is **not** wired to anything. It exists so that the property tested above has a near miss that must
/// take the wrong path: on 2026-11-17 the control was produced by reverting the code by hand, which meant the evidence
/// lived in a terminal and could not be re-run. A control that cannot be re-run is a claim, not a measurement.
#[cfg(test)]
pub fn write_read_the_defective_way<B: Bus + ?Sized>(
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
    // The defect: the bus is released here, so the slave forgets the register pointer it was just given.
    bus.stop()?;
    bus.start()?;
    bus.write(((addr << 1) & 0xFE) | 1)?; // same address, read -- of register 0
    let last = out.len() - 1;
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = bus.read(i != last)?;
    }
    bus.stop()?;
    Ok(())
}
