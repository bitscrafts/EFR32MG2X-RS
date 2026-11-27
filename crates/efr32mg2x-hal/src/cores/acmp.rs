//! The `ACMP` reference ladder as arithmetic, so the driver's numbers have a host test.
//!
//! `INPUTCTRL.VREFDIV` is not a divider in the sense the name suggests: it is the tap of a 63-step ladder, and
//! `VREFOUT = VREFIN * VREFDIV / 63`, with `63` the pass-through tap (`em_acmp.h`: "Set VREFDIV to maximum to disable
//! divide"). The HAL cannot be compiled for the host -- it contains ARM `asm!` -- so the sweep arithmetic that turns a
//! ladder index into a voltage lives here, where a test runs it in microseconds.
//!
//! What this file does **not** claim: absolute accuracy. The ladder is ratiometric to its own reference and both come
//! from the same bandgap, so a number computed here is "what the ladder should give if the reference is what its name
//! says", and that is exactly how the hardware record uses it -- as an interval, never as a calibrated voltmeter.
//!
//! `libraries/acmp-core` compiles this file by path so the tests below run on the host.

/// The pass-through tap: `VREFDIV = 63` divides by one.
pub const LADDER_FULL: u8 = 63;

/// The ladder tap in millivolts: `VREFIN * div / 63`.
///
/// `div` is six bits, so any value above [`LADDER_FULL`] is clamped to it: a caller that computed an index cannot ask
/// for a voltage the field cannot express, and clamping is visible in the return value instead of being a silent wrap.
#[must_use]
pub fn ladder_mv(reference_mv: u32, div: u8) -> u32 {
    let div = if div > LADDER_FULL { LADDER_FULL } else { div };
    // 63 fits in u32 and reference_mv is a millivolt figure below 5 V, so the product cannot overflow.
    reference_mv * u32::from(div) / u32::from(LADDER_FULL)
}

/// The interval a compared signal must lie in, given the tap at which the sweep first reads *low*.
///
/// A sweep of a fixed positive input against a rising ladder reads high while the tap is below the input and low once
/// it is above, so the first low tap `n` brackets the input between `ladder(n - 1)` and `ladder(n)`. The window is
/// open at both ends; `None` means the sweep never turned, which is not an interval but a failed measurement.
#[must_use]
pub fn window_mv(reference_mv: u32, first_low_div: u8) -> Option<(u32, u32)> {
    if first_low_div == 0 || first_low_div > LADDER_FULL {
        return None;
    }
    Some((
        ladder_mv(reference_mv, first_low_div - 1),
        ladder_mv(reference_mv, first_low_div),
    ))
}

/// The overlap of two windows, or `None` when they disagree.
///
/// Two sweeps of the same signal against two different ladder references are two independent measurements of one
/// voltage. They can only both be right if their intervals intersect; an empty intersection means one of the sweeps
/// (or one of the references) is not what the record says it is.
#[must_use]
pub fn overlap(a: (u32, u32), b: (u32, u32)) -> Option<(u32, u32)> {
    let lo = if a.0 > b.0 { a.0 } else { b.0 };
    let hi = if a.1 < b.1 { a.1 } else { b.1 };
    if lo <= hi {
        Some((lo, hi))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    #[test]
    fn the_full_tap_is_the_reference_itself() {
        assert_eq!(ladder_mv(1_250, LADDER_FULL), 1_250);
        assert_eq!(ladder_mv(2_500, LADDER_FULL), 2_500);
    }

    #[test]
    fn the_zero_tap_is_zero_volts() {
        assert_eq!(ladder_mv(1_250, 0), 0);
    }

    #[test]
    fn a_tap_above_the_field_is_clamped_not_wrapped() {
        // 64 is 63 + 1: a wrap would give 1.25 V * 64 mod 64, the arithmetic here gives the pass-through tap.
        assert_eq!(ladder_mv(1_250, 64), 1_250);
        assert_eq!(ladder_mv(1_250, 255), 1_250);
    }

    #[test]
    fn the_window_brackets_the_signal_between_two_adjacent_taps() {
        // 1.25 V / 63 = 19.84 mV per step.
        assert_eq!(window_mv(1_250, 42), Some((813, 833)));
        assert_eq!(window_mv(1_250, 1), Some((0, 19)));
        assert_eq!(window_mv(1_250, 63), Some((1_230, 1_250)));
    }

    #[test]
    fn a_sweep_that_never_turned_is_not_a_window() {
        assert_eq!(window_mv(1_250, 0), None);
        assert_eq!(window_mv(1_250, 64), None);
    }

    #[test]
    fn two_windows_that_agree_intersect_and_two_that_do_not_do_not() {
        assert_eq!(overlap((813, 833), (793, 833)), Some((813, 833)));
        assert_eq!(overlap((813, 833), (900, 950)), None);
    }
}
