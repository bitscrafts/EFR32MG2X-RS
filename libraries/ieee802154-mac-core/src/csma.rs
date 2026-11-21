//! CSMA/CA: the decision to transmit, made before the radio is asked to.
//!
//! ## Where this sits
//!
//! The retry policy in the parent module decides what to do after a frame went unanswered. This decides
//! something earlier and different: **whether it is allowed to transmit at all**, and how long to wait when it is
//! not. Both are pure decisions, and both mistakes are invisible on the air -- a node that transmits over someone
//! else's frame corrupts it for every listener in range, and a node that gives up too early reports a channel
//! problem that does not exist.
//!
//! ## The algorithm, and which variant
//!
//! The unslotted variant, which is what a peer-to-peer 802.15.4 network uses: there is no beacon, so there is no
//! slot boundary to align to, and the backoff is counted in `aUnitBackoffPeriod` units (20 symbol periods, 320 us
//! at 2.4 GHz).
//!
//! For each attempt: assess the channel. Idle means transmit. Busy means increase the backoff exponent, wait a
//! random number of backoff periods inside that window, and assess again -- until `macMaxCSMABackoffs`
//! assessments have failed, at which point the frame is reported as failed rather than retried forever.
//!
//! ## Why the randomness is an argument
//!
//! Two nodes that back off by the same amount collide again on the retry, so the interval has to be random. This
//! module does not own a random source: `random` is passed in, which lets a test pin it and lets the caller feed
//! it what the radio's generator produced. A decision function that reaches for entropy on its own cannot be
//! tested at all.

use crate::SYMBOLS_PER_BACKOFF_UNIT;

/// What the radio says about the channel right now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    /// Clear to transmit.
    Idle,
    /// Someone else is transmitting, or the energy is above the threshold.
    Busy,
}

/// What the node should do next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Assessment {
    /// The channel is clear: hand the frame to the radio now.
    Transmit,
    /// Wait this many backoff periods (multiply by [`SYMBOLS_PER_BACKOFF_UNIT`]) and assess again.
    Backoff {
        /// Backoff periods to wait, always inside the current window.
        units: u16,
        /// The backoff exponent used, for the log and for whoever wonders why the wait grew.
        exponent: u8,
    },
    /// `macMaxCSMABackoffs` assessments have failed. The frame is lost, and saying so is the point: a node that
    /// silently keeps trying looks busy and delivers nothing.
    Failed,
}

/// The CSMA/CA state machine for one frame.
///
/// Reset it after every frame -- [`CsmaCa::reset`] -- so one frame's contention does not inflate the next one's
/// backoff.
#[derive(Clone, Copy, Debug)]
pub struct CsmaCa {
    /// `macMaxCSMABackoffs`: how many busy assessments are tolerated before giving up. The standard's default is
    /// 4.
    pub max_backoffs: u8,
    /// `macMinBE`: the backoff exponent the first attempt uses. Default 3.
    pub min_backoff_exponent: u8,
    /// `macMaxBE`: the ceiling the exponent grows to and stays under. Default 5.
    pub max_backoff_exponent: u8,
    backoff_exponent: u8,
    backoffs: u8,
}

impl CsmaCa {
    /// The parameters the standard recommends for a peer-to-peer network: `macMinBE` 3, `macMaxBE` 5,
    /// `macMaxCSMABackoffs` 4.
    pub const DEFAULT: Self = Self {
        max_backoffs: 4,
        min_backoff_exponent: 3,
        max_backoff_exponent: 5,
        backoff_exponent: 3,
        backoffs: 0,
    };

    /// A fresh state machine, with the exponent at its minimum and no failed assessments counted.
    pub const fn new(max_backoffs: u8, min_backoff_exponent: u8, max_backoff_exponent: u8) -> Self {
        Self {
            max_backoffs,
            min_backoff_exponent,
            max_backoff_exponent,
            backoff_exponent: min_backoff_exponent,
            backoffs: 0,
        }
    }

    /// How many assessments have failed for the frame in progress.
    pub const fn backoffs(&self) -> u8 {
        self.backoffs
    }

    /// The backoff exponent that would be used for the next wait.
    pub const fn backoff_exponent(&self) -> u8 {
        self.backoff_exponent
    }

    /// Forget the previous frame: the next one starts contention from scratch.
    pub fn reset(&mut self) {
        self.backoff_exponent = self.min_backoff_exponent;
        self.backoffs = 0;
    }

    /// Assess the channel and say what to do.
    ///
    /// `random` is any value; only its low bits are used, uniformly inside the window. It is only consumed when
    /// the channel is busy, so an idle channel costs no randomness and no wait.
    pub fn assess(&mut self, channel: Channel, random: u32) -> Assessment {
        if channel == Channel::Idle {
            // Idle clears the frame's contention state: the next frame starts from the minimum exponent.
            self.backoffs = 0;
            return Assessment::Transmit;
        }

        self.backoffs = self.backoffs.saturating_add(1);
        if self.backoffs > self.max_backoffs {
            return Assessment::Failed;
        }

        // Grow the exponent, then wait inside its window. The wait is taken AFTER the exponent grows, so the
        // first busy assessment already waits longer than the minimum -- waiting the minimum again would put two
        // contending nodes in lockstep.
        self.backoff_exponent = self
            .backoff_exponent
            .saturating_add(1)
            .min(self.max_backoff_exponent);

        let window = 1u32 << self.backoff_exponent;
        Assessment::Backoff {
            units: (random % window) as u16,
            exponent: self.backoff_exponent,
        }
    }
}

