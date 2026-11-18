//! What one IADC single conversion selects, as a register word, with the one thing this project could not measure
//! named as such.
//!
//! The defect this closes, from the 2026-11-17 review (H13): `Adc::read` hard-coded `portpos = 8`, which is Port A, so
//! every channel was a Port A pin whatever the caller asked for -- while the board's own analogue input is PC9, the
//! microphone. A wrong port does not fail: it reads a different pin and returns a plausible number.
//!
//! The field layout is the vendor's (`efr32mg24_iadc.h:935-963`): `PINNEG` at bit 0, `PORTNEG` at 4, `PINPOS` at 8,
//! `PORTPOS` at 12, four bits each. A port's code is `8 + index` (Port A is 8) and ground is 0, which the PAC confirms by
//! having `portpos().porta()` and `portneg().gnd()` as the variants the working driver uses.
//!
//! **What is not here, and why.** Routing a pin to the ADC also involves `GPIO->ABUSALLOC`, whose fields select analogue
//! *buses* (`adc0`, `aodd0`, `aeven0`, ...) rather than pins. Which bus a pin sits on is a table in the reference manual,
//! and this project does not have the manual. What it does have is one measured input: the microphone on PC9 works, and
//! its driver writes no `ABUSALLOC` at all (`microphone.rs:205-214`). That is a single data point about one pin in one
//! configuration, and it is recorded here as a data point rather than generalised into a table.

/// The four ports, in the order the `PORTPOS` field encodes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Port {
    /// Port A, code 8.
    A,
    /// Port B, code 9.
    B,
    /// Port C, code 10.
    C,
    /// Port D, code 11.
    D,
}

impl Port {
    /// The `PORTPOS` code: Port A is 8, then one per port.
    pub fn code(self) -> u32 {
        match self {
            Port::A => 8,
            Port::B => 9,
            Port::C => 10,
            Port::D => 11,
        }
    }

    /// The port for a code, if it is one this enum names.
    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            8 => Some(Port::A),
            9 => Some(Port::B),
            10 => Some(Port::C),
            11 => Some(Port::D),
            _ => None,
        }
    }

    /// Every port, for exhaustive tests.
    pub const ALL: [Port; 4] = [Port::A, Port::B, Port::C, Port::D];
}

/// The `PORTNEG`/`PINNEG` code for the negative input when it is ground.
pub const GROUND: u32 = 0;

/// Field shifts in the `SINGLE` register (`efr32mg24_iadc.h`).
pub const PINNEG_SHIFT: u32 = 0;
/// Field shift for the negative port.
pub const PORTNEG_SHIFT: u32 = 4;
/// Field shift for the positive pin.
pub const PINPOS_SHIFT: u32 = 8;
/// Field shift for the positive port.
pub const PORTPOS_SHIFT: u32 = 12;

/// A pin, as a conversion names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Input {
    /// Which port.
    pub port: Port,
    /// Which pin, 0..15.
    pub pin: u8,
}

impl Input {
    /// An input, or why it cannot exist.
    pub fn new(port: Port, pin: u8) -> Result<Self, AdcError> {
        if pin >= 16 {
            return Err(AdcError::NoSuchPin);
        }
        Ok(Self { port, pin })
    }

    /// The `SINGLE` register value for this input, single-ended against ground.
    pub fn single_word(self) -> u32 {
        (self.port.code() << PORTPOS_SHIFT)
            | ((self.pin as u32) << PINPOS_SHIFT)
            | (GROUND << PORTNEG_SHIFT)
            | (GROUND << PINNEG_SHIFT)
    }
}

/// Why a conversion cannot be described.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdcError {
    /// There is no pin 16 on a 16-pin port.
    NoSuchPin,
    /// The positive port field does not name a port: ground, or a reserved value.
    NotAPort,
    /// The word describes a differential pair, which this crate does not model.
    Differential,
}

/// The input a `SINGLE` word describes, or why it does not describe one.
///
/// The round trip is the point: a value written by one function and read back by another is the only way to know the two
/// agree, and an encoder nobody decodes can be wrong in the same direction as its own test.
pub fn parse(word: u32) -> Result<Input, AdcError> {
    if (word >> PORTNEG_SHIFT) & 0xF != GROUND {
        return Err(AdcError::Differential);
    }
    let port_code = (word >> PORTPOS_SHIFT) & 0xF;
    let port = Port::from_code(port_code).ok_or(AdcError::NotAPort)?;
    let pin = ((word >> PINPOS_SHIFT) & 0xF) as u8;
    Input::new(port, pin)
}

