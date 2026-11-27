//! The watchdog's timeout encoding, and the rule that its lock is taken only while it is running.
//!
//! The defect this closes, from the 2026-11-17 review (H27): the driver wrote `LOCK` while the watchdog was disabled. On
//! this family `LOCK` is write-once: setting it while the watchdog is off leaves the peripheral locked with no timeout
//! programmed, and the only way back is a reset. The review graded it C and named `hal/wdog/mod.rs:126`.
//!
//! The other half is the arithmetic. A watchdog period is `(prescaler * (top + 1)) / clock`, with the same fields as any
//! timer, and the point of a watchdog is that the number it was given is the number it uses -- a period that silently
//! truncates is a reset nobody can explain.
#![deny(missing_docs)]
#![no_std]

/// The prescalers this family offers for the watchdog: powers of two from 1 to 32768.
pub const PRESCALERS: [u32; 16] = [
    1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16_384, 32_768,
];

/// `TOP` is 16 bits.
pub const MAX_TOP: u32 = 0xFFFF;

/// Why a watchdog period cannot be produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchdogError {
    /// A zero clock or a zero wanted period.
    Zero,
    /// The wanted period is shorter than the fastest the fields allow.
    TooShort,
    /// The wanted period is longer than the slowest the fields allow.
    TooLong,
    /// The lock cannot be taken in this state.
    CannotLock(&'static str),
}

/// A watchdog configuration, in the units the registers take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    /// One of [`PRESCALERS`].
    pub prescaler: u32,
    /// The value written to `TOP`; the period is `top + 1` clocks of the divided clock.
    pub top: u32,
}

impl Period {
    /// The period in milliseconds this configuration produces from `clock_hz`, rounded down.
    ///
    /// The watchdog's job is to reset a hung system, so the useful number is the one it will actually use, not the one
    /// that was requested.
    pub fn period_ms(self, clock_hz: u32) -> u32 {
        let clocks = (self.prescaler as u64) * (self.top as u64 + 1);
        if clock_hz == 0 {
            return 0;
        }
        ((clocks * 1000) / clock_hz as u64) as u32
    }
}

/// The closest expressible period for `wanted_ms`, and how far off it is in parts per thousand.
pub fn period_for_ms(clock_hz: u32, wanted_ms: u32) -> Result<(Period, u32), WatchdogError> {
    if clock_hz == 0 || wanted_ms == 0 {
        return Err(WatchdogError::Zero);
    }
    let wanted_clocks = (clock_hz as u64 * wanted_ms as u64) / 1000;
    let mut best: Option<(Period, u32)> = None; // (period, error per thousand)
    for prescaler in PRESCALERS {
        let ticks = wanted_clocks / prescaler as u64;
        if ticks == 0 {
            continue;
        }
        for candidate_top in [ticks.saturating_sub(1), ticks] {
            // A top beyond the field is not clamped: clamping shortens the period silently, and a watchdog that resets
            // earlier than it was told is the defect this crate exists to prevent. The candidate is skipped, and if no
            // prescaler can express the period the caller gets `TooLong`.
            if candidate_top > MAX_TOP as u64 {
                continue;
            }
            let period = Period {
                prescaler,
                top: candidate_top as u32,
            };
            let actual = period.period_ms(clock_hz);
            if actual == 0 {
                continue;
            }
            let error =
                ((actual as i64 - wanted_ms as i64).unsigned_abs() * 1000) / wanted_ms as u64;
            let better = match best {
                None => true,
                Some((_, e)) => error < e as u64,
            };
            if better {
                best = Some((period, error as u32));
            }
        }
    }
    match best {
        Some((period, error)) => Ok((period, error)),
        None => Err(WatchdogError::TooLong),
    }
}

/// The state the lock may be taken in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Configured and running: the watchdog is doing its job.
    Running,
    /// Not running: a lock taken here leaves the peripheral locked with no timeout.
    Disabled,
}

