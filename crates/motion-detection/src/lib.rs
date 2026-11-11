//! Motion classification: fall detection from accelerometer samples.
//!
//! This crate is **hardware free**: it takes samples and returns events. It has no
//! dependency on the PAC, on a HAL or on a board, which is the point — the same state
//! machine runs on the Cortex-M33, in a host test, and on any device that can hand it
//! three acceleration axes and a time step.
//!
//! The thresholds are policy, and policy is not a driver concern: a HAL answers "what
//! did the accelerometer say", not "was that a fall".
//!
//! ## The shape of a real fall, as measured
//!
//! Two drops recorded from this board (200 Hz, ±8 g, `docs/03-development/
//! SESSION-2026-11-11.md`) give the numbers the constants below come from:
//!
//! | event | free fall | impact peak |
//! |---|---|---|
//! | drop 1 | 70 ms, minimum 32 mg | 2268 mg |
//! | drop 2 | 190 ms, minimum 45 mg | 3885 mg |
//! | handling (shake, set down) | never below 400 mg | 1801 mg |
//!
//! The first version of this state machine required the impact **immediately** after
//! the free fall and detected zero of those two drops: in both recordings the board
//! reads about one g for a few hundred milliseconds between the fall and the impact
//! (it lands, bounces, or is caught before hitting). The corrected model opens a
//! *window* on a qualifying free fall and accepts the impact anywhere inside it, then
//! waits for calm.
//!
//! With that change both drops are detected and the handling recording produces no
//! fall, which is what the tests encode.

#![no_std]

/// Below this magnitude the device is falling, not resting.
///
/// Measured: drops reach 32–45 mg; a board at rest never goes below 400 mg even while
/// being shaken and set down.
pub const FREEFALL_MG: i32 = 400;
/// Free fall shorter than this is a dip during handling, not a fall.
///
/// Measured: real drops gave 42, 70, 80, 100, 138, 140, 145 and 190 ms; the shortest
/// was 42 ms at 400 Hz. Handling produced dips of 10–25 ms and never went below 630 mg.
pub const FREEFALL_MIN_MS: u32 = 40;
/// A knock: a hard hit with **no** free fall before it.
///
/// Measured: the hardest handling peak was 1801 mg, so 2000 mg separates a bang from
/// picking the device up.
pub const IMPACT_MG: i32 = 2000;

/// After a free fall, this is enough to call it a landing.
///
/// Much lower than [`IMPACT_MG`] because the free fall has already established that
/// the device was in the air, and because a drop onto a desk is *softer* than the
/// assumption suggests: nine real drops landed at 1148–12823 mg, the gentlest at
/// 1148. A board caught in the hand returns to ~990 mg and is excluded by this
/// threshold; a board that lands is not.
pub const LANDING_MG: i32 = 1100;
/// How long after a free fall the impact may still arrive.
pub const IMPACT_WINDOW_MS: u32 = 600;
/// After the impact the device must stay *below* this to count as settled.
///
/// Measured, and the reason this is not a band: a dropped board bounces. With a
/// 700..1300 mg band every bounce reset the timer and two of five real drops were
/// never reported; requiring only "quiet" — anything under 1.5 g — detects them and
/// still produces no false fall on the handling recording.
pub const QUIET_MG: i32 = 1500;
/// How long the device must stay quiet before the fall is reported.
///
/// 600 ms rather than 800: with a 800 ms requirement one of four drops was lost because
/// the board was still rocking, and 600 ms detects all nine recorded drops with no false
/// alarm on the handling recording.
pub const SETTLE_MS: u32 = 600;
/// A candidate with no impact for this long was not a fall.
pub const GIVE_UP_MS: u32 = 3000;

/// What the accelerometer magnitude was doing.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Phase {
    /// Nothing interesting.
    Idle,
    /// Magnitude below [`FREEFALL_MG`]; how long it has been there, in microseconds.
    FreeFall { us: u32 },
    /// A qualifying free fall happened; watching for the impact and then for calm.
    Candidate {
        /// Peak magnitude since the free fall.
        peak_mg: i32,
        /// Free fall that opened this candidate, in microseconds.
        freefall_us: u32,
        /// Time since the candidate opened, in microseconds.
        elapsed_us: u32,
        /// Continuous quiet time since the peak, in microseconds.
        settled_us: u32,
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
    /// An impact without a preceding free fall — knocked, tapped, or set down hard.
    /// Reported separately so a caller can log it without alarming.
    Impact {
        /// Largest magnitude seen, in thousandths of a g.
        peak_mg: i32,
    },
}

/// The fall detector.
#[derive(Debug, Clone)]
pub struct FallDetector {
    phase: Phase,
}

impl FallDetector {
    /// A detector that has seen nothing yet.
    #[must_use]
    pub const fn new() -> Self {
        Self { phase: Phase::Idle }
    }

