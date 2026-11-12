//! Pin-level primitives: the HAL's lowest layer above the registers.
//!
//! Nothing here knows which board it is on. A pin is a port, a bit and a polarity; which
//! pins a board uses is the BSP's business. That separation is what lets the same driver
//! serve two boards — and it is the rule this crate broke for months by naming PC03, PC01,
//! PC04, PD5 and PA7 in its own board module.

use crate::pac;

/// Which GPIO port a pin is on.
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

/// `MODE n` values from the GPIO model registers (`gpio_s::porta_model::Mode0`).
///
/// Only the ones this crate configures are named; the rest are documented in the reference
/// manual and adding one here is a one-line change.
pub mod mode {
    /// Input buffer off; the pull-up applies if `DOUT` is set.
    pub const DISABLED: u32 = 0;
    /// Input enabled, no pull.
    pub const INPUT: u32 = 1;
    /// Input enabled with a pull-up (the pull-up is selected by `DOUT = 1`).
    pub const INPUT_PULL: u32 = 2;
    /// Push-pull output.
    pub const PUSH_PULL: u32 = 4;
    /// Open-drain output: pulls low, releases high; the pull-up brings it back.
    pub const WIRED_AND: u32 = 8;
}

/// Set the mode of one pin without disturbing the other seven on the same half-port.
///
/// Pins 0..7 live in `MODEL`, pins 8..15 in `MODEH`, four bits each. A blind write to those
/// registers reconfigures every other pin that shares them, which is how a bus silently
/// breaks whatever else is on the port — so the read-modify-write below is not tidiness.
pub(crate) fn set_mode(gpio: &pac::gpio_s::RegisterBlock, port: Port, pin: u8, value: u32) {
    debug_assert!(pin < 16, "pin index out of range");
    let shift = 4 * u32::from(pin % 8);
    let clear = !(0xF << shift);
    let set = (value & 0xF) << shift;

    // A macro because the four MODEL/MODEH register types differ, and the write is the same
    // expression in each. The value `modify` returns is the register's contents; this setter
    // only wants the side effect, so it is dropped explicitly.
    macro_rules! write {
        ($reg:expr) => {{
            let _ = $reg.modify(|r, w| unsafe { w.bits((r.bits() & clear) | set) });
        }};
    }
    match (port, pin < 8) {
        (Port::A, true) => write!(gpio.porta_model()),
        (Port::A, false) => write!(gpio.porta_modeh()),
        (Port::C, true) => write!(gpio.portc_model()),
        (Port::C, false) => write!(gpio.portc_modeh()),
        (Port::B, true) => write!(gpio.portb_model()),
        (Port::D, true) => write!(gpio.portd_model()),
        // PORTB and PORTD have no MODEH in this package's SVD because those ports expose
        // only pins 0..7 here; a pin index above 7 on them is a wiring error, not a register
        // to invent.
        (Port::B | Port::D, false) => debug_assert!(false, "port has no pins above 7"),
    };
}

/// Read a whole port's output latch.
pub(crate) fn read_dout(gpio: &pac::gpio_s::RegisterBlock, port: Port) -> u32 {
    match port {
        Port::A => gpio.porta_dout().read().bits(),
        Port::B => gpio.portb_dout().read().bits(),
        Port::C => gpio.portc_dout().read().bits(),
        Port::D => gpio.portd_dout().read().bits(),
    }
}

/// Write a whole port's output latch.
pub(crate) fn write_dout(gpio: &pac::gpio_s::RegisterBlock, port: Port, bits: u32) {
    match port {
        Port::A => gpio.porta_dout().write(|w| unsafe { w.bits(bits) }),
        Port::B => gpio.portb_dout().write(|w| unsafe { w.bits(bits) }),
        Port::C => gpio.portc_dout().write(|w| unsafe { w.bits(bits) }),
        Port::D => gpio.portd_dout().write(|w| unsafe { w.bits(bits) }),
    };
}

/// Read a whole port's input register.
pub(crate) fn read_din(gpio: &pac::gpio_s::RegisterBlock, port: Port) -> u32 {
    match port {
        Port::A => gpio.porta_din().read().bits(),
        Port::B => gpio.portb_din().read().bits(),
        Port::C => gpio.portc_din().read().bits(),
        Port::D => gpio.portd_din().read().bits(),
    }
}

/// A push-pull output pin with a polarity.
///
/// `active_low` is not a preference: an LED wired to ground through a resistor lights when
/// the pin sinks current, and saying so here is what stops every caller from rediscovering
/// it (one of them, in this project, printed a "kept" label over a clock it had just written).
pub struct Output<'a> {
    gpio: &'a pac::gpio_s::RegisterBlock,
    port: Port,
    mask: u32,
    active_low: bool,
}

impl<'a> Output<'a> {
    /// Configure one pin as a push-pull output, starting in the off state.
    pub(crate) fn new(
        gpio: &'a pac::gpio_s::RegisterBlock,
        port: Port,
        pin: u8,
        active_low: bool,
    ) -> Self {
        set_mode(gpio, port, pin, mode::PUSH_PULL);
        let mask = 1 << pin;
        // Off is "not asserted": high for an active-low pin, low for an active-high one.
        let latch = if active_low {
            read_dout(gpio, port) | mask
        } else {
            read_dout(gpio, port) & !mask
        };
        write_dout(gpio, port, latch);
        Self {
            gpio,
            port,
            mask,
            active_low,
        }
    }

    /// Assert or release the pin, honouring the polarity it was built with.
    pub fn set(&self, on: bool) {
        let level = on != self.active_low;
        let latch = if level {
            read_dout(self.gpio, self.port) | self.mask
        } else {
            read_dout(self.gpio, self.port) & !self.mask
        };
        write_dout(self.gpio, self.port, latch);
    }

    /// Assert the pin.
    pub fn on(&self) {
        self.set(true);
    }

    /// Release the pin.
    pub fn off(&self) {
        self.set(false);
    }

    /// Invert the pin.
    pub fn toggle(&self) {
        write_dout(
            self.gpio,
            self.port,
            read_dout(self.gpio, self.port) ^ self.mask,
        );
    }
}

/// An input pin with the internal pull-up enabled.
///
/// A button that shorts to ground reads high at rest and low when pressed. Boards that add
/// their own pull-up are unaffected: two pull-ups in parallel is still a pull-up.
pub struct Input<'a> {
    gpio: &'a pac::gpio_s::RegisterBlock,
    port: Port,
    mask: u32,
}

impl<'a> Input<'a> {
    /// Configure one pin as an input with the internal pull-up.
    pub(crate) fn with_pull_up(gpio: &'a pac::gpio_s::RegisterBlock, port: Port, pin: u8) -> Self {
        set_mode(gpio, port, pin, mode::INPUT_PULL);
        // The pull-up is selected by the output latch, not by the mode alone.
        write_dout(gpio, port, read_dout(gpio, port) | (1 << pin));
        Self {
            gpio,
            port,
            mask: 1 << pin,
        }
    }

    /// True while the pin is pulled low — the pressed state of a button to ground.
    pub fn is_low(&self) -> bool {
        read_din(self.gpio, self.port) & self.mask == 0
    }
}
