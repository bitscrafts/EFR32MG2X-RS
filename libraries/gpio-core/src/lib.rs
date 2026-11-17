//! One pin at a time: the GPIO model fields, the data register, and the route encoding.
//!
//! The defect this closes, from the 2026-11-17 review (H17): the pin driver read a whole port's `DOUT` or `MODEL`
//! register, changed one field, and wrote the whole register back. Between the read and the write, every other pin on
//! that port is carried along -- so a button press sampled in the window, or an interrupt flag cleared elsewhere, is
//! silently undone by a write that was only supposed to touch one pin. The review graded it C and named two sites
//! (`hal/pin.rs:140-166`, `hal/bus/bitbang.rs:237-250`).
//!
//! Two things make it testable here: the fields are four bits wide at a known position (`MODEL n` occupies bits
//! `4n..4n+3`), and the route encoding is `port << 4 | pin` (`efr32mg24_gpio.h`). Both are arithmetic, and arithmetic
//! with a mask is exactly what a host test can enumerate: every pin, every mode, and a check that no other pin moved.

#![no_std]

/// Pins per port on this family.
pub const PINS_PER_PORT: u8 = 16;
/// Pins per model register: `PORTA_MODEL` has `MODEL0..MODEL7`, four bits each, which is exactly 32 bits.
///
/// The first version of this crate assumed sixteen four-bit fields in one register -- 64 bits in a 32-bit register, which
/// the exhaustive test caught by shifting out of the word. Pins 8..15 live in the *second* register (`MODELH`), with the
/// same fields and the same width.
pub const PINS_PER_MODEL: u8 = 8;
/// Bits per `MODEL` field.
pub const MODEL_BITS: u32 = 4;
/// Mask of one pin's model field.
pub const MODEL_MASK: u32 = (1 << MODEL_BITS) - 1;

/// The four ports, as the route registers encode them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Port {
    /// Port A.
    A,
    /// Port B.
    B,
    /// Port C.
    C,
    /// Port D.
    D,
}

impl Port {
    /// The value the route register wants for this port (`PORT` field).
    pub fn code(self) -> u32 {
        match self {
            Port::A => 0,
            Port::B => 1,
            Port::C => 2,
            Port::D => 3,
        }
    }

    /// Every port, for exhaustive tests.
    pub const ALL: [Port; 4] = [Port::A, Port::B, Port::C, Port::D];
}

/// Why a pin operation was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinError {
    /// There is no pin 16 on a 16-pin port.
    NoSuchPin,
    /// A model value wider than the four-bit field.
    ModelTooWide,
}

/// Which of the two model registers a pin lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelRegister {
    /// `MODEL`: pins 0..7.
    Low,
    /// `MODELH`: pins 8..15.
    High,
}

/// The register and the shift for a pin's field, or why there is none.
///
/// A caller that uses the wrong register changes a pin it did not name, which is the same class of defect as H17 in a
/// different disguise -- so the pair is returned together and cannot be mixed up.
pub fn field_of(pin: u8) -> Result<(ModelRegister, u32), PinError> {
    if pin >= PINS_PER_PORT {
        return Err(PinError::NoSuchPin);
    }
    let register = if pin < PINS_PER_MODEL {
        ModelRegister::Low
    } else {
        ModelRegister::High
    };
    Ok((register, (pin % PINS_PER_MODEL) as u32 * MODEL_BITS))
}

/// The shift for a pin's field within its own register, or why there is none.
pub fn shift_for(pin: u8) -> Result<u32, PinError> {
    Ok(field_of(pin)?.1)
}

/// Both model registers of one port, so a pin and its field can never be separated.
///
/// Carrying them as a pair is not tidiness: the first version took a bare `u32`, and the exhaustive test showed pin 0 and
/// pin 8 aliasing onto the same bits -- which is precisely the confusion the hardware resolves by having two registers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Models {
    /// `MODEL`: pins 0..7, four bits each.
    pub low: u32,
    /// `MODELH`: pins 8..15, four bits each.
    pub high: u32,
}

