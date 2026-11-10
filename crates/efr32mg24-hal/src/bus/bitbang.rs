//! Bit-banged I2C master on the XIAO I2C pins (PC04 = SDA, PC05 = SCL).
//!
//! This is the working transport on this die. The on-chip I2C0 master does not
//! complete a read that follows a write — fourteen attempts are recorded in
//! `docs/04-reference/I2C0-READ-DEFECT.md` — so the bus is driven from software
//! through PAC GPIO registers. No external crate, no HAL of another project.
//!
//! Electrical contract (established by measurement, not assumption):
//!
//! - pins in SVD MODE 8 (`WIREDAND`, open drain),
//! - `DOUT` = 0 pulls the line low, `DOUT` = 1 releases it,
//! - pull-ups come from the Expansion Base (both lines read 1 when idle),
//! - `DIN` samples SDA.
//!
//! `HALF_BIT_CYCLES` is a CPU-cycle count, so the bus clock follows SYSCLK.
//! Anything from ~1 us per half-bit upwards is inside the I2C timing; slower is
//! always safe, and a sensor refresh does not care about the difference between
//! 90 kHz and 400 kHz.

use efr32mg24_pac as pac;

use super::{BusError, I2cBus};

/// Half-bit delay in CPU cycles (~5 us at 19 MHz): sets the bus clock.
const HALF_BIT_CYCLES: u32 = 60;

const PIN_SDA: u32 = 1 << 4; // PC04
const PIN_SCL: u32 = 1 << 5; // PC05

/// Software I2C master driving PC04/PC05 through PAC GPIO registers.
///
/// Cheap, deterministic and — unlike the on-chip master on this die — able to
/// complete a read after a write.
pub struct BitBangI2c<'a> {
    gpio: &'a pac::gpio_s::RegisterBlock,
}

impl<'a> BitBangI2c<'a> {
    /// Take the two pins and release both lines.
    pub fn new(gpio: &'a pac::gpio_s::RegisterBlock) -> Self {
        gpio.portc_model()
            .modify(|_r, w| w.mode4().wiredand().mode5().wiredand());
        gpio.portc_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() | PIN_SDA | PIN_SCL) });
        Self { gpio }
    }

    fn delay(&self) {
        cortex_m::asm::delay(HALF_BIT_CYCLES);
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

    fn start(&self) {
        self.set(true, true);
        self.set(true, false);
        self.set(false, false); // SDA falls while SCL is high: START
        self.set(true, false);
    }

    fn stop(&self) {
        self.set(true, true);
        self.set(true, false);
        self.set(false, false); // SDA rises while SCL is high: STOP
    }

    fn write_byte(&self, byte: u8) -> bool {
        for i in (0..8).rev() {
            let low = (byte >> i) & 1 == 0;
            self.set(low, true);
            self.set(low, false);
            self.set(low, true);
        }
        self.set(false, true); // release SDA for the acknowledge clock
        self.set(false, false);
        let acked = !self.sda_high();
        self.set(false, true);
        acked
    }

    fn read_byte(&self, ack: bool) -> u8 {
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

    fn write_all(&self, addr: u8, bytes: &[u8]) -> Result<(), BusError> {
        self.start();
        if !self.write_byte(addr << 1) {
            self.stop();
            return Err(BusError::NoAck);
        }
        for b in bytes {
            if !self.write_byte(*b) {
                self.stop();
                return Err(BusError::DataNack);
            }
        }
        self.stop();
        Ok(())
    }

    fn read_all(&self, addr: u8, out: &mut [u8]) -> Result<(), BusError> {
        self.start();
        if !self.write_byte((addr << 1) | 1) {
            self.stop();
            return Err(BusError::NoAck);
        }
        let last = out.len().saturating_sub(1);
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = self.read_byte(i != last);
        }
        self.stop();
        Ok(())
    }
}

impl I2cBus for BitBangI2c<'_> {
    fn probe(&mut self, addr: u8) -> bool {
        self.start();
        let acked = self.write_byte(addr << 1);
        self.stop();
        acked
    }

    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), BusError> {
        self.write_all(addr, bytes)
    }

    fn read(&mut self, addr: u8, out: &mut [u8]) -> Result<(), BusError> {
        self.read_all(addr, out)
    }

    fn write_read(&mut self, addr: u8, prefix: &[u8], out: &mut [u8]) -> Result<(), BusError> {
        self.start();
        if !self.write_byte(addr << 1) {
            self.stop();
            return Err(BusError::NoAck);
        }
        for b in prefix {
            if !self.write_byte(*b) {
                self.stop();
                return Err(BusError::DataNack);
            }
        }
        self.start(); // repeated START, no STOP in between
        if !self.write_byte((addr << 1) | 1) {
            self.stop();
            return Err(BusError::NoAck);
        }
        let last = out.len().saturating_sub(1);
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = self.read_byte(i != last);
        }
        self.stop();
        Ok(())
    }
}
