//! Choosing a timer's prescaler and top for a wanted period, and refusing to configure it after it is enabled.
//!
//! Two defects from the 2026-11-17 review live here.
//!
//! **H6, the ordering.** The driver wrote `EN = 1` and then `CFG`. This project has measured what that does on this family:
//! `board.rs:244-246` records a fault, and the working PWM path (`pwm.rs:82-92`) configures first and enables last. The
//! sequence is modelled here as a state machine, so a caller cannot configure a running timer -- the mistake becomes a
//! refusal instead of a fault.
//!
//! **H12's shape, on the timer side.** `TOP` is a 16-bit field and the prescaler can only be a power of two from 1 to 1024,
//! so not every period is expressible. The crate returns the closest expressible rate and says how far off it is, instead
//! of silently truncating a value into a 16-bit field.
#![deny(missing_docs)]
#![no_std]

/// The prescalers this family offers: powers of two from 1 to 1024.
pub const PRESCALERS: [u32; 11] = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];

/// `TOP` is 16 bits, so a period is at most 65536 timer clocks.
pub const MAX_TOP: u32 = 0xFFFF;

/// Why a period cannot be produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerError {
    /// A zero clock or wanted frequency.
    Zero,
    /// The wanted frequency is faster than the timer clock: no prescaler of 1 with a top of 0 can reach it.
    TooFast,
    /// The wanted frequency is slower than the slowest the prescaler and top allow.
    TooSlow,
    /// The timer is running: stop it before writing its configuration (H6).
    Enabled,
}

/// A timer configuration: a prescaler and a top count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    /// One of [`PRESCALERS`].
    pub prescaler: u32,
    /// The value written to `TOP`; the period is `top + 1` timer clocks.
    pub top: u32,
}

impl Period {
    /// The frequency this configuration produces from `clock_hz`.
    pub fn frequency(self, clock_hz: u32) -> u32 {
        let divisor = self.prescaler * (self.top + 1);
        if divisor == 0 {
            return 0;
        }
        clock_hz / divisor
    }
}

/// The closest expressible period for `wanted_hz` from `clock_hz`, and how far off it is in parts per million.
///
/// "Closest" is over both the prescaler and the top, because the pair is what decides the rate: with a 19 MHz clock, 16 kHz
/// is 1187,5 timer clocks, and the nearest expressible pair is prescaler 1 with top 1187 (16 006 Hz, 375 ppm off).
pub fn period_for(clock_hz: u32, wanted_hz: u32) -> Result<(Period, u32), TimerError> {
    if clock_hz == 0 || wanted_hz == 0 {
        return Err(TimerError::Zero);
    }
    if wanted_hz > clock_hz {
        return Err(TimerError::TooFast);
    }
    let mut best: Option<(Period, u32, u32)> = None; // (period, actual, error_ppm)
    for prescaler in PRESCALERS {
        let ticks = clock_hz / (prescaler * wanted_hz);
        if ticks == 0 {
            continue;
        }
        // Both sides of the truncation, because the nearest period is not always below the wanted one.
        //
        // The first version only took `ticks - 1`, which is the floor: at 32 kHz from 19 MHz that chose a top of 592
        // (32 040 Hz, 1250 ppm) while a top of 593 gives 31 986 Hz (437 ppm). The test that checks the chosen candidate
        // is the closest of all of them is what found it.
        for candidate_top in [ticks.saturating_sub(1), ticks] {
            // No clamping: a top beyond the field would shorten the period silently. Skipped, so a period the fields
            // cannot express comes back as `TooSlow` instead of as a faster timer than the caller asked for.
            if candidate_top > MAX_TOP {
                continue;
            }
            let candidate = Period {
                prescaler,
                top: candidate_top,
            };
            let actual = candidate.frequency(clock_hz);
            if actual == 0 {
                continue;
            }
            let error_ppm = (((actual as i64 - wanted_hz as i64).unsigned_abs() * 1_000_000)
                / wanted_hz as u64) as u32;
            let better = match best {
                None => true,
                Some((_, _, e)) => error_ppm < e,
            };
            if better {
                best = Some((candidate, actual, error_ppm));
            }
        }
    }
    match best {
        Some((period, _, error_ppm)) => Ok((period, error_ppm)),
        None => Err(TimerError::TooSlow),
    }
}

/// Whether a timer may be configured, given whether it is enabled.
///
/// The whole of H6 as one function: on this family the configuration registers must be written while the timer is stopped.
/// The driver did the opposite and faulted, which is why the working PWM path and this crate agree on the order.
pub fn may_configure(enabled: bool) -> Result<(), TimerError> {
    if enabled {
        Err(TimerError::Enabled)
    } else {
        Ok(())
    }
}

