//! The application: sampling, classification and bookkeeping, with no hardware of
//! its own.
//!
//! It owns everything that is *policy*: how often to sample, what to do with an
//! event, what the panel says, and the self tests that run before the live loop.
//! The hardware arrives as a trait object of sorts — an `Imu` and a display — so the
//! application can be reasoned about without the board.

use motion_detection::{Event, FallDetector};

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
}

impl SelfTest {
    /// Both tests passed.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.real_data_clean && self.synthetic_detected
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

    /// Feed one sample. `dt_ms` is the time since the previous one.
    pub fn sample(&mut self, accel_mg: [i32; 3], dt_ms: u32) {
        self.status.magnitude_mg = magnitude(accel_mg);
        match self.detector.update(accel_mg, dt_ms) {
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
            None => {}
        }
        self.status.in_free_fall = self.detector.in_free_fall();
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

    /// Run both self tests.
    #[must_use]
    pub fn self_test(&self) -> SelfTest {
        SelfTest {
            real_data_clean: self.self_test_real(),
            synthetic_detected: self.self_test_synthetic(),
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
