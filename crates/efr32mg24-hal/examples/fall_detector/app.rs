//! The application: sampling, classification and bookkeeping, with no hardware of
//! its own.
//!
//! It owns everything that is *policy*: how often to sample, what to do with an
//! event, what the panel says, and the self tests that run before the live loop.
//! The hardware arrives as a trait object of sorts — an `Imu` and a display — so the
//! application can be reasoned about without the board.

use motion_detection::{Event, FallDetector};

use crate::falls::{DROPS, DROPS_DT_MS, DROPS_EXPECTED_FALLS};
use crate::falls_400hz::{DROPS_400HZ, DROPS_400HZ_DT_US, DROPS_400HZ_EXPECTED};
use crate::recorded::{RECORDED, RECORDED_DT_MS};

/// What the interface needs to know. Plain data, no drivers.
#[derive(Debug, Clone, Copy, Default)]
pub struct Status {
    /// Falls detected since boot.
    pub falls: u32,
    /// Impacts without a preceding free fall (knocked, tapped, set down hard).
    pub impacts: u32,
    /// Largest impact of any reported fall, in mg.
    pub peak_fall_mg: i32,
    /// Live magnitude, in mg.
    pub magnitude_mg: i32,
    /// True while the device is believed to be in the air.
    pub in_free_fall: bool,
    /// How long the last free fall lasted, in milliseconds.
    pub freefall_ms: u32,
    /// The most recent event, if any.
    pub last_event: Option<Event>,
}

/// The application state machine — a thin layer over the classifier that adds the
/// counters and the self tests.
pub struct App {
    detector: FallDetector,
    status: Status,
}

/// The outcome of the boot-time self tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelfTest {
    /// True when the recorded handling data raised no fall.
    pub real_data_clean: bool,
    /// True when the synthetic drop was detected.
    pub synthetic_detected: bool,
    /// Falls found in the first recorded round (two drops at 200 Hz).
    pub real_drops_detected: u32,
    /// Falls found in the second recorded round (four drops at 400 Hz).
    pub drops_400hz_detected: u32,
}

impl SelfTest {
    /// Every test passed.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.real_data_clean
            && self.synthetic_detected
            && self.real_drops_detected == DROPS_EXPECTED_FALLS
    }
}

impl App {
    /// A fresh application.
    #[must_use]
    pub fn new() -> Self {
        Self {
            detector: FallDetector::new(),
            status: Status::default(),
        }
    }

    /// Current status, for the interface.
    #[must_use]
    pub fn status(&self) -> Status {
        self.status
    }

    /// Feed one sample. `dt_us` is the time since the previous one.
    ///
    /// Microseconds because the sampler runs at 400 Hz: 2.5 ms per sample rounded to
    /// whole milliseconds is a 20 % error in every timer in the detector.
    /// Returns the event, if this sample produced one, so the caller can log it.
    pub fn sample_us(&mut self, accel_mg: [i32; 3], dt_us: u32) -> Option<Event> {
        self.status.magnitude_mg = magnitude(accel_mg);
        let event = self.detector.update_us(accel_mg, dt_us);
        match event {
            Some(Event::Fall {
                peak_mg,
                freefall_ms,
            }) => {
                self.status.falls += 1;
                self.status.last_event = Some(Event::Fall {
                    peak_mg,
                    freefall_ms,
                });
                if peak_mg > self.status.peak_fall_mg {
                    self.status.peak_fall_mg = peak_mg;
                }
            }
            Some(event @ Event::Impact { .. }) => {
                self.status.impacts += 1;
                self.status.last_event = Some(event);
            }
            Some(event @ Event::FreeFall { .. }) => {
                self.status.freefall_ms = match event {
                    Event::FreeFall { ms } => ms,
                    _ => 0,
                };
                self.status.last_event = Some(event);
            }
            None => {}
        }
        self.status.in_free_fall = self.detector.in_free_fall();
        event
    }

    /// Replay the recorded handling data through a fresh detector.
    ///
    /// This is the test that matters: real samples from this board, including the
    /// 1601 mg peaks that handling produces, must not be reported as a fall.
    #[must_use]
    pub fn self_test_real(&self) -> bool {
        let mut detector = FallDetector::new();
        for sample in RECORDED.iter() {
            let accel = [sample[0] as i32, sample[1] as i32, sample[2] as i32];
            if let Some(Event::Fall { .. }) = detector.update(accel, RECORDED_DT_MS) {
                return false;
            }
        }
        true
    }

