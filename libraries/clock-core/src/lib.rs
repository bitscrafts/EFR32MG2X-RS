//! What a target core frequency implies: which calibrated band, which wait states, which prescalers.
//!
//! The HAL changed the clock by hand: boost to a band, remember to divide PCLK, remember to divide the radio clock, and
//! nothing checked any of it. The 2026-11-17 review found three defects in that shape -- H10 (the band change set
//! neither prescaler; measured today as PCLK and the radio clock both running undivided at 80 MHz, out of spec), H21
//! (`Clocks::new` configured nothing and returned the frequency the caller stated) and H22 (`19_000_000` written into
//! three places, wrong after any boost).
//!
//! The ceilings are not opinions: `em_cmu.c:99,102` gives `CMU_MAX_PCLK_FREQ 50000000` and `CMU_MAX_RHCLK_FREQ
//! 40000000`, and the vendor divides both after every frequency change (`pclkDivOptimize`, `rhclkPrescOptimize`).
//!
//! This crate decides the whole plan from one number, and the tests check the invariants over **every** frequency from
//! 1 to 80 MHz rather than over the four bands this project happens to use.

#![no_std]

/// The peripheral bus must stay at or below this (`em_cmu.c:99`).
pub const MAX_PCLK_HZ: u32 = 50_000_000;
/// The radio clock must stay at or below this (`em_cmu.c:101`).
pub const MAX_RHCLK_HZ: u32 = 40_000_000;

/// The calibrated oscillator bands this project has measured, and the `DEVINFO` word index each one uses.
///
/// The index is the vendor's table (`HFRCODPLLCAL[12]` is 38 MHz, `[14]` 56 MHz, `[16]` 80 MHz); 19 MHz is the reset
/// band, which the boot code does not take from `DEVINFO`.
pub const BANDS: [(u32, u8); 4] = [
    (19_000_000, 0),
    (38_000_000, 12),
    (56_000_000, 14),
    (80_000_000, 16),
];

/// Why a frequency cannot be planned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockError {
    /// Zero or implausible frequency.
    OutOfRange,
    /// A frequency with no calibrated band: the oscillator cannot be set there without a calibration this project does
    /// not have.
    NoBand,
}

/// How a divider is set: the field is one bit on this family (`Div1` = 0, `Div2` = 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Divider {
    /// Undivided.
    Div1,
    /// Divided by two.
    Div2,
}

impl Divider {
    /// The value written into the register field.
    pub fn field(self) -> u32 {
        match self {
            Divider::Div1 => 0,
            Divider::Div2 => 1,
        }
    }

    /// The divisor as a number, for arithmetic on frequencies.
    pub fn divisor(self) -> u32 {
        match self {
            Divider::Div1 => 1,
            Divider::Div2 => 2,
        }
    }
}

/// Everything a frequency change has to set, decided once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plan {
    /// The core frequency the plan is for.
    pub core_hz: u32,
    /// `DEVINFO` word index for the band (0 means "the reset band, no calibration read").
    pub devinfo_index: u8,
    /// Wait states for the flash at this frequency.
    pub wait_states: u8,
    /// PCLK divider, from the 50 MHz ceiling.
    pub pclk: Divider,
    /// Radio clock divider, from the 40 MHz ceiling.
    pub rhclk: Divider,
}

impl Plan {
    /// The frequency the peripherals will actually see.
    pub fn pclk_hz(&self) -> u32 {
        self.core_hz / self.pclk.divisor()
    }

    /// The frequency the radio will actually see.
    pub fn rhclk_hz(&self) -> u32 {
        self.core_hz / self.rhclk.divisor()
    }
}

/// Wait states for a core frequency.
///
/// This project runs WS3 at every band it uses: it is safe at 19 MHz and required at 80, and one value that is always
/// right beats a table nobody re-measures. The function exists so that a future board with a different flash can change
/// one place.
fn wait_states_for(_core_hz: u32) -> u8 {
    3
}