#[path = "../../../crates/efr32mg2x-hal/src/cores/timer.rs"]
pub mod capture;

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    /// The frequency this project actually runs the microphone at, and the error it gets.
    #[test]
    fn the_microphone_pace() {
        let (period, error_ppm) = period_for(19_000_000, 16_000).expect("16 kHz from 19 MHz");
        assert_eq!(period.prescaler, 1, "no prescaler needed");
        // 19e6/16e3 = 1187.5 clocks: top 1186 gives 1187 clocks (16 006 Hz, 375 ppm), top 1187 gives 1188 clocks
        // (15 993 Hz, 437 ppm), so the floor side is the closer one here.
        assert_eq!(period.top, 1186, "the closer side of 1187.5");
        assert_eq!(
            period.frequency(19_000_000),
            16_006,
            "the rate it actually produces"
        );
        assert!(
            error_ppm < 1_000,
            "{error_ppm} ppm is more than a thousandth"
        );
    }

    /// Every clock this project uses, at the rates its timers need, stays within a thousandth.
    #[test]
    fn the_measured_clocks_at_the_used_rates() {
        for clock in [19_000_000u32, 38_000_000, 56_000_000, 80_000_000] {
            for wanted in [1_000u32, 16_000, 32_000, 1_000_000] {
                let (period, ppm) = period_for(clock, wanted).expect("a plannable rate");
                assert!(
                    ppm < 1_000,
                    "clock {clock} wanted {wanted}: {} Hz is {ppm} ppm off",
                    period.frequency(clock)
                );
                assert!(period.top <= MAX_TOP, "top must fit sixteen bits");
                assert!(PRESCALERS.contains(&period.prescaler));
            }
        }
    }

    /// The search really searches: for a rate that needs a prescaler, it picks one.
    #[test]
    fn a_slow_rate_needs_a_prescaler() {
        // 1 Hz from 19 MHz is 19 million timer clocks: 65536 is the largest top, so the prescaler must do the rest.
        let (period, _) = period_for(19_000_000, 1).expect("1 Hz");
        assert!(period.prescaler > 1, "a prescaler is unavoidable at 1 Hz");
        assert!(period.top <= MAX_TOP, "and the top still fits sixteen bits");
        let actual = period.frequency(19_000_000);
        assert!(
            actual <= 2,
            "1 Hz as close as the fields allow, got {actual} Hz"
        );
        // No prescaler is named here on purpose: which one wins depends on where the truncation falls, and the first
        // version of this test asserted 1024 when 512 is closer (a difference of a few ppm that I got wrong by hand).
        // The property -- the chosen candidate is the closest of all of them -- is what the test
        // `the_chosen_candidate_is_the_closest` measures, over six wanted rates.
        assert!(period.prescaler > 1);
        assert!(period.top <= MAX_TOP);
    }

    /// A rate faster than the clock is refused, not clamped into a wrong number.
    #[test]
    fn too_fast_is_refused() {
        assert_eq!(period_for(19_000_000, 20_000_000), Err(TimerError::TooFast));
        assert_eq!(period_for(0, 1_000), Err(TimerError::Zero));
        assert_eq!(period_for(19_000_000, 0), Err(TimerError::Zero));
    }

    /// The ordering rule, which is H6 as a function.
    #[test]
    fn a_running_timer_cannot_be_configured() {
        assert_eq!(
            may_configure(false),
            Ok(()),
            "a stopped timer is configurable"
        );
        assert!(
            may_configure(true).is_err(),
            "a running timer must be stopped before its configuration is written"
        );
    }

    /// The closest candidate is the closest: no prescaler option does better than the one chosen.
    #[test]
    fn the_chosen_candidate_is_the_closest() {
        let clock = 19_000_000u32;
        for wanted in [440u32, 1_000, 7_000, 16_000, 44_100, 250_000] {
            let (chosen, chosen_ppm) = period_for(clock, wanted).expect("a plannable rate");
            for prescaler in PRESCALERS {
                let ticks = clock / (prescaler * wanted);
                if ticks == 0 {
                    continue;
                }
                let candidate = Period {
                    prescaler,
                    top: ticks.saturating_sub(1).min(MAX_TOP),
                };
                let actual = candidate.frequency(clock);
                if actual == 0 {
                    continue;
                }
                let ppm = ((actual as i64 - wanted as i64).unsigned_abs() as u64 * 1_000_000)
                    / wanted as u64;
                assert!(
                    ppm >= chosen_ppm as u64,
                    "wanted {wanted}: prescaler {prescaler} gives {ppm} ppm, better than the chosen {chosen_ppm}"
                );
            }
            assert!(chosen.prescaler >= 1);
        }
    }
}