impl Models {
    /// Both registers, as the hardware reads them.
    pub fn new(low: u32, high: u32) -> Self {
        Self { low, high }
    }

    /// This set with `pin`'s field set to `model`, and every other field exactly as it was.
    ///
    /// The mask confines the read-modify-write to one four-bit field, so a value sampled from another pin between a read
    /// and a write cannot be carried away by the write -- the whole of H17, as a type.
    pub fn with(self, pin: u8, model: u8) -> Result<Self, PinError> {
        if model as u32 > MODEL_MASK {
            return Err(PinError::ModelTooWide);
        }
        let (register, shift) = field_of(pin)?;
        let field = MODEL_MASK << shift;
        let value = (model as u32) << shift;
        Ok(match register {
            ModelRegister::Low => Self {
                low: (self.low & !field) | value,
                ..self
            },
            ModelRegister::High => Self {
                high: (self.high & !field) | value,
                ..self
            },
        })
    }

    /// The model value stored for `pin`.
    pub fn model_of(&self, pin: u8) -> Result<u8, PinError> {
        let (register, shift) = field_of(pin)?;
        let word = match register {
            ModelRegister::Low => self.low,
            ModelRegister::High => self.high,
        };
        Ok(((word >> shift) & MODEL_MASK) as u8)
    }
}

/// `dout` with `pin` set to `level`, and every other pin exactly as it was.
pub fn with_dout(dout: u32, pin: u8, level: bool) -> Result<u32, PinError> {
    if pin >= PINS_PER_PORT {
        return Err(PinError::NoSuchPin);
    }
    let bit = 1u32 << pin;
    Ok(if level { dout | bit } else { dout & !bit })
}

/// The level stored for `pin` in `dout`.
pub fn dout_of(dout: u32, pin: u8) -> Result<bool, PinError> {
    if pin >= PINS_PER_PORT {
        return Err(PinError::NoSuchPin);
    }
    Ok(dout & (1u32 << pin) != 0)
}

/// The value to write into a `*_ROUTE` register for `port`/`pin`: `port << 4 | pin`.
pub fn route(port: Port, pin: u8) -> Result<u32, PinError> {
    if pin >= PINS_PER_PORT {
        return Err(PinError::NoSuchPin);
    }
    Ok((port.code() << 4) | pin as u32)
}

/// The model value for each documented pin mode (`efr32mg24_gpio.h`, `MODEL n`).
///
/// Only the two this project uses are named; the rest are values, because a mode nobody has measured is a number with a
/// comment, not a promise.
pub mod model {
    /// Input, disabled, no pull.
    pub const DISABLED: u8 = 0;
    /// Push-pull output: drives high and low.
    pub const PUSHPULL: u8 = 4;
    /// Wired-and: open drain with the internal pull-up disengaged; the external pull-up does the work.
    pub const WIREDAND: u8 = 8;
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    /// Exhaustive: for every pin and every mode, only that pin's field changes.
    ///
    /// This is the property H17 broke, checked across the whole space rather than at one call site: 16 pins x 16 modes,
    /// with a background pattern that is not zero, because a read-modify-write bug hides behind zeros.
    #[test]
    fn setting_one_pin_never_touches_another() {
        // Two backgrounds of 0xA per field: a value no correct write should disturb.
        let background = Models::new(0xAAAA_AAAA, 0xAAAA_AAAA);
        assert_eq!(background.model_of(0).unwrap(), 0xA);
        assert_eq!(background.model_of(15).unwrap(), 0xA);
        for pin in 0..PINS_PER_PORT {
            for m in 0..=MODEL_MASK as u8 {
                let after = background.with(pin, m).expect("a valid pin and model");
                for other in 0..PINS_PER_PORT {
                    let expected = if other == pin {
                        m
                    } else {
                        background.model_of(other).unwrap()
                    };
                    assert_eq!(
                        after.model_of(other).unwrap(),
                        expected,
                        "pin {pin} set to {m} changed pin {other}"
                    );
                }
            }
        }
    }

