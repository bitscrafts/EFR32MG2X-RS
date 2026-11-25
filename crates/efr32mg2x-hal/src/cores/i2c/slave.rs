//! The I2C slave's state machine, as arithmetic: address match, direction, and the three things it can answer.
//!
//! A slave cannot start anything. It is addressed, and from then on it either receives bytes it must store or hands out
//! bytes it was given — and every byte, in both directions, ends with an acknowledge or a refusal that the master sees.
//! Three decisions live here, and each has a wrong answer that looks like a working bus:
//!
//! 1. **The address is seven bits and the direction is the eighth.** `0x6A` written and `0x6A` read are `0xD4` and `0xD5`
//!    on the wire, and a slave that compares the whole byte matches only one of them.
//! 2. **A full receive buffer is refused, not overwritten.** The master gets the NACK and knows the byte was lost; without
//!    it the master believes everything arrived, which is the failure mode a protocol cannot recover from.
//! 3. **An empty transmit buffer stretches the clock; it does not invent a byte.** Handing back `0xFF` makes the master
//!    read a value that was never written, and there is no way for it to tell.
//!
//! Not modelled, deliberately: 10-bit addresses, general call, arbitration, and the timing of the stretch (which is a
//! hardware handshake; here it is a value the caller can act on).

/// What the slave answers to the byte it was just handed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Response {
    /// Acknowledge: the byte was taken.
    Ack,
    /// Refuse: the byte was not taken, and the master will see it.
    Nack,
    /// Hold the clock line until there is something to send.
    ///
    /// Only a read can produce this. A slave that answers `Nack` on a read or sends a filler byte instead has decided
    /// something the master cannot see.
    Stretch,
}

/// Which half of a transaction the slave is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// No transaction in progress.
    Idle,
    /// Addressed for writing: the master sends, the slave stores.
    Receiving,
    /// Addressed for reading: the master reads, the slave hands out what it was given.
    Transmitting,
}

/// How many bytes a slave holds in each direction. Small on purpose: the depth is a decision about interrupt latency, and
/// four is enough to make the full-buffer path testable without pretending to be a driver's buffer.
pub const DEPTH: usize = 4;

/// An I2C slave's state: one address, one buffer per direction, and the rules above.
#[derive(Debug, Clone, Copy)]
pub struct Slave {
    address: u8,
    direction: Direction,
    rx: [u8; DEPTH],
    rx_len: usize,
    tx: [u8; DEPTH],
    tx_len: usize,
    tx_pos: usize,
}

impl Slave {
    /// A slave that answers to `address` (seven bits; the top bit is ignored).
    #[must_use]
    pub const fn new(address: u8) -> Self {
        Self {
            address: address & 0x7F,
            direction: Direction::Idle,
            rx: [0; DEPTH],
            rx_len: 0,
            tx: [0; DEPTH],
            tx_len: 0,
            tx_pos: 0,
        }
    }

    /// The address this slave answers to.
    #[must_use]
    pub fn address(&self) -> u8 {
        self.address
    }

    /// Which half of a transaction is in progress.
    #[must_use]
    pub fn direction(&self) -> Direction {
        self.direction
    }

    /// A START condition arrived: the previous transaction, if any, is over.
    pub fn start(&mut self) {
        self.direction = Direction::Idle;
        self.rx_len = 0;
        self.tx_pos = 0;
    }

    /// The master sent the address byte. Returns whether this slave is the one being addressed.
    ///
    /// The direction is taken from bit 0 and the comparison uses bits 7..1, which is the difference between matching one
    /// direction of a device and matching the device.
    pub fn address_byte(&mut self, byte: u8) -> Response {
        if (byte >> 1) != self.address {
            self.direction = Direction::Idle;
            return Response::Nack;
        }
        self.rx_len = 0;
        self.tx_pos = 0;
        self.direction = if byte & 1 == 0 {
            Direction::Receiving
        } else {
            Direction::Transmitting
        };
        Response::Ack
    }

    /// A data byte arrived while receiving.
    ///
    /// A full buffer is a NACK, and the bytes already stored stay stored: the master is told this one was lost, which is
    /// the only way it can retry.
    pub fn receive(&mut self, byte: u8) -> Response {
        if self.direction != Direction::Receiving || self.rx_len >= DEPTH {
            return Response::Nack;
        }
        self.rx[self.rx_len] = byte;
        self.rx_len += 1;
        Response::Ack
    }

    /// The bytes received in this transaction.
    #[must_use]
    pub fn received(&self) -> &[u8] {
        &self.rx[..self.rx_len]
    }

