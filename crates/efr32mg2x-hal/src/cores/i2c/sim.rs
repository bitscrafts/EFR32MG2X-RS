//! A slave model and a bus that drives it, for measuring the transaction without hardware.
//!
//! The model behaves like a real device in the one way this crate cares about: it latches a register pointer when the
//! master writes one, and **loses it when the bus is released**. A master that emits STOP between the write and the read
//! therefore reads from register 0, which is exactly the failure the HAL's old `write_read` had.

use super::{write_read, Bus, Error};

/// A slave that keeps a register file and records the conversation.
///
/// It behaves like a real device in the one way that matters here: the register pointer is latched by the write and
/// is *lost* when the bus is released, so a master that emits STOP between the write and the read reads from
/// register 0 instead of the one it asked for. That is the symptom, not a rule invented for the test.
pub struct Slave {
    /// The register file, eight bytes initialised to `0x10 + index` so a wrong pointer reads a wrong value.
    pub registers: [u8; 8],
    /// The register the next read comes from; latched by a write, reset by STOP.
    pub pointer: u8,
    /// Whether the master's last address byte selected the read phase.
    pub read_phase: bool,
    /// Everything the master did, in order: 'S' start, 'W' byte written, 'R' byte read, 'P' stop.
    pub trace: [u8; 32],
    /// How many bytes of `trace` were written.
    pub trace_len: usize,
    /// Whether the current transaction is still addressed to this slave.
    pub addressed: bool,
    /// Set when a START arrives on a bus that was released: the pointer restart a real device does.
    pub pointer_after_stop: u8,
    /// When set, the slave refuses the next address byte with `Error::Nack`.
    pub nack_address: bool,
}

impl Default for Slave {
    fn default() -> Self {
        Self::new()
    }
}

impl Slave {
    /// A slave with the default register file (`0x10..0x17`), pointer 0, and no NACK armed.
    pub fn new() -> Self {
        let mut registers = [0u8; 8];
        for (i, r) in registers.iter_mut().enumerate() {
            *r = 0x10 + i as u8;
        }
        Self {
            registers,
            pointer: 0,
            read_phase: false,
            trace: [b' '; 32],
            trace_len: 0,
            addressed: false,
            pointer_after_stop: 0,
            nack_address: false,
        }
    }

    fn record(&mut self, c: u8) {
        if self.trace_len < self.trace.len() {
            self.trace[self.trace_len] = c;
            self.trace_len += 1;
        }
    }

    /// The recorded trace as text, for comparing against a recording: `"SWWRSRW..."`, `"?"` if it is not valid UTF-8.
    pub fn trace_str(&self) -> &str {
        core::str::from_utf8(&self.trace[..self.trace_len]).unwrap_or("?")
    }
}

/// The master side of the bus, driven by the slave above.
pub struct MockBus<'a> {
    /// The slave this master addresses; every operation is recorded there.
    pub slave: &'a mut Slave,
}

impl Bus for MockBus<'_> {
    fn start(&mut self) -> Result<(), Error> {
        self.slave.record(b'S');
        // A START releases the pointer a real device latched before the previous STOP.
        self.slave.addressed = false;
        self.slave.read_phase = false;
        Ok(())
    }

    fn write(&mut self, byte: u8) -> Result<(), Error> {
        self.slave.record(b'W');
        if !self.slave.addressed {
            if self.slave.nack_address {
                return Err(Error::Nack);
            }
            self.slave.read_phase = byte & 1 == 1;
            self.slave.addressed = true;
            return Ok(());
        }
        if self.slave.read_phase {
            // A write during the read phase is a protocol error on a real bus.
            return Err(Error::Bus);
        }
        self.slave.pointer = byte & 0x07;
        self.slave.pointer_after_stop = self.slave.pointer;
        Ok(())
    }

    fn read(&mut self, ack: bool) -> Result<u8, Error> {
        self.slave.record(b'R');
        if !self.slave.addressed || !self.slave.read_phase {
            return Err(Error::Bus);
        }
        let value = self.slave.registers[self.slave.pointer as usize];
        self.slave.pointer = (self.slave.pointer + 1) & 0x07;
        let _ = ack; // the slave model does not need the ACK to answer; the master's NACK ends the transfer
        Ok(value)
    }

    fn stop(&mut self) -> Result<(), Error> {
        self.slave.record(b'P');
        self.slave.addressed = false;
        self.slave.read_phase = false;
        // The defect's mechanism: a device that sees STOP drops the pointer it was given.
        self.slave.pointer = 0;
        Ok(())
    }
}

/// The outcome of one simulated exchange: the bus events as text, the bytes read, and whether it succeeded.
pub struct SimResult {
    /// `S` start, `W` byte written, `R` byte read, `P` stop, in order.
    pub trace_tail: [u8; 32],
    /// How many bytes of `trace_tail` were written.
    pub trace_len: usize,
    /// The bytes the master read, in order.
    pub values: [u8; 8],
    /// How many of `values` were read; `count` minus this is what the transfer did not deliver.
    pub count: usize,
    /// Whether the whole exchange returned `Ok`.
    pub ok: bool,
}

/// Run `write_read` against the slave model and report what happened on the bus.
///
/// This is the measurement that does not need a board: the same function the driver calls, over a model that loses its
/// register pointer on STOP exactly as a real device does.
pub fn simulate_write_read(addr: u8, register: u8, count: usize) -> SimResult {
    let mut slave = sim_slave();
    let mut values = [0u8; 8];
    let count = count.min(values.len());
    let ok = {
        let mut bus = sim_bus(&mut slave);
        write_read(&mut bus, addr, register, &mut values[..count]).is_ok()
    };
    let mut out = SimResult {
        trace_tail: [0; 32],
        trace_len: slave.trace_len,
        values,
        count,
        ok,
    };
    out.trace_tail[..slave.trace_len].copy_from_slice(&slave.trace[..slave.trace_len]);
    out
}

impl SimResult {
    /// The bus events as text (empty on a malformed trace).
    pub fn trace(&self) -> &str {
        core::str::from_utf8(&self.trace_tail[..self.trace_len]).unwrap_or("?")
    }
}

fn sim_slave() -> Slave {
    Slave::new()
}

fn sim_bus(slave: &mut Slave) -> MockBus<'_> {
    MockBus { slave }
}
