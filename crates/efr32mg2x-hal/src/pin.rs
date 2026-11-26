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
            // SAFETY: one 4-bit `MODE` field is replaced. `clear` and `set` are derived from `pin % 8` and a value from
        // `mode::*`, and the mask keeps every other pin that shares the register.
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
    // SAFETY: `DOUT` is a whole-port output word and `bits` is the mask the caller asked for; writing the word is
    // the only way to set several pins at once, and the arm chosen is the port the caller named.
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
    ///
    /// ## Why this is inside a critical section
    ///
    /// `DOUT` has no atomic set/clear register on this part -- the reference manual and the SVD have
    /// neither `DOUTSET` nor `DOUTCLR` (measured 2026-11-19: zero occurrences in both), so changing one
    /// bit means reading the whole port, changing the bit, and writing the port back. If anything else
    /// writes `DOUT` between that read and that write -- the bit-banged bus driving another pin, or an
    /// interrupt -- the other pin's bit is lost. That is a silent defect: the pin simply does not move.
    ///
    /// Reading and writing under one critical section makes the pair indivisible. It is the same
    /// guarantee the missing registers would have given, obtained in software because the hardware
    /// does not offer it.
    pub fn set(&self, on: bool) {
        let level = on != self.active_low;
        critical_section::with(|_cs| {
            let latch = if level {
                read_dout(self.gpio, self.port) | self.mask
            } else {
                read_dout(self.gpio, self.port) & !self.mask
            };
            write_dout(self.gpio, self.port, latch);
        });
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
    ///
    /// Same read-modify-write as [`Self::set`], so the same critical section: an inversion is a
    /// read and a write too, and losing the other pins' bits is just as silent.
    pub fn toggle(&self) {
        critical_section::with(|_cs| {
            write_dout(
                self.gpio,
                self.port,
                read_dout(self.gpio, self.port) ^ self.mask,
            );
        });
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

    /// True while the pad reads high.
    pub fn is_high(&self) -> bool {
        read_din(self.gpio, self.port) & self.mask != 0
    }

    /// Choose which way the internal pull resistor drives the pin.
    ///
    /// In the pull mode, the output latch is not a data output — it selects the resistor: `true` ties the pad
    /// towards VDD, `false` towards ground. On a pin with nothing attached that makes the latch a slow but real
    /// edge generator, which is how a probe with no jumper wire can drive the edge detector.
    pub fn set_pull_up(&self, up: bool) {
        let latch = if up {
            read_dout(self.gpio, self.port) | self.mask
        } else {
            read_dout(self.gpio, self.port) & !self.mask
        };
        write_dout(self.gpio, self.port, latch);
    }
}

/// One GPIO external-interrupt channel, bound to the pin it watches and the edges it reacts to.
///
/// ## What the registers actually say, and why this is written in nibbles
///
/// The 2025.6.2 SVD and the vendor header agree on the layout, and both differ from what one expects from the
/// Series 1 parts: an external interrupt is **not** pointed at (port, pin) with a single 7-bit field. The port is a
/// two-bit field in `EXTIPSELL` (`EXTIPSELH` for channels 8..15), the pin is a two-bit field in `EXTIPINSELL`
/// (`EXTIPINSELH`), and the two bits hold the pin's index **inside its group of four**. The emlib macro says the
/// same thing from the caller's side: `GPIO_INTNO_PIN_VALID(intNo, pin)` requires
/// `(intNo & ~3) == (pin & ~3)`. Measured 2026-11-26: channel 3 on port C, pin 3 reads back `EXTIPSEL = 0x2000`
/// and `EXTIPINSELL = 0x3000`, which is that encoding with both nibbles non-zero.
///
/// So a channel can only watch within its own group: channel `n` watches pin `(n & 0xC) | v` for `v` in `0..4`.
/// [`Board::watch_edges`](crate::board::Board::watch_edges) binds channel `n = pin`, which is always legal, and the
/// PRS refuses channels above 7 because the vendor signal table only names `GPIOPIN0..7`.
///
/// ## The edges, and what they are not
///
/// `EXTIRISE`/`EXTIFALL` arm the GPIO's own interrupt flag. They are **not** the path the PRS takes: measured
/// 2026-11-26, a channel with `EXTIRISE` cleared and only `EXTIFALL` armed still let four rising pad edges drive four
/// conversions, because the asynchronous signal carries the pad's level. Configuring both edges remains the right
/// default — a caller of this type is normally asking for an interrupt as well.
///
/// ## The flag
///
/// `IF` is the sticky per-channel flag. As on LETIMER (measured 2026-11-26), writing a one to `IF` sets it; it is
/// cleared through the alias window at `IF + 0x2000`. Measured for this block as well: after eight edges `IF` read
/// `0x9` (channels 0 and 3) and `clear()` on channel 3 left `0x1`, so the alias clears the channel it names and no
/// other.
pub struct EdgeWatch<'a> {
    gpio: &'a pac::gpio_s::RegisterBlock,
    channel: u8,
}

