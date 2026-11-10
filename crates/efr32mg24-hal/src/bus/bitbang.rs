//! Bit-banged I2C master on the XIAO's two I2C buses.
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

/// Which pair of pins this master drives.
///
/// Both buses are bit-banged from the same code; only the pins differ. The
/// Expansion Base sits on PC04/PC05, the on-board LSM6DS3TR-C on PB2/PB3.
///
/// The order is worth stating because it cost a night: the Arduino variant
/// declares `PIN_WIRE_SDA1 = PB3` / `PIN_WIRE_SCL1 = PB2`, and with the lines in
/// that order the sensor never answers. Measured on the board, SDA is **PB2**
/// and SCL is **PB3**.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Wiring {
    /// Expansion Base: PC04 = SDA, PC05 = SCL.
    BaseBus,
    /// On-board IMU: PB2 = SDA, PB3 = SCL.
    ImuBus,
}

impl Wiring {
    fn pins(self) -> (u32, u32) {
        match self {
            Wiring::BaseBus => (1 << 4, 1 << 5),
            Wiring::ImuBus => (1 << 2, 1 << 3),
        }
    }
}

/// Software I2C master driving PC04/PC05 through PAC GPIO registers.
///
/// Cheap, deterministic and — unlike the on-chip master on this die — able to
/// complete a read after a write.
pub struct BitBangI2c<'a> {
    gpio: &'a pac::gpio_s::RegisterBlock,
    wiring: Wiring,
}

impl<'a> BitBangI2c<'a> {
    /// Take the Expansion Base pins (PC04/PC05) and release both lines.
    pub fn new(gpio: &'a pac::gpio_s::RegisterBlock) -> Self {
        Self::on(gpio, Wiring::BaseBus)
    }

    /// Take the given bus, set both pins open drain and release them.
    pub fn on(gpio: &'a pac::gpio_s::RegisterBlock, wiring: Wiring) -> Self {
        let (sda, scl) = wiring.pins();
        match wiring {
            Wiring::BaseBus => {
                gpio.portc_model()
                    .modify(|_r, w| w.mode4().wiredand().mode5().wiredand());
                gpio.portc_dout()
                    .modify(|r, w| unsafe { w.bits(r.bits() | sda | scl) });
            }
            Wiring::ImuBus => {
                gpio.portb_model()
                    .modify(|_r, w| w.mode2().wiredand().mode3().wiredand());
                gpio.portb_dout()
                    .modify(|r, w| unsafe { w.bits(r.bits() | sda | scl) });
            }
        }
        Self { gpio, wiring }
    }

    fn delay(&self) {
        cortex_m::asm::delay(HALF_BIT_CYCLES);
    }

    fn set(&self, sda_low: bool, scl_low: bool) {
        let (pin_sda, pin_scl) = self.wiring.pins();
        let mut bits = match self.wiring {
            Wiring::BaseBus => self.gpio.portc_dout().read().bits(),
            Wiring::ImuBus => self.gpio.portb_dout().read().bits(),
        };
        bits = if sda_low {
            bits & !pin_sda
        } else {
            bits | pin_sda
        };
        bits = if scl_low {
            bits & !pin_scl
        } else {
            bits | pin_scl
        };
        match self.wiring {
            Wiring::BaseBus => {
                self.gpio.portc_dout().write(|w| unsafe { w.bits(bits) });
            }
            Wiring::ImuBus => {
                self.gpio.portb_dout().write(|w| unsafe { w.bits(bits) });
            }
        }
        self.delay();
    }

    fn sda_high(&self) -> bool {
        let (pin_sda, _) = self.wiring.pins();
        let din = match self.wiring {
            Wiring::BaseBus => self.gpio.portc_din().read().bits(),
            Wiring::ImuBus => self.gpio.portb_din().read().bits(),
        };
        din & pin_sda != 0
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