    /// Replay a drop: free fall, impact, then stillness.
    ///
    /// No amount of shaking on a desk reproduces this, so it is constructed — but
    /// from the physics the classifier is meant to recognise: 250 ms near zero g
    /// (a fall of about 30 cm), a 6 g impact, then lying still in a new
    /// orientation.
    #[must_use]
    pub fn self_test_synthetic(&self) -> bool {
        let mut detector = FallDetector::new();
        let dt = 5;

        for _ in 0..400 {
            let _ = detector.update([990, 40, 60], dt); // 2 s at rest
        }
        for _ in 0..50 {
            let _ = detector.update([15, 12, 8], dt); // 250 ms falling
        }
        for _ in 0..4 {
            let _ = detector.update([6000, 200, 300], dt); // impact
        }
        let mut fall = None;
        for _ in 0..400 {
            if let Some(e) = detector.update([60, 985, 80], dt) {
                fall = Some(e);
            }
        }
        matches!(fall, Some(Event::Fall { .. }))
    }

    /// Replay the two recorded drops and count what the detector reports.
    ///
    /// This is the test that decides whether the classifier is real: data from this
    /// board, falls that happened, and a count to compare with what was performed.
    #[must_use]
    pub fn self_test_drops(&self) -> u32 {
        let mut detector = FallDetector::new();
        let mut falls = 0;
        for sample in DROPS.iter() {
            let accel = [sample[0] as i32, sample[1] as i32, sample[2] as i32];
            if let Some(Event::Fall { .. }) = detector.update(accel, DROPS_DT_MS) {
                falls += 1;
            }
        }
        falls
    }

    /// Replay the four drops recorded at 400 Hz.
    #[must_use]
    pub fn self_test_drops_400hz(&self) -> u32 {
        let mut detector = FallDetector::new();
        let mut falls = 0;
        for sample in DROPS_400HZ.iter() {
            let accel = [sample[0] as i32, sample[1] as i32, sample[2] as i32];
            if let Some(Event::Fall { .. }) = detector.update_us(accel, DROPS_400HZ_DT_US) {
                falls += 1;
            }
        }
        falls
    }

    /// Run every self test.
    #[must_use]
    pub fn self_test(&self) -> SelfTest {
        SelfTest {
            real_data_clean: self.self_test_real(),
            synthetic_detected: self.self_test_synthetic(),
            real_drops_detected: self.self_test_drops(),
            drops_400hz_detected: self.self_test_drops_400hz(),
        }
    }
}

impl Default for App {
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

/// Integer square root: `no_std` without libm.
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

/// The audible alarm, advanced by the sampling loop instead of by a blocking delay.
///
/// A blocking `beep()` would stop classification for the whole pattern — three beeps
/// of 150 ms is 750 ms of 200 Hz samples, and a fall comes in bursts: the second
/// impact lands while the first alarm is still sounding. So the pattern is a state
/// machine ticked once per sample, and the sampling never stops.
#[derive(Debug, Clone, Copy)]
pub struct Alarm {
    beeps_left: u8,
    /// True while the tone should be sounding.
    sounding: bool,
    /// Time left in the current phase, in microseconds.
    phase_ms: u32,
}

impl Alarm {
    /// How long each beep lasts.
    pub const BEEP_MS: u32 = 150;
    /// Silence between beeps.
    pub const GAP_MS: u32 = 120;
    /// Beeps in one alarm.
    pub const BEEPS: u8 = 3;

    /// A silent alarm.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            beeps_left: 0,
            sounding: false,
            phase_ms: 0,
        }
    }

    /// Start the pattern. Called on a detected fall.
    pub fn trigger(&mut self) {
        self.beeps_left = Self::BEEPS;
        self.sounding = true;
        self.phase_ms = Self::BEEP_MS * 1000;
    }

    /// Advance by `dt_ms` and say whether the tone should be on now.
    ///
    /// Idempotent with respect to the buzzer: the caller can set the output every
    /// sample without cost, because the driver writes a register and the timer keeps
    /// the tone going.
    pub fn tick(&mut self, dt_us: u32) -> bool {
        if self.beeps_left == 0 {
            return false;
        }
        if dt_us >= self.phase_ms * 1000 {
            if self.sounding {
                // A beep just ended: short silence, then the next beep.
                self.sounding = false;
                self.phase_ms = Self::GAP_MS * 1000;
            } else {
                self.beeps_left -= 1;
                if self.beeps_left == 0 {
                    self.sounding = false;
                    self.phase_ms = 0;
                    return false;
                }
                self.sounding = true;
                self.phase_ms = Self::BEEP_MS * 1000;
            }
            // Whatever is left of this tick counts against the new phase.
            self.phase_ms = self.phase_ms.saturating_sub(dt_us);
        } else {
            self.phase_ms -= dt_us;
        }
        self.sounding
    }

    /// True while a pattern is in progress.
    #[must_use]
    pub fn active(&self) -> bool {
        self.beeps_left > 0
    }
}

impl Default for Alarm {
    fn default() -> Self {
        Self::new()
    }
}