/// Offset of `IF` in the GPIO block; the vendor header and the PAC agree on `0x420`.
const IF_OFFSET: usize = 0x420;

/// Offset added to a register to reach its clear alias. The LETIMER record measured the same `+0x2000` window.
const CLR_ALIAS: usize = 0x2000;

impl<'a> EdgeWatch<'a> {
    /// Point external-interrupt `channel = pin` at `port`/`pin`, with the requested edges, and clear its flag.
    pub(crate) fn bind(
        gpio: &'a pac::gpio_s::RegisterBlock,
        port: Port,
        pin: u8,
        rising: bool,
        falling: bool,
    ) -> Self {
        debug_assert!(pin < 16, "pin index out of range");
        let channel = pin;
        let bit = 1u32 << channel;
        let shift = 4 * u32::from(channel % 8);
        let field = 0x3u32 << shift;

        // One macro, because every write below is the same shape: a read-modify-write of one register with a mask
        // that holds only this channel's bits. The four select registers have different generated types, and a note
        // repeated six times is a note nobody reads.
        macro_rules! merge {
            ($reg:expr, $mask:expr, $value:expr) => {{
                // SAFETY: `$mask` is the complement of this channel's own field (or that field itself), so every
                // other channel's bits keep their value; `channel < 16` and the field width is two bits, so the
                // shifted value cannot leave its nibble.
                $reg.modify(|r, w| unsafe { w.bits((r.bits() & $mask) | $value) });
            }};
        }

        // Both edge masks off first: a channel left armed by an earlier configuration keeps generating edges while
        // the port and pin fields are being rewritten, which is an edge from the old pin attributed to the new one.
        merge!(gpio.extirise(), !bit, 0);
        merge!(gpio.extifall(), !bit, 0);

        let port_bits = match port {
            Port::A => 0u32,
            Port::B => 1,
            Port::C => 2,
            Port::D => 3,
        };
        let port_word = port_bits << shift;
        let pin_word = (u32::from(pin) & 0x3) << shift;
        if channel < 8 {
            merge!(gpio.extipsell(), !field, port_word);
            merge!(gpio.extipinsell(), !field, pin_word);
        } else {
            merge!(gpio.extipselh(), !field, port_word);
            merge!(gpio.extipinselh(), !field, pin_word);
        }

        let watch = Self { gpio, channel };
        // A flag left over from whatever the channel watched before would be read as this pin's first edge.
        watch.clear();

        if rising {
            merge!(gpio.extirise(), !bit, bit);
        }
        if falling {
            merge!(gpio.extifall(), !bit, bit);
        }
        watch
    }

    /// The external-interrupt channel this watch drives, which is also the PRS `SIGSEL` for `SOURCE = GPIO`.
    pub fn channel(&self) -> u8 {
        self.channel
    }

    /// The whole `IF` word, for a probe that reports what the hardware latched rather than what it expected.
    pub fn flags(&self) -> u32 {
        self.gpio.if_().read().bits()
    }

    /// The port and pin select words this channel sits in, as they read back.
    ///
    /// Channels 8..15 live in the `*H` registers, so a caller comparing the two channels also compares the two
    /// registers; the value is the whole word, not the channel's own nibble, because a nibble that was not written
    /// is exactly the thing worth seeing.
    pub fn selection(&self) -> (u32, u32) {
        if self.channel < 8 {
            (
                self.gpio.extipsell().read().bits(),
                self.gpio.extipinsell().read().bits(),
            )
        } else {
            (
                self.gpio.extipselh().read().bits(),
                self.gpio.extipinselh().read().bits(),
            )
        }
    }

    /// True while this channel's flag is latched.
    pub fn pending(&self) -> bool {
        self.flags() & (1u32 << self.channel) != 0
    }

    /// Clear this channel's flag through the alias window.
    pub fn clear(&self) {
        let address = self.gpio as *const _ as usize + IF_OFFSET + CLR_ALIAS;
        // SAFETY: the alias window maps the same `IF` word with write-one-to-clear semantics, and the mask is this
        // channel's own bit, so no other channel's flag is touched.
        unsafe { core::ptr::write_volatile(address as *mut u32, 1u32 << self.channel) }
    }
}