    /// Feed one sample. `dt_ms` is the time since the previous sample.
    ///
    /// Whole milliseconds are not enough at 400 Hz (2.5 ms per sample), so
    /// [`Self::update_us`] takes the interval exactly; this is the convenience wrapper.
    pub fn update(&mut self, accel_mg: [i32; 3], dt_ms: u32) -> Option<Event> {
        self.update_us(accel_mg, dt_ms.saturating_mul(1000))
    }

    /// Feed one sample with a microsecond interval.
    ///
    /// The state machine keeps its timers in microseconds for this reason: at 400 Hz a
    /// whole-millisecond clock would round every sample by 20 %, which is the difference
    /// between an 800 ms settle and a 640 ms one.
    pub fn update_us(&mut self, accel_mg: [i32; 3], dt_us: u32) -> Option<Event> {
        let magnitude = magnitude(accel_mg);
        const FREEFALL_MIN_US: u32 = FREEFALL_MIN_MS * 1000;
        const WINDOW_US: u32 = IMPACT_WINDOW_MS * 1000;
        const SETTLE_US: u32 = SETTLE_MS * 1000;
        const GIVE_UP_US: u32 = GIVE_UP_MS * 1000;

        match self.phase {
            Phase::Idle => {
                if magnitude < FREEFALL_MG {
                    self.phase = Phase::FreeFall { us: dt_us };
                    None
                } else if magnitude > IMPACT_MG {
                    Some(Event::Impact { peak_mg: magnitude })
                } else {
                    None
                }
            }

            Phase::FreeFall { us } => {
                if magnitude < FREEFALL_MG {
                    self.phase = Phase::FreeFall {
                        us: us.saturating_add(dt_us),
                    };
                    return None;
                }
                if us >= FREEFALL_MIN_US {
                    // A qualifying free fall: open the window for the impact.
                    if magnitude > IMPACT_MG {
                        self.phase = Phase::Candidate {
                            peak_mg: magnitude,
                            freefall_us: us,
                            elapsed_us: dt_us,
                            settled_us: 0,
                        };
                        None
                    } else {
                        self.phase = Phase::Candidate {
                            peak_mg: 0,
                            freefall_us: us,
                            elapsed_us: 0,
                            settled_us: 0,
                        };
                        None
                    }
                } else {
                    self.phase = Phase::Idle;
                    if magnitude > IMPACT_MG {
                        Some(Event::Impact { peak_mg: magnitude })
                    } else {
                        None
                    }
                }
            }

            Phase::Candidate {
                peak_mg,
                freefall_us,
                elapsed_us,
                settled_us,
            } => {
                let elapsed = elapsed_us.saturating_add(dt_us);
                let peak = peak_mg.max(magnitude);

                if peak >= LANDING_MG {
                    // The impact has happened: from here only quiet matters. There is
                    // deliberately no deadline for the settle — a board that lands,
                    // bounces and rolls takes a second, and giving up mid-settle would
                    // call a real fall a knock (measured: two of five drops).
                    let settled = if magnitude < QUIET_MG {
                        settled_us.saturating_add(dt_us)
                    } else {
                        0
                    };
                    if settled >= SETTLE_US {
                        self.phase = Phase::Idle;
                        Some(Event::Fall {
                            peak_mg: peak,
                            freefall_ms: freefall_us / 1000,
                        })
                    } else {
                        self.phase = Phase::Candidate {
                            peak_mg: peak,
                            freefall_us,
                            elapsed_us: elapsed,
                            settled_us: settled,
                        };
                        None
                    }
                } else if elapsed > WINDOW_US {
                    // No impact within the window: the device fell and was caught.
                    self.phase = Phase::Idle;
                    None
                } else {
                    self.phase = Phase::Candidate {
                        peak_mg: peak,
                        freefall_us,
                        elapsed_us: elapsed,
                        settled_us,
                    };
                    None
                }
            }
        }
    }