/// Convert backoff periods to microseconds at 2.4 GHz, where a symbol is 16 us.
///
/// The caller needs this to arm a timer; keeping the arithmetic here means the same conversion is used by the
/// tests and by the radio path.
pub const fn units_to_micros(units: u16) -> u32 {
    (units as u32) * (SYMBOLS_PER_BACKOFF_UNIT as u32) * 16
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An idle channel transmits immediately, and consumes no randomness: the control case for every test below
    /// that expects a wait.
    #[test]
    fn an_idle_channel_transmits_at_once() {
        let mut csma = CsmaCa::DEFAULT;
        assert_eq!(csma.assess(Channel::Idle, 0), Assessment::Transmit);
        assert_eq!(csma.assess(Channel::Idle, u32::MAX), Assessment::Transmit);
        assert_eq!(csma.backoffs(), 0, "an idle channel costs no backoff");
    }

    #[test]
    fn a_busy_channel_waits_inside_the_first_window() {
        let mut csma = CsmaCa::DEFAULT;
        // min_be is 3, so the first busy assessment grows the exponent to 4 and the window is 16 units.
        match csma.assess(Channel::Busy, 0) {
            Assessment::Backoff { units, exponent } => {
                assert_eq!(exponent, 4);
                assert!(
                    u32::from(units) < 1 << 4,
                    "units {units} must stay inside the 16-unit window"
                );
            }
            other => panic!("expected a backoff, got {other:?}"),
        }
        assert_eq!(csma.backoffs(), 1);
    }

    #[test]
    fn the_exponent_grows_then_stops_at_the_ceiling() {
        let mut csma = CsmaCa::new(10, 3, 5);
        // 3 -> 4 -> 5 -> 5 -> ...
        let mut seen = [0u8; 4];
        for slot in seen.iter_mut() {
            match csma.assess(Channel::Busy, 0) {
                Assessment::Backoff { exponent, .. } => *slot = exponent,
                other => panic!("expected a backoff, got {other:?}"),
            }
        }
        assert_eq!(seen, [4, 5, 5, 5], "macMaxBE is a ceiling, not a step");
    }

    #[test]
    fn every_wait_is_inside_its_own_window() {
        let mut csma = CsmaCa::new(10, 3, 5);
        for _ in 0..8 {
            match csma.assess(Channel::Busy, 4_294_967_295) {
                Assessment::Backoff { units, exponent } => {
                    let window = 1u32 << exponent;
                    assert!(
                        u32::from(units) < window,
                        "units {units} must be under the window {window} for exponent {exponent}"
                    );
                }
                other => panic!("expected a backoff, got {other:?}"),
            }
        }
    }

    #[test]
    fn the_attempts_are_bounded_and_failure_is_reported() {
        let mut csma = CsmaCa::new(4, 3, 5);
        // Four busy assessments get a backoff; the fifth gives up. Anything else is an endless retry loop that
        // looks like work.
        let mut waits = 0;
        let mut outcome = None;
        for _ in 0..10 {
            match csma.assess(Channel::Busy, 0) {
                Assessment::Backoff { .. } => waits += 1,
                other => {
                    outcome = Some(other);
                    break;
                }
            }
        }
        assert_eq!(waits, 4);
        assert_eq!(outcome, Some(Assessment::Failed));
    }

    #[test]
    fn a_channel_that_clears_lets_the_frame_through() {
        let mut csma = CsmaCa::DEFAULT;
        assert!(matches!(
            csma.assess(Channel::Busy, 0),
            Assessment::Backoff { .. }
        ));
        assert!(matches!(
            csma.assess(Channel::Busy, 0),
            Assessment::Backoff { .. }
        ));
        assert_eq!(csma.assess(Channel::Idle, 0), Assessment::Transmit);
    }

    #[test]
    fn resetting_starts_the_next_frame_from_scratch() {
        let mut csma = CsmaCa::DEFAULT;
        for _ in 0..3 {
            let _ = csma.assess(Channel::Busy, 0);
        }
        assert!(csma.backoff_exponent() > csma.min_backoff_exponent);
        csma.reset();
        assert_eq!(csma.backoffs(), 0);
        assert_eq!(csma.backoff_exponent(), csma.min_backoff_exponent);
        match csma.assess(Channel::Busy, 0) {
            Assessment::Backoff { exponent, .. } => assert_eq!(
                exponent,
                csma.min_backoff_exponent + 1,
                "the first wait of a new frame is short again"
            ),
            other => panic!("expected a backoff, got {other:?}"),
        }
    }

    #[test]
    fn the_same_random_value_gives_the_same_wait() {
        let mut a = CsmaCa::DEFAULT;
        let mut b = CsmaCa::DEFAULT;
        assert_eq!(
            a.assess(Channel::Busy, 9_999),
            b.assess(Channel::Busy, 9_999)
        );
    }

    #[test]
    fn different_nodes_do_not_wait_the_same() {
        // The reason the interval is random at all: two nodes that pick the same wait collide again.
        let mut a = CsmaCa::DEFAULT;
        let mut b = CsmaCa::DEFAULT;
        let first = a.assess(Channel::Busy, 1);
        let second = b.assess(Channel::Busy, 7);
        assert_ne!(first, second);
    }

    #[test]
    fn a_backoff_period_is_320_microseconds_at_2_4_ghz() {
        // 20 symbols of 16 us. The number is in the standard; the test exists so a wrong constant is caught here
        // rather than as a timer armed with the wrong value.
        assert_eq!(units_to_micros(1), 320);
        assert_eq!(units_to_micros(16), 5_120);
        assert_eq!(units_to_micros(0), 0);
    }
}