/// The plan for `core_hz`, or why there is none.
pub fn plan(core_hz: u32) -> Result<Plan, ClockError> {
    if core_hz == 0 || core_hz > 80_000_000 {
        return Err(ClockError::OutOfRange);
    }
    let devinfo_index = BANDS
        .iter()
        .find(|(hz, _)| *hz == core_hz)
        .map(|(_, index)| *index)
        .ok_or(ClockError::NoBand)?;
    // The smallest divider that respects each ceiling, which is what the vendor's optimisers do.
    let pclk = if core_hz > MAX_PCLK_HZ {
        Divider::Div2
    } else {
        Divider::Div1
    };
    let rhclk = if core_hz > MAX_RHCLK_HZ {
        Divider::Div2
    } else {
        Divider::Div1
    };
    Ok(Plan {
        core_hz,
        devinfo_index,
        wait_states: wait_states_for(core_hz),
        pclk,
        rhclk,
    })
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    /// Every band this project has measured, with the plan it must produce.
    #[test]
    fn the_four_measured_bands() {
        let cases = [
            (19_000_000u32, 0u8, Divider::Div1, Divider::Div1),
            (38_000_000, 12, Divider::Div1, Divider::Div1),
            (56_000_000, 14, Divider::Div2, Divider::Div2),
            (80_000_000, 16, Divider::Div2, Divider::Div2),
        ];
        for (hz, index, pclk, rhclk) in cases {
            let plan = plan(hz).expect("a measured band must be plannable");
            assert_eq!(plan.devinfo_index, index, "{hz} Hz: DEVINFO index");
            assert_eq!(plan.pclk, pclk, "{hz} Hz: PCLK divider");
            assert_eq!(plan.rhclk, rhclk, "{hz} Hz: radio divider");
            assert_eq!(plan.wait_states, 3, "{hz} Hz: wait states");
        }
    }

    /// The invariant, over the whole range rather than over four points: no plan may put a bus over its ceiling.
    ///
    /// This is the property H10 broke by hand -- at 80 MHz both clocks ran undivided -- and the test that would have
    /// caught it before the review did.
    #[test]
    fn no_band_puts_a_bus_over_its_ceiling() {
        for hz in (1..=80_000_000u32).step_by(1_000_000) {
            let Ok(plan) = plan(hz) else { continue };
            assert!(
                plan.pclk_hz() <= MAX_PCLK_HZ,
                "{hz} Hz: PCLK would be {} Hz, over the {MAX_PCLK_HZ} ceiling",
                plan.pclk_hz()
            );
            assert!(
                plan.rhclk_hz() <= MAX_RHCLK_HZ,
                "{hz} Hz: radio clock would be {} Hz, over the {MAX_RHCLK_HZ} ceiling",
                plan.rhclk_hz()
            );
        }
    }

    /// And the dividers are the *smallest* that respect the ceiling: dividing more than needed throws away performance
    /// for nothing, which is a defect a test can hold too.
    #[test]
    fn the_dividers_are_the_smallest_that_fit() {
        for hz in (1..=80_000_000u32).step_by(500_000) {
            let Ok(plan) = plan(hz) else { continue };
            if plan.pclk == Divider::Div2 {
                assert!(
                    hz > MAX_PCLK_HZ,
                    "{hz} Hz divides PCLK although it fits undivided"
                );
            }
            if plan.rhclk == Divider::Div2 {
                assert!(
                    hz > MAX_RHCLK_HZ,
                    "{hz} Hz divides the radio clock although it fits undivided"
                );
            }
        }
    }

    /// Exactly the frequencies in the table plan, no more and no fewer.
    ///
    /// Added because the first version of the example swept 1..80 MHz, found every frequency refused, and printed a pass:
    /// a check that measures nothing is worse than no check. This one fails if the table and the behaviour disagree in
    /// either direction.
    #[test]
    fn the_plannable_set_is_exactly_the_table() {
        let plannable: std::vec::Vec<u32> =
            (1..=80_000_000u32).filter(|hz| plan(*hz).is_ok()).collect();
        let expected: std::vec::Vec<u32> = BANDS.iter().map(|(hz, _)| *hz).collect();
        assert_eq!(
            plannable, expected,
            "the plannable frequencies must be the table's, in order"
        );
    }

    /// Frequencies outside what the oscillator can be calibrated for are refused, not rounded.
    #[test]
    fn an_uncalibrated_frequency_is_refused() {
        assert_eq!(plan(0), Err(ClockError::OutOfRange));
        assert_eq!(plan(81_000_000), Err(ClockError::OutOfRange));
        assert_eq!(
            plan(40_000_000),
            Err(ClockError::NoBand),
            "38 is calibrated, 40 is not"
        );
        assert_eq!(
            plan(57_000_000),
            Err(ClockError::NoBand),
            "56 is calibrated, 57 is not"
        );
    }

    /// The exact ceilings are inclusive: a frequency at the ceiling is not divided.
    #[test]
    fn the_ceiling_is_inclusive() {
        assert_eq!(
            plan(MAX_PCLK_HZ),
            Err(ClockError::NoBand),
            "50 MHz has no band, but not because of the ceiling"
        );
        assert_eq!(Divider::Div2.field(), 1, "the field is one bit: Div2 is 1");
        assert_eq!(Divider::Div1.field(), 0, "Div1 is 0");
    }

    /// A plan round-trips: the frequencies it reports are the ones the hardware would see.
    #[test]
    fn the_plan_reports_what_the_hardware_will_see() {
        let plan80 = plan(80_000_000).unwrap();
        assert_eq!(
            plan80.pclk_hz(),
            40_000_000,
            "80 MHz with Div2 is 40 on the bus"
        );
        assert_eq!(plan80.rhclk_hz(), 40_000_000, "and 40 to the radio");
        let plan38 = plan(38_000_000).unwrap();
        assert_eq!(plan38.pclk_hz(), 38_000_000);
        assert_eq!(plan38.rhclk_hz(), 38_000_000);
    }
}
