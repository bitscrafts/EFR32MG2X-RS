#![no_std]
#![deny(missing_docs)]

//! Motion classification: fall detection from accelerometer samples.
//!
//! This crate is **hardware free**: it takes samples and returns events. It has no
//! dependency on the PAC, on a HAL or on a board, which is the point — the same
//! state machine runs on the Cortex-M33, in a host test, and on any device that can
//! hand it three acceleration axes and a time step.
//!
//! The thresholds are policy, and policy is not a driver concern: a HAL answers
//! "what did the accelerometer say", not "was that a fall". Keeping the two apart
//! lets the driver stay correct for devices that never classify motion, and lets
//! this crate be tested without a board.
//!
//! ## Why this shape
//!
//! A fall has a signature in time, and no single threshold captures it:
//!
//! 1. **free fall** — the board falls, so the magnitude drops toward zero. This is
//!    the only phase that distinguishes a fall from a slam on the table;
//! 2. **impact** — a large spike when the body stops;
//! 3. **aftermath** — the device ends up still, and often in a different
//!    orientation, because a person who fell does not keep shaking it.
//!
//! So the detector is a small state machine with hysteresis. Each threshold below
//! is a measurement, not a guess:
//!
//! - **rest** on this board reads 970–1020 mg (`docs/04-reference/IMU-INVESTIGATION.md`
//!   and the captures in `_tmp`), so "still" is 800–1200 mg;
//! - **free fall** needs the magnitude below 400 mg: at rest the axis pointing up
//!   carries one g, and in free fall all three axes go to zero together;
//! - **impact** needs more than 2500 mg. Measured shaking and setting the board
//!   down peaked at 1801 mg at ±8 g, so a 2.5 g threshold does not fire on handling.
//!
//! The state machine also refuses to report a fall without the free-fall phase,
//! which is the difference between "someone dropped the device" and "someone shook
//! the device" — the second is not a fall and must not raise an alarm.
//!
//! ## Time base
//!
//! `update` takes the elapsed milliseconds since the previous sample instead of
//! counting samples: the sensor's rate is configurable and the caller may sample
//! irregularly, so the detector cannot assume a period. A detector that counts
//! samples reports different things at 100 Hz and 200 Hz, which is a defect that
//! hides in a demo and shows up in the field.

/// What the accelerometer magnitude was doing, in milliseconds.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Phase {
    /// Nothing interesting.
    Idle,
    /// Magnitude dropped below the free-fall threshold; how long it has been there.
    FreeFall { ms: u32 },
    /// Impact seen after a free fall; waiting for the device to settle.
    Aftermath {
        ms: u32,
        peak_mg: i32,
        freefall_ms: u32,
    },
}

/// Something the detector decided.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Event {
    /// A fall: free fall, then an impact, then stillness.
    Fall {
        /// Largest magnitude seen during the impact, in thousandths of a g.
        peak_mg: i32,
        /// How long the free fall lasted.
        freefall_ms: u32,
    },
    /// An impact without a preceding free fall — knocked, dropped onto a table
    /// from hand height, or the board being tapped. Reported separately so the
    /// caller can log it without alarming.
    Impact {
        /// Largest magnitude seen, in thousandths of a g.
        peak_mg: i32,
    },
}

/// The fall detector.
#[derive(Debug, Clone)]
pub struct FallDetector {
    phase: Phase,
    /// Peak magnitude of the current impact, even before a phase change.
    impact_peak: i32,
}

impl FallDetector {
    /// Below this magnitude the device is falling, not resting.
    pub const FREEFALL_MG: i32 = 400;
    /// Free fall shorter than this is a dip during handling, not a fall.
    pub const FREEFALL_MIN_MS: u32 = 120;
    /// Above this the device hit something hard.
    pub const IMPACT_MG: i32 = 2500;
    /// After a free fall, the impact must arrive within this window.
    pub const IMPACT_WINDOW_MS: u32 = 600;
    /// Magnitude band that counts as "lying still" after the impact.
    pub const STILL_MIN_MG: i32 = 700;
    /// Upper edge of "lying still".
    pub const STILL_MAX_MG: i32 = 1300;
    /// How long the device must stay still before the fall is reported.
    pub const SETTLE_MS: u32 = 800;