    /// Applying the same change twice is the same as applying it once (idempotence), which a mask-based write has and a
    /// read-modify-write with a stale read does not.
    #[test]
    fn a_model_write_is_idempotent() {
        for pin in 0..PINS_PER_PORT {
            for m in 0..=MODEL_MASK as u8 {
                let once = Models::new(0x1234_5678, 0x9ABC_DEF0).with(pin, m).unwrap();
                let twice = once.with(pin, m).unwrap();
                assert_eq!(once, twice, "pin {pin} model {m} is not idempotent");
            }
        }
    }

    /// A pin does not exist above 15, and the refusal is the same in every operation.
    #[test]
    fn there_is_no_pin_sixteen() {
        assert_eq!(field_of(16), Err(PinError::NoSuchPin));
        assert_eq!(Models::default().with(16, 0), Err(PinError::NoSuchPin));
        assert_eq!(with_dout(0, 16, true), Err(PinError::NoSuchPin));
        assert_eq!(route(Port::A, 16), Err(PinError::NoSuchPin));
        assert_eq!(Models::default().model_of(200), Err(PinError::NoSuchPin));
        assert_eq!(dout_of(0, 255), Err(PinError::NoSuchPin));
    }

    /// A model value that does not fit four bits is refused, not truncated into a neighbouring pin's field.
    #[test]
    fn a_model_wider_than_the_field_is_refused() {
        assert_eq!(Models::default().with(0, 0x10), Err(PinError::ModelTooWide));
        assert!(
            Models::default().with(0, 0x0F).is_ok(),
            "fifteen still fits"
        );
    }

    /// `DOUT` is one bit per pin: setting a pin changes that bit and no other.
    #[test]
    fn dout_touches_one_bit() {
        let background = 0xFFFF_0000u32;
        for pin in 0..PINS_PER_PORT {
            let high = with_dout(background, pin, true).unwrap();
            let low = with_dout(background, pin, false).unwrap();
            for other in 0..PINS_PER_PORT {
                let before = dout_of(background, other).unwrap();
                if other == pin {
                    assert!(dout_of(high, other).unwrap(), "pin {pin} did not go high");
                    assert!(!dout_of(low, other).unwrap(), "pin {pin} did not go low");
                } else {
                    assert_eq!(
                        dout_of(high, other).unwrap(),
                        before,
                        "pin {pin} disturbed {other}"
                    );
                    assert_eq!(
                        dout_of(low, other).unwrap(),
                        before,
                        "pin {pin} disturbed {other}"
                    );
                }
            }
        }
    }

    /// The route encoding is `port << 4 | pin`, and it round-trips for every port and pin.
    #[test]
    fn the_route_encoding_round_trips() {
        for port in Port::ALL {
            for pin in 0..PINS_PER_PORT {
                let value = route(port, pin).unwrap();
                assert_eq!(
                    value >> 4,
                    port.code(),
                    "port {port:?} pin {pin}: port field"
                );
                assert_eq!(
                    (value & 0xF) as u8,
                    pin,
                    "port {port:?} pin {pin}: pin field"
                );
            }
        }
        assert_eq!(route(Port::C, 4).unwrap(), 0x24, "PC04 is 0x24");
        assert_eq!(route(Port::C, 5).unwrap(), 0x25, "PC05 is 0x25");
    }

    /// The documented model values are what the drivers use, and they are distinct.
    #[test]
    fn the_model_values_are_distinct_and_fit() {
        let modes = [model::DISABLED, model::PUSHPULL, model::WIREDAND];
        for m in modes {
            assert!(m as u32 <= MODEL_MASK, "a model value must fit four bits");
        }
        assert_ne!(model::DISABLED, model::PUSHPULL);
        assert_ne!(model::PUSHPULL, model::WIREDAND);
        assert_ne!(model::DISABLED, model::WIREDAND);
    }
}