    /// True while the detector believes the device is in the air.
    #[must_use]
    pub fn in_free_fall(&self) -> bool {
        matches!(self.phase, Phase::FreeFall { us } if us >= FREEFALL_MIN_MS * 1000)
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

/// Magnitude of a three-axis vector, in the same units as the input.
fn magnitude(accel_mg: [i32; 3]) -> i32 {
    let squared = accel_mg[0] as i64 * accel_mg[0] as i64
        + accel_mg[1] as i64 * accel_mg[1] as i64
        + accel_mg[2] as i64 * accel_mg[2] as i64;
    isqrt(squared as u64) as i32
}

/// Integer square root; `f32::sqrt` needs libm, which a `no_std` binary does not link
/// by default.
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

    /// 5 ms per sample, as the board samples at 200 Hz.
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
    fn handling_is_not_a_fall() {
        // The measured handling signature: 1.8 g peaks with brief dips. Never a fall.
        let mut det = FallDetector::new();
        let mut event = None;
        for _ in 0..40 {
            event = det.update([1800, 0, 0], DT).or(event);
            event = det.update([990, 0, 0], DT).or(event);
            event = det.update([650, 0, 0], DT).or(event);
        }
        assert!(!matches!(event, Some(Event::Fall { .. })));
    }

    #[test]
    fn real_drop_with_impact_after_a_one_g_interlude_is_detected() {
        // Drop 1, replayed at the values the board recorded: 70 ms of free fall,
        // then about one g, then a 2268 mg impact, then calm.
        let mut det = FallDetector::new();
        assert_eq!(feed(&mut det, [990, 40, 60], 200), None);
        feed(&mut det, [20, 18, 15], 70);
        assert!(det.in_free_fall());
        feed(&mut det, [980, 45, 60], 120); // lands / bounces before the peak
        feed(&mut det, [1500, 1300, 1100], 25); // 2268 mg
        match feed(&mut det, [975, 40, 55], 900) {
            Some(Event::Fall {
                peak_mg,
                freefall_ms,
            }) => {
                assert!(peak_mg >= IMPACT_MG, "peak {peak_mg}");
                assert!(freefall_ms >= FREEFALL_MIN_MS, "freefall {freefall_ms}");
            }
            other => panic!("expected a fall, got {other:?}"),
        }
    }

    #[test]
    fn real_drop_with_a_long_interlude_is_detected() {
        // Drop 2: 190 ms of free fall, 400 ms at one g, then 3885 mg, then a board
        // that takes a second to stop moving.
        let mut det = FallDetector::new();
        feed(&mut det, [990, 40, 60], 200);
        feed(&mut det, [30, 25, 20], 190);
        feed(&mut det, [1010, 40, 60], 400);
        feed(&mut det, [2500, 2200, 1900], 30); // 3885 mg
        feed(&mut det, [900, 600, 700], 300); // still rolling about
        match feed(&mut det, [960, 40, 60], 900) {
            Some(Event::Fall { peak_mg, .. }) => assert!(peak_mg >= 3800),
            other => panic!("expected a fall, got {other:?}"),
        }
    }

    #[test]
    fn a_soft_landing_after_a_short_free_fall_is_detected() {
        // Drop 3 of the 400 Hz round: 42 ms of free fall, a landing of only 1493 mg
        // (a board laid down flat lands softly), then quiet. This is the case the
        // 800 ms settle rule lost and the 600 ms rule keeps.
        let mut det = FallDetector::new();
        feed(&mut det, [990, 40, 60], 300);
        feed(&mut det, [30, 25, 20], 42);
        feed(&mut det, [1200, 700, 400], 20); // 1493 mg
        match feed(&mut det, [980, 40, 60], 700) {
            Some(Event::Fall { peak_mg, .. }) => {
                assert!((1200..1700).contains(&peak_mg), "peak {peak_mg}");
            }
            other => panic!("expected a fall, got {other:?}"),
        }
    }

    #[test]
    fn a_dip_too_short_is_not_a_fall() {
        // 25 ms below the threshold, then a hard impact: a knock, not a drop.
        let mut det = FallDetector::new();
        assert_eq!(feed(&mut det, [100, 0, 0], 25), None);
        assert!(!det.in_free_fall());
        let event = feed(&mut det, [6000, 0, 0], DT);
        assert!(matches!(event, Some(Event::Impact { .. })), "{event:?}");
        assert!(!matches!(
            feed(&mut det, [980, 0, 0], 1000),
            Some(Event::Fall { .. })
        ));
    }

    #[test]
    fn impact_without_free_fall_is_reported_as_impact() {
        let mut det = FallDetector::new();
        let event = feed(&mut det, [6000, 300, 200], DT);
        match event {
            Some(Event::Impact { peak_mg }) => assert!(peak_mg > IMPACT_MG),
            other => panic!("expected an impact, got {other:?}"),
        }
    }

    #[test]
    fn a_fall_caught_before_landing_is_not_reported() {
        // Free fall, then back to one g with no impact: someone caught it.
        let mut det = FallDetector::new();
        feed(&mut det, [990, 40, 60], 200);
        feed(&mut det, [25, 20, 18], 150);
        assert_eq!(feed(&mut det, [990, 45, 60], 1000), None);
    }

    #[test]
    fn isqrt_is_exact() {
        for v in [0u64, 1, 3, 4, 15, 16, 99, 1000, 980_100, 36_000_000] {
            let got = isqrt(v);
            assert!(
                got * got <= v && (got + 1) * (got + 1) > v,
                "isqrt({v}) = {got}"
            );
        }
    }
}
