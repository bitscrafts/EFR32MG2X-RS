//! Bit-banged I2C master on PC04 (SDA) / PC05 (SCL) — the Expansion Base bus.
//!
//! ## Why not the I2C0 peripheral
//!
//! Measured on this die (2026-11-10): with the pins correctly configured
//! (open-drain, both lines idle high, writes ACKing), a read never completes —
//! after the read address the peripheral reports `STATE=0xd7`
//! (BUSY|MASTER|TRANSMITTER|NACKED, state field 6 = DATAACK) and never raises
//! RXDATAV, with `CMD.ACK`, without it, with AUTOACK on, with a repeated START
//! and with two separate transactions. Writes work; reads do not.
//!
//! Driving the bus from software removes the question: every edge is ours and
//! the ACK bit is sampled by us, so "device present" and "byte received" are
//! direct observations. It is slower (~90 kHz here) and that is irrelevant for
//! a sensor bus.
//!
//! ## Electrical contract
//!
//! - pins in SVD MODE 8 `WIREDAND` (open drain),
//! - `DOUT` = 0 pulls low, `DOUT` = 1 releases,
//! - pull-ups are on the Expansion Base (both lines read 1 when idle),
//! - `DIN` samples SDA.
//!
//! `HALF_BIT` is a cycle count, so the bus speed follows SYSCLK; anything from
//! ~1 us up is inside the 100 kHz I2C timing, and slower is always safe.

#![allow(dead_code)]

use efr32mg24_pac as pac;

/// Half-bit delay in CPU cycles (~5 us at 19 MHz).
pub const HALF_BIT: u32 = 60;

const PIN_SDA: u32 = 1 << 4; // PC04
const PIN_SCL: u32 = 1 << 5; // PC05

pub struct Bus<'a> {
    gpio: &'a pac::gpio_s::RegisterBlock,
}

impl<'a> Bus<'a> {
    /// Configure PC04/PC05 as open-drain outputs, both released.
    pub fn new(gpio: &'a pac::gpio_s::RegisterBlock) -> Self {
        gpio.portc_model()
            .modify(|_r, w| w.mode4().wiredand().mode5().wiredand());
        gpio.portc_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() | PIN_SDA | PIN_SCL) });
        Self { gpio }
    }

    fn delay(&self) {
        cortex_m::asm::delay(HALF_BIT);
    }

    fn set(&self, sda_low: bool, scl_low: bool) {
        let mut bits = self.gpio.portc_dout().read().bits();
        bits = if sda_low {
            bits & !PIN_SDA
        } else {
            bits | PIN_SDA
        };
        bits = if scl_low {
            bits & !PIN_SCL
        } else {
            bits | PIN_SCL
        };
        self.gpio.portc_dout().write(|w| unsafe { w.bits(bits) });
        self.delay();
    }

    fn sda_high(&self) -> bool {
        self.gpio.portc_din().read().bits() & PIN_SDA != 0
    }

    pub fn start(&self) {
        self.set(true, true);
        self.set(true, false);
        self.set(false, false); // SDA falls while SCL is high: START
        self.set(true, false);
    }

    pub fn stop(&self) {
        self.set(true, true);
        self.set(true, false);
        self.set(false, false); // SDA rises while SCL is high: STOP
    }

    /// One byte, MSB first. Returns true when the device ACKed.
    pub fn write_byte(&self, byte: u8) -> bool {
        for i in (0..8).rev() {
            let low = (byte >> i) & 1 == 0;
            self.set(low, true);
            self.set(low, false);
            self.set(low, true);
        }
        self.set(false, true); // release SDA for the ACK clock
        self.set(false, false);
        let acked = !self.sda_high();
        self.set(false, true);
        acked
    }

    /// One byte, MSB first; `ack` keeps the bus for another byte.
    pub fn read_byte(&self, ack: bool) -> u8 {
        let mut value = 0u8;
        self.set(false, true);
        for _ in 0..8 {
            self.set(false, false);
            value = (value << 1) | u8::from(self.sda_high());
            self.set(false, true);
        }
        self.set(ack, true); // ACK drives low, NACK releases
        self.set(ack, false);
        self.set(ack, true);
        value
    }

    /// Does anything answer at this 7-bit address?
    pub fn probe(&self, addr: u8) -> bool {
        self.start();
        let acked = self.write_byte(addr << 1);
        self.stop();
        acked
    }

    /// Read one register: START, addr+W, reg, repeated START, addr+R, byte.
    pub fn read_reg(&self, addr: u8, reg: u8) -> Option<u8> {
        self.start();
        if !self.write_byte(addr << 1) || !self.write_byte(reg) {
            self.stop();
            return None;
        }
        self.start();
        if !self.write_byte((addr << 1) | 1) {
            self.stop();
            return None;
        }
        let value = self.read_byte(false);
        self.stop();
        Some(value)
    }

    /// Write one register: START, addr+W, reg, value, STOP.
    pub fn write_reg(&self, addr: u8, reg: u8, value: u8) -> bool {
        self.start();
        let ok = self.write_byte(addr << 1) && self.write_byte(reg) && self.write_byte(value);
        self.stop();
        ok
    }

    /// Write a run of command bytes to one address (a control stream).
    pub fn write_cmds(&self, addr: u8, bytes: &[u8]) -> bool {
        self.start();
        if !self.write_byte(addr << 1) {
            self.stop();
            return false;
        }
        for b in bytes {
            if !self.write_byte(*b) {
                self.stop();
                return false;
            }
        }
        self.stop();
        true
    }

    /// Read `out.len()` bytes starting at `reg` (repeated START).
    pub fn read_regs(&self, addr: u8, reg: u8, out: &mut [u8]) -> bool {
        self.start();
        if !self.write_byte(addr << 1) || !self.write_byte(reg) {
            self.stop();
            return false;
        }
        self.start();
        if !self.write_byte((addr << 1) | 1) {
            self.stop();
            return false;
        }
        let last = out.len().saturating_sub(1);
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = self.read_byte(i != last);
        }
        self.stop();
        true
    }
}
