//! Routing a peripheral resource to a physical pin.
//!
//! ## Why this exists
//!
//! On this part a peripheral works perfectly while reaching no pin at all: the signal has to be
//! pointed at a port and a pin, and then enabled. Until both happen, the driver looks alive and the
//! wire is silent. That is the defect the product has been living with -- `i2c`, `usart` and `spi`
//! worked internally and never touched a pad.
//!
//! The registers are not on the peripheral: they are on the GPIO block, which is why looking for
//! `i2c0_ns/route.rs` finds nothing. The manual says it plainly (EFR32xG24 RM, page 816): to route
//! I2C0's SDA to PB03, write PORT = 1 and PIN = 3 into `GPIO_I2C0_SDAROUTE`, then set
//! `GPIO_I2C0_ROUTEEN.SDAPEN`. It also says the pin itself must be configured for the resource --
//! open-drain for an I2C line.
//!
//! ## What is pure here, and what is not
//!
//! The arithmetic -- which bits carry the port, which carry the pin, whether a given port/pin pair
//! exists, and what number means open-drain -- is pure and tested on the host. Writing those values
//! into the registers is the driver's job and needs the part.

/// `PORT` occupies bits [1:0] of a route register: PA = 0, PB = 1, PC = 2, PD = 3.
pub const PORT_SHIFT: u32 = 0;
/// Two bits, so four ports -- and a port number above three must be refused, not masked.
pub const PORT_MAX: u8 = 3;
/// `PIN` occupies bits [19:16]: pin 0 to 15 of the chosen port.
pub const PIN_SHIFT: u32 = 16;
/// Four bits, so pins 0 to 15.
pub const PIN_MAX: u8 = 15;

/// A port/pin pair that cannot exist on this part.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RouteError {
    /// The part has ports PA to PD; anything above is a typo, not a fourth bit.
    PortDoesNotExist(u8),
    /// A port has sixteen pins; a sixteenth pin is a typo.
    PinDoesNotExist(u8),
}

/// How a pin is driven.
///
/// Only the modes whose numbering was read out of the reference manual are here. The manual's
/// `GPIO_Px_MODEL.MODEn` table has many entries and the rest were not measured, so they are not
/// invented: a wrong number here drives a pin the wrong way, and the symptom is a bus that almost
/// works. Add a variant when its number comes from the manual, not from a guess.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PinMode {
    /// Open-drain output, no pull-up of its own -- an I2C line with external pull-ups. Manual: 8.
    OpenDrain,
    /// Open-drain with the pin's own pull-up -- an I2C line on a board that has none. Manual: 10.
    OpenDrainPullUp,
    /// Open-drain with filter and pull-up. Manual: 11.
    OpenDrainPullUpFilter,
}

impl PinMode {
    /// The value for `MODEn`, as the manual numbers it.
    pub fn value(self) -> u8 {
        match self {
            PinMode::OpenDrain => 8,
            PinMode::OpenDrainPullUp => 10,
            PinMode::OpenDrainPullUpFilter => 11,
        }
    }
}

/// The value to write into a route register for `port`/`pin`.
///
/// Returns an error rather than masking: silently turning port 5 into port 1 would route the signal
/// to a different pin than the caller asked for, and the wire would stay silent while the register
/// looked plausible.
pub fn route_value(port: u8, pin: u8) -> Result<u32, RouteError> {
    if port > PORT_MAX {
        return Err(RouteError::PortDoesNotExist(port));
    }
    if pin > PIN_MAX {
        return Err(RouteError::PinDoesNotExist(pin));
    }
    Ok((u32::from(port) << PORT_SHIFT) | (u32::from(pin) << PIN_SHIFT))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The worked example from the reference manual, page 816: I2C0 SDA on PB03.
    #[test]
    fn the_manual_example_is_reproduced() {
        assert_eq!(route_value(1, 3).unwrap(), 0x0003_0001);
    }

    #[test]
    fn pa00_is_all_zeroes() {
        assert_eq!(route_value(0, 0).unwrap(), 0);
    }

    #[test]
    fn the_last_port_and_pin_land_in_their_fields() {
        // PD15: port 3 in [1:0], pin 15 in [19:16].
        assert_eq!(route_value(3, 15).unwrap(), 0x000F_0003);
    }

    #[test]
    fn a_port_that_does_not_exist_is_refused() {
        assert_eq!(route_value(4, 0), Err(RouteError::PortDoesNotExist(4)));
    }

    #[test]
    fn a_pin_that_does_not_exist_is_refused() {
        assert_eq!(route_value(0, 16), Err(RouteError::PinDoesNotExist(16)));
    }

    /// The near-miss: the largest legal pair must still take the normal path, so the two tests above
    /// are not passing simply because everything is refused.
    #[test]
    fn the_largest_legal_pair_is_accepted() {
        assert!(route_value(PORT_MAX, PIN_MAX).is_ok());
    }

    /// The I2C lines are open-drain, and the manual numbers that mode 8 (10 with pull-up). Recording
    /// the numbers here means the driver cannot drift from the manual silently.
    #[test]
    fn open_drain_is_the_manual_number() {
        assert_eq!(PinMode::OpenDrain.value(), 8);
        assert_eq!(PinMode::OpenDrainPullUp.value(), 10);
    }

    /// A variant added later must not silently take a number another one already uses. Without this,
    /// a typo in the table would look fine and drive the pin wrongly.
    #[test]
    fn no_two_modes_share_a_number() {
        let modos = [
            PinMode::OpenDrain,
            PinMode::OpenDrainPullUp,
            PinMode::OpenDrainPullUpFilter,
        ];
        for (i, a) in modos.iter().enumerate() {
            for b in &modos[i + 1..] {
                assert_ne!(a.value(), b.value(), "{a:?} and {b:?} share a value");
            }
        }
    }
}