    /// Queue bytes for a subsequent read, replacing whatever was queued before.
    pub fn queue(&mut self, bytes: &[u8]) {
        let n = bytes.len().min(DEPTH);
        self.tx[..n].copy_from_slice(&bytes[..n]);
        self.tx_len = n;
        self.tx_pos = 0;
    }

    /// A byte is requested by the master, with `ack` false on the last one it intends to read.
    ///
    /// `Err(Response::Stretch)` when nothing is queued -- never a filler byte.
    pub fn transmit(&mut self) -> Result<u8, Response> {
        if self.direction != Direction::Transmitting || self.tx_pos >= self.tx_len {
            return Err(Response::Stretch);
        }
        let byte = self.tx[self.tx_pos];
        self.tx_pos += 1;
        Ok(byte)
    }

    /// A STOP condition arrived: the transaction is over and the next one starts clean.
    pub fn stop(&mut self) {
        self.direction = Direction::Idle;
        self.rx_len = 0;
        self.tx_pos = 0;
    }
}

#[cfg(test)]
mod slave_tests {
    use super::*;

    /// Both directions of the same device are the same device. This is the mistake that makes a slave answer only when
    /// it is written to, and it is invisible until someone tries to read.
    #[test]
    fn the_direction_bit_is_not_part_of_the_address() {
        let mut slave = Slave::new(0x6A);
        assert_eq!(slave.address_byte((0x6A << 1) | 1), Response::Ack);
        assert_eq!(slave.direction(), Direction::Transmitting);
        slave.start();
        assert_eq!(slave.address_byte(0x6A << 1), Response::Ack);
        assert_eq!(slave.direction(), Direction::Receiving);
    }

    /// The control: another device's address is refused, in both directions. A slave that ACKed everything would pass
    /// the test above.
    #[test]
    fn another_address_is_refused() {
        let mut slave = Slave::new(0x6A);
        assert_eq!(slave.address_byte((0x51 << 1) | 1), Response::Nack);
        assert_eq!(slave.address_byte(0x51 << 1), Response::Nack);
        assert_eq!(slave.direction(), Direction::Idle);
    }

    /// A write transaction stores what arrives, in order.
    #[test]
    fn a_write_stores_the_bytes_in_order() {
        let mut slave = Slave::new(0x6A);
        slave.start();
        assert_eq!(slave.address_byte(0x6A << 1), Response::Ack);
        assert_eq!(slave.receive(0x11), Response::Ack);
        assert_eq!(slave.receive(0x22), Response::Ack);
        assert_eq!(slave.received(), &[0x11, 0x22]);
    }

    /// A full buffer is refused **and the bytes already stored survive**: the master is told this byte was lost, and the
    /// slave does not quietly drop what came before it.
    #[test]
    fn a_full_buffer_nacks_and_keeps_what_it_had() {
        let mut slave = Slave::new(0x6A);
        slave.start();
        slave.address_byte(0x6A << 1);
        for i in 0..DEPTH {
            assert_eq!(slave.receive(i as u8), Response::Ack);
        }
        assert_eq!(
            slave.receive(0xFF),
            Response::Nack,
            "the fifth byte does not fit"
        );
        assert_eq!(slave.received().len(), DEPTH);
        assert_eq!(slave.received()[DEPTH - 1], (DEPTH - 1) as u8);
    }

    /// A read hands out what was queued, then stretches instead of inventing a byte.
    #[test]
    fn a_read_stretches_when_it_runs_out() {
        let mut slave = Slave::new(0x6A);
        slave.queue(&[0xA5, 0x5A]);
        slave.start();
        assert_eq!(slave.address_byte((0x6A << 1) | 1), Response::Ack);
        assert_eq!(slave.transmit(), Ok(0xA5));
        assert_eq!(slave.transmit(), Ok(0x5A));
        assert_eq!(
            slave.transmit(),
            Err(Response::Stretch),
            "an empty buffer has no byte to give"
        );
    }

    /// A new transaction does not inherit the previous one's bytes.
    #[test]
    fn a_stop_clears_the_transaction() {
        let mut slave = Slave::new(0x6A);
        slave.start();
        slave.address_byte(0x6A << 1);
        slave.receive(0x11);
        slave.stop();
        assert_eq!(slave.direction(), Direction::Idle);
        assert!(slave.received().is_empty());

        slave.start();
        slave.address_byte(0x6A << 1);
        assert_eq!(
            slave.received(),
            &[] as &[u8],
            "the previous write must not leak into this one"
        );
    }
}
