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

/// Which GPIO port a bus is on.
///
/// The HAL does not know which pins a board uses; it only knows how to drive a port. The
/// BSP supplies the port and the two bit masks.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Port {
    /// PORTA.
    A,
    /// PORTB.
    B,
    /// PORTC.
    C,
    /// PORTD.
    D,
}

/// The two pins of a bit-banged bus, as bit masks on one port.
///
/// **The order is not a detail.** On the XIAO MG24 Sense the IMU answers on `SDA = PB2`,
/// `SCL = PB3` — the opposite of what the Arduino variant declares — and with the lines the
/// other way round the START condition never occurs, so the device never answers and the bus
/// looks empty. Measured, not assumed: see `docs/04-reference/IMU-INVESTIGATION.md`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bus {
    /// The port both lines live on.
    pub port: Port,
    /// Bit mask of the data line within that port.
    pub sda: u32,
    /// Bit mask of the clock line within that port.
    pub scl: u32,
}

impl Bus {
    /// Two pins on one port, as masks (`1 << pin`).
    #[must_use]
    pub const fn new(port: Port, sda: u32, scl: u32) -> Self {
        Self { port, sda, scl }
    }
}

/// `MODE n = 0b1000` in the GPIO model register: open drain, pulls down, lets the pull-up
/// bring the line high. The value is the hardware's, from `gpio_s::porta_model::Mode0`.
const MODE_WIREDAND: u32 = 8;

/// Software I2C master over PAC GPIO registers, on whatever pins it is given.
///
/// Cheap, deterministic and — unlike the on-chip master on this die — able to complete a
/// read after a write.
pub struct BitBangI2c<'a> {
    gpio: &'a pac::gpio_s::RegisterBlock,
    bus: Bus,
}

impl<'a> BitBangI2c<'a> {
    /// Take the given two pins, set both open drain, and release them.
    pub fn on(gpio: &'a pac::gpio_s::RegisterBlock, bus: Bus) -> Self {
        // Both pins become WIREDAND, keeping the other pins of the port untouched: a blind
        // write would reconfigure every other pin on the port, which is how a bus silently
        // breaks whatever else shares it.
        let clear = !((0xF << (4 * bus.sda.trailing_zeros()))
            | (0xF << (4 * bus.scl.trailing_zeros())));
        let set = (MODE_WIREDAND << (4 * bus.sda.trailing_zeros()))
            | (MODE_WIREDAND << (4 * bus.scl.trailing_zeros()));
        // The four MODEL registers are four different types, so the same write has to be
        // spelled in each arm; a macro keeps that from being four copies of the expression.
        macro_rules! set_mode {
            ($reg:expr) => {
                $reg.modify(|r, w| unsafe { w.bits((r.bits() & clear) | set) })
            };
        }
        match bus.port {
            Port::A => set_mode!(gpio.porta_model()),
            Port::B => set_mode!(gpio.portb_model()),
            Port::C => set_mode!(gpio.portc_model()),
            Port::D => set_mode!(gpio.portd_model()),
        };
        // Writing 1 releases the line: the pull-up does the rest.
        self_write_dout(gpio, bus.port, self_read_dout(gpio, bus.port) | bus.sda | bus.scl);
        Self { gpio, bus }
    }

    fn delay(&self) {
        cortex_m::asm::delay(HALF_BIT_CYCLES);
    }

    fn set(&self, sda_low: bool, scl_low: bool) {
        let mut bits = self_read_dout(self.gpio, self.bus.port);
        bits = if sda_low {
            bits & !self.bus.sda
        } else {
            bits | self.bus.sda
        };
        bits = if scl_low {
            bits & !self.bus.scl
        } else {
            bits | self.bus.scl
        };
        self_write_dout(self.gpio, self.bus.port, bits);
        self.delay();
    }

    fn sda_high(&self) -> bool {
        let din = match self.bus.port {
            Port::A => self.gpio.porta_din().read().bits(),
            Port::B => self.gpio.portb_din().read().bits(),
            Port::C => self.gpio.portc_din().read().bits(),
            Port::D => self.gpio.portd_din().read().bits(),
        };
        din & self.bus.sda != 0
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


/// Read the output latch of a port.
fn self_read_dout(gpio: &pac::gpio_s::RegisterBlock, port: Port) -> u32 {
    match port {
        Port::A => gpio.porta_dout().read().bits(),
        Port::B => gpio.portb_dout().read().bits(),
        Port::C => gpio.portc_dout().read().bits(),
        Port::D => gpio.portd_dout().read().bits(),
    }
}

/// Write the output latch of a port.
fn self_write_dout(gpio: &pac::gpio_s::RegisterBlock, port: Port, bits: u32) {
    match port {
        Port::A => gpio.porta_dout().write(|w| unsafe { w.bits(bits) }),
        Port::B => gpio.portb_dout().write(|w| unsafe { w.bits(bits) }),
        Port::C => gpio.portc_dout().write(|w| unsafe { w.bits(bits) }),
        Port::D => gpio.portd_dout().write(|w| unsafe { w.bits(bits) }),
    };
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