/// What this project knows about the analogue bus for a pin.
///
/// Deliberately not a table of guesses. `Measured` carries the pins whose input has actually produced numbers on the
/// board; `Unknown` is every other pin, and a caller that needs one has to go and measure it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bus {
    /// This project has measured this pin as an ADC input, and in that configuration no `ABUSALLOC` write was needed.
    MeasuredWithoutAllocation,
    /// Nothing is known about routing this pin to the ADC.
    Unknown,
}

/// The bus knowledge for `pin`, by measurement rather than by table.
///
/// One entry: PC9, the microphone. `microphone.rs:205-214` configures `SINGLE` with `portpos().portc()` and
/// `pinpos(9)` and writes no `ABUSALLOC`, and the driver produces readings (`rms=12 pk=29` at one window, and a DC bias
/// of 2391). Every other pin is `Unknown`, including pins that may well work -- the point is that this crate does not
/// say so.
pub fn bus_for(input: Input) -> Bus {
    if input.port == Port::C && input.pin == 9 {
        Bus::MeasuredWithoutAllocation
    } else {
        Bus::Unknown
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    /// The encoding the working driver uses, checked against the register.
    #[test]
    fn the_measured_input_matches_the_working_driver() {
        let input = Input::new(Port::C, 9).unwrap();
        let word = input.single_word();
        assert_eq!((word >> PORTPOS_SHIFT) & 0xF, 10, "Port C is 10");
        assert_eq!((word >> PINPOS_SHIFT) & 0xF, 9, "pin 9");
        assert_eq!((word >> PORTNEG_SHIFT) & 0xF, 0, "negative is ground");
        assert_eq!((word >> PINNEG_SHIFT) & 0xF, 0, "and its pin is ignored");
        assert_eq!(parse(word), Ok(input), "the word round-trips");
    }

    /// Exhaustive: every port and every pin round-trips, and no two inputs share a word.
    #[test]
    fn every_port_and_pin_round_trips_and_no_two_collide() {
        let mut seen: std::vec::Vec<u32> = std::vec::Vec::new();
        for port in Port::ALL {
            for pin in 0..16u8 {
                let input = Input::new(port, pin).unwrap();
                let word = input.single_word();
                assert_eq!(parse(word), Ok(input), "{port:?}{pin} did not round-trip");
                assert!(
                    !seen.contains(&word),
                    "{port:?}{pin} shares a word with an earlier input"
                );
                seen.push(word);
            }
        }
        assert_eq!(seen.len(), 64, "four ports x sixteen pins");
    }

    /// Ground is not a port: a word with `PORTPOS = 0` describes the internal ground, not a pin.
    #[test]
    fn ground_is_not_a_port() {
        let ground_word = 0u32;
        assert_eq!(parse(ground_word), Err(AdcError::NotAPort));
    }

    /// A differential word is refused rather than silently read as single-ended.
    #[test]
    fn a_differential_word_is_refused() {
        let mut word = Input::new(Port::A, 3).unwrap().single_word();
        word |= 9 << PORTNEG_SHIFT; // a negative port, which this crate does not model
        assert_eq!(parse(word), Err(AdcError::Differential));
    }

    /// There is no pin 16, and the refusal comes before any arithmetic.
    #[test]
    fn there_is_no_pin_sixteen() {
        assert_eq!(Input::new(Port::A, 16), Err(AdcError::NoSuchPin));
        assert_eq!(Input::new(Port::C, 200), Err(AdcError::NoSuchPin));
    }

    /// The bus knowledge is one measured pin and nothing else -- a test that the crate does not pretend to know a table
    /// it does not have.
    #[test]
    fn the_bus_knowledge_is_one_measurement() {
        assert_eq!(
            bus_for(Input::new(Port::C, 9).unwrap()),
            Bus::MeasuredWithoutAllocation
        );
        let mut unknown = 0;
        for port in Port::ALL {
            for pin in 0..16u8 {
                let input = Input::new(port, pin).unwrap();
                if bus_for(input) == Bus::Unknown {
                    unknown += 1;
                }
            }
        }
        assert_eq!(unknown, 63, "exactly one of the 64 inputs is measured");
    }
}