/// Whether `LOCK` may be written, and why not if it may not.
///
/// This is H27 as a function. The rule is not "always" and not "never": the lock is meaningful only on a watchdog that is
/// already running with the timeout the product wants, because after the lock nothing can change it.
pub fn may_lock(state: State) -> Result<(), WatchdogError> {
    match state {
        State::Running => Ok(()),
        State::Disabled => Err(WatchdogError::CannotLock(
            "the watchdog is disabled: locking now leaves it locked with no timeout programmed",
        )),
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    /// One second from the 32.768 kHz clock this project's low-frequency path uses.
    #[test]
    fn one_second_is_expressible() {
        let (period, error) = period_for_ms(32_768, 1000).expect("a second");
        assert_eq!(period.period_ms(32_768), 1000, "exactly a second");
        assert_eq!(error, 0, "no error");
        assert!(PRESCALERS.contains(&period.prescaler));
        assert!(period.top <= MAX_TOP);
    }

    /// Every useful period is within a thousandth, at both clocks this project runs.
    #[test]
    fn periods_this_project_uses() {
        for clock in [32_768u32, 19_000_000] {
            for wanted in [10u32, 100, 500, 1000, 2000, 5000] {
                let (period, error) = period_for_ms(clock, wanted).expect("a period");
                assert!(
                    error <= 1,
                    "clock {clock} wanted {wanted} ms: {} ms is {error} per thousand off",
                    period.period_ms(clock)
                );
            }
        }
    }

    /// The fields bound the period: eighteen hours needs the largest prescaler, and beyond the bound is refused.
    #[test]
    fn the_limits_are_the_fields() {
        // 18 hours is 64 800 s; with the 32768 prescaler that is 64 800 timer clocks, so top ~64 799.
        let (period, error) = period_for_ms(32_768, 18 * 3600 * 1000).expect("a long period");
        assert_eq!(period.prescaler, 32_768, "the largest prescaler");
        assert!(
            (64_798..=64_801).contains(&period.top),
            "top {} is not within a couple of counts of 64 800",
            period.top
        );
        assert!(error <= 1, "{error} per thousand is more than a thousandth");

        // The maximum expressible period is prescaler 32768 with top 65535: 65 536 s, 18.2 hours.
        let maximum = Period {
            prescaler: 32_768,
            top: MAX_TOP,
        };
        assert_eq!(maximum.period_ms(32_768), 65_536_000, "18.2 hours in ms");

        // And beyond it, a refusal rather than a truncated top.
        assert!(
            period_for_ms(32_768, 20 * 3600 * 1000).is_err(),
            "20 hours does not fit the fields and must be refused"
        );
    }

    /// Zero inputs are refused rather than divided by.
    #[test]
    fn zero_is_refused() {
        assert_eq!(period_for_ms(0, 1000), Err(WatchdogError::Zero));
        assert_eq!(period_for_ms(32_768, 0), Err(WatchdogError::Zero));
    }

    /// The lock rule, which is H27.
    #[test]
    fn the_lock_needs_a_running_watchdog() {
        assert_eq!(may_lock(State::Running), Ok(()));
        assert!(
            may_lock(State::Disabled).is_err(),
            "locking a disabled watchdog leaves it locked with no timeout"
        );
        assert_eq!(
            may_lock(State::Disabled),
            Err(WatchdogError::CannotLock(
                "the watchdog is disabled: locking now leaves it locked with no timeout programmed"
            ))
        );
    }

    /// The period the hardware uses is the period reported: a watchdog that reports one number and uses another is worse
    /// than no watchdog.
    #[test]
    fn the_reported_period_is_the_one_the_fields_give() {
        for clock in [32_768u32, 19_000_000] {
            for wanted in [10u32, 250, 1000, 3000] {
                let (period, _) = period_for_ms(clock, wanted).expect("a period");
                let clocks = period.prescaler as u64 * (period.top as u64 + 1);
                let expected_ms = ((clocks * 1000) / clock as u64) as u32;
                assert_eq!(
                    period.period_ms(clock),
                    expected_ms,
                    "clock {clock} wanted {wanted}: the arithmetic and the fields disagree"
                );
            }
        }
    }
}