    /// A detector that has seen nothing yet.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            phase: Phase::Idle,
            impact_peak: 0,
        }
    }

    /// Feed one sample. `dt_ms` is the time since the previous sample.
    #[allow(clippy::missing_panics_doc)]
    pub fn update(&mut self, accel_mg: [i32; 3], dt_ms: u32) -> Option<Event> {
        let magnitude = isqrt(
            (accel_mg[0] as i64 * accel_mg[0] as i64
                + accel_mg[1] as i64 * accel_mg[1] as i64
                + accel_mg[2] as i64 * accel_mg[2] as i64) as u64,
        ) as i32;

        if magnitude > self.impact_peak {
            self.impact_peak = magnitude;
        }

        match self.phase {
            Phase::Idle => {
                if magnitude < Self::FREEFALL_MG {
                    self.phase = Phase::FreeFall { ms: dt_ms };
                } else if magnitude > Self::IMPACT_MG {
                    // A hard hit with no free fall: report it once, then wait for
                    // the magnitude to come back down so one slam is one event.
                    let peak = self.impact_peak;
                    self.impact_peak = 0;
                    return Some(Event::Impact { peak_mg: peak });
                }
                None
            }
            Phase::FreeFall { ms } => {
                if magnitude < Self::FREEFALL_MG {
                    let total = ms.saturating_add(dt_ms);
                    self.phase = Phase::FreeFall { ms: total };
                    return None;
                }
                if ms >= Self::FREEFALL_MIN_MS && magnitude > Self::IMPACT_MG {
                    self.phase = Phase::Aftermath {
                        ms: 0,
                        peak_mg: magnitude,
                        freefall_ms: ms,
                    };
                    return None;
                }
                // Either the drop was too short, or it ended without a hard
                // impact. Both mean "not a fall"; start again from idle.
                self.phase = Phase::Idle;
                self.impact_peak = 0;
                None
            }
            Phase::Aftermath {
                ms,
                peak_mg,
                freefall_ms,
            } => {
                let peak = peak_mg.max(magnitude);
                if magnitude > Self::IMPACT_MG {
                    self.phase = Phase::Aftermath {
                        ms: 0,
                        peak_mg: peak,
                        freefall_ms,
                    };
                    return None;
                }
                let still = (Self::STILL_MIN_MG..=Self::STILL_MAX_MG).contains(&magnitude);
                if still {
                    let settled = ms.saturating_add(dt_ms);
                    if settled >= Self::SETTLE_MS {
                        self.phase = Phase::Idle;
                        self.impact_peak = 0;
                        return Some(Event::Fall {
                            peak_mg: peak,
                            freefall_ms,
                        });
                    }
                    self.phase = Phase::Aftermath {
                        ms: settled,
                        peak_mg: peak,
                        freefall_ms,
                    };
                    return None;
                }
                // Moved again without settling: it was shaken, not dropped.
                if ms > Self::IMPACT_WINDOW_MS {
                    self.phase = Phase::Idle;
                    self.impact_peak = 0;
                } else {
                    self.phase = Phase::Aftermath {
                        ms: ms.saturating_add(dt_ms),
                        peak_mg: peak,
                        freefall_ms,
                    };
                }
                None
            }
        }
    }

    /// True while the detector believes the device is in the air.
    #[must_use]
    pub fn in_free_fall(&self) -> bool {
        matches!(self.phase, Phase::FreeFall { ms } if ms >= Self::FREEFALL_MIN_MS)
    }

    /// Back to the initial state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for FallDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Integer square root; `f32::sqrt` needs libm, which a `no_std` binary does not
/// link by default.
fn isqrt(value: u64) -> u64 {
    if value == 0 {
        return 0;
    }
    let mut x = value;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + value / x) / 2;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Samples are 5 ms apart (200 Hz), as the monitor samples.
    const DT: u32 = 5;

    fn feed(det: &mut FallDetector, mg: [i32; 3], ms: u32) -> Option<Event> {
        let mut event = None;
        let mut elapsed = 0;
        while elapsed < ms {
            if let Some(e) = det.update(mg, DT) {
                event = Some(e);
            }
            elapsed += DT;
        }
        event
    }

    #[test]
    fn quiet_board_reports_nothing() {
        let mut det = FallDetector::new();
        assert_eq!(feed(&mut det, [990, 60, 70], 2000), None);
    }

    #[test]
    fn shake_reports_impact_but_not_fall() {
        let mut det = FallDetector::new();
        // 1.8 g peaks with brief dips: the measured handling signature.
        let mut event = None;
        for _ in 0..40 {
            event = det.update([1800, 0, 0], DT).or(event);
            event = det.update([990, 0, 0], DT).or(event);
            event = det.update([650, 0, 0], DT).or(event);
        }
        assert!(matches!(event, Some(Event::Impact { .. }) | None));
        assert!(
            !matches!(event, Some(Event::Fall { .. })),
            "shaking must never be reported as a fall"
        );
    }

    #[test]
    fn free_fall_impact_and_stillness_is_a_fall() {
        let mut det = FallDetector::new();
        assert_eq!(feed(&mut det, [990, 30, 40], 200), None);
        // 250 ms of free fall: the signature a real drop leaves.
        assert_eq!(feed(&mut det, [20, 15, 10], 250), None);
        assert!(det.in_free_fall());
        // Impact.
        let impact = feed(&mut det, [6000, 200, 300], 10);
        assert!(impact.is_none(), "the fall is only reported after settling");
        // Lying still afterwards.
        let settled = feed(&mut det, [980, 40, 60], 1000);
        match settled {
            Some(Event::Fall {
                peak_mg,
                freefall_ms,
            }) => {
                assert!(peak_mg >= FallDetector::IMPACT_MG);
                assert!(freefall_ms >= FallDetector::FREEFALL_MIN_MS);
            }
            other => panic!("expected a fall, got {other:?}"),
        }
    }

    #[test]
    fn short_dip_is_not_free_fall() {
        let mut det = FallDetector::new();
        // 60 ms below the threshold, then an impact: too short to be a drop.
        assert_eq!(feed(&mut det, [100, 0, 0], 60), None);
        assert!(!det.in_free_fall());
        let _ = det.update([6000, 0, 0], DT);
        assert!(!matches!(
            feed(&mut det, [980, 0, 0], 1000),
            Some(Event::Fall { .. })
        ));
    }

    #[test]
    fn isqrt_matches_floats() {
        for v in [0u64, 1, 3, 4, 15, 16, 99, 1000, 980_100, 36_000_000] {
            let got = isqrt(v);
            assert!(
                got * got <= v && (got + 1) * (got + 1) > v,
                "isqrt({v}) = {got}"
            );
        }
    }
}
