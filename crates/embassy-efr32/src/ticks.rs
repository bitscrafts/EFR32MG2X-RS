//! Ticks: embassy counts in a 64-bit monotonic tick, the SYSRTC in a 32-bit counter at 32768 Hz.
//!
//! Everything here is pure. That is deliberate: converting between the two, and turning a 64-bit
//! deadline into a 32-bit comparator value, is where a driver goes wrong in ways a board will not
//! tell you about -- an alarm that fires late shows up as a slow task, not as an error.
//!
//! Two rules the tests below exist to hold:
//!
//! - a deadline in the past is armed *now*, not after the counter wraps (36.4 h at this rate);
//! - a deadline further away than the counter can express is clamped, and the caller is told, so it
//!   can be re-armed later instead of silently waiting for a wrap.

/// The SYSRTC rate. Embassy works in ticks; this is the conversion the whole tree shares.
pub const TICK_HZ: u64 = 32_768;

/// The counter is 32 bits wide; a comparator value is that width.
pub const COUNTER_MAX: u64 = u32::MAX as u64;

/// Where the comparator should point, and whether the caller must re-arm afterwards.
///
/// `now` and `deadline` are 64-bit ticks. The result is a 32-bit counter value:
///
/// - a deadline already passed gives `now` (fire immediately -- the defect this project paid for
///   once: a past deadline armed at its own value never matched until the wrap);
/// - a deadline inside the counter's reach gives its low 32 bits, no re-arm;
/// - a deadline further away gives the last value the counter can express, and asks for a re-arm.
pub fn comparator_value(now: u64, deadline: u64) -> (u32, bool) {
    let delta = deadline.saturating_sub(now);
    if deadline <= now {
        return (now as u32, false);
    }
    if delta >= COUNTER_MAX {
        // Arm as late as this counter can express; the caller arms again when it gets there.
        return (((now + COUNTER_MAX) & COUNTER_MAX) as u32, true);
    }
    (deadline as u32, false)
}

/// Milliseconds to ticks, rounded up so a `delay(1 ms)` never becomes zero ticks.
pub fn ms_to_ticks(ms: u64) -> u64 {
    (ms * TICK_HZ).div_ceil(1000)
}

/// Ticks to milliseconds, rounded down -- a duration is not overstated.
pub fn ticks_to_ms(ticks: u64) -> u64 {
    ticks * 1000 / TICK_HZ
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_deadline_in_the_past_fires_now() {
        let (value, rearm) = comparator_value(1_000, 500);
        assert_eq!(
            value, 1_000,
            "a deadline that has passed must not wait for the wrap"
        );
        assert!(!rearm);
    }

    #[test]
    fn a_deadline_equal_to_now_fires_now() {
        // The comparator does not match on equality after the counter has moved past it.
        let (value, _) = comparator_value(1_000, 1_000);
        assert_eq!(value, 1_000);
    }

    #[test]
    fn a_near_deadline_is_its_own_low_32_bits() {
        let (value, rearm) = comparator_value(10, 10 + 5_000);
        assert_eq!(value, 5_010);
        assert!(!rearm);
    }

    #[test]
    fn a_deadline_beyond_the_counter_is_clamped_and_asks_for_a_rearm() {
        let (value, rearm) = comparator_value(0, COUNTER_MAX + 10);
        assert_eq!(value, COUNTER_MAX as u32);
        assert!(rearm, "a clamped deadline must ask to be armed again");
    }

    #[test]
    fn a_whole_counter_ahead_is_still_clamped() {
        let (_, rearm) = comparator_value(123, 123 + COUNTER_MAX);
        assert!(rearm);
    }

    #[test]
    fn one_millisecond_is_never_zero_ticks() {
        assert!(ms_to_ticks(1) >= 1, "a 1 ms delay must not vanish");
        assert_eq!(ms_to_ticks(1000), TICK_HZ);
    }

    #[test]
    fn a_second_is_a_thousand_milliseconds_back() {
        assert_eq!(ticks_to_ms(TICK_HZ), 1000);
        assert_eq!(ticks_to_ms(TICK_HZ / 2), 500);
    }
}
