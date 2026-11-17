//! Deadlines and wrap: the arithmetic SYSRTC waits do, measured across the 32-bit boundary.
//!
//! The defect this closes, from the 2026-11-17 review (H25): a deadline already in the past never fired until the counter
//! wrapped -- 36.4 hours at 32.768 kHz. The driver compared `now < deadline` with a plain integer comparison, which is
//! only correct while the counter never wraps and the deadline is ahead.
//!
//! The rule that makes it correct is the one every embedded timer uses and every hand-rolled comparison gets wrong: a
//! deadline is in the **future** when `deadline - now`, taken in wrapping arithmetic, is smaller than half the range. That
//! is the same test as "the signed difference is positive", written without signed arithmetic and without assuming
//! anything about where in the range the two values sit.

#![no_std]

/// The SYSRTC counter is 32 bits at 32.768 kHz: 2^32 ticks is 36.4 hours.
pub const COUNTER_HZ: u32 = 32_768;
/// Ticks in the 32-bit counter: 36.4 hours at `COUNTER_HZ`.
pub const WRAP_TICKS: u64 = 1 << 32;

/// Whether `deadline` is still in the future relative to `now`, across the wrap.
///
/// Half the range is the horizon: a deadline further than 2^31 ticks (18.2 hours) away cannot be told from one just
/// past, which is the price of a 32-bit counter and the reason a deadline is scheduled in the near future.
pub fn is_future(now: u32, deadline: u32) -> bool {
    deadline.wrapping_sub(now) < 0x8000_0000
}

/// Ticks from `now` to `deadline`, in wrapping arithmetic.
pub fn ticks_until(now: u32, deadline: u32) -> u32 {
    deadline.wrapping_sub(now)
}

/// Whether a deadline has been reached, which is the negation of `is_future`.
pub fn has_passed(now: u32, deadline: u32) -> bool {
    !is_future(now, deadline)
}

/// The tick count for a duration in milliseconds, saturating at the counter's range.
pub fn ticks_for_ms(ms: u32) -> u32 {
    let ticks = (ms as u64 * COUNTER_HZ as u64) / 1000;
    if ticks > u32::MAX as u64 {
        u32::MAX
    } else {
        ticks as u32
    }
}

/// The duration a tick count represents, in whole milliseconds.
pub fn ms_for_ticks(ticks: u32) -> u32 {
    ((ticks as u64 * 1000) / COUNTER_HZ as u64) as u32
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    /// The ordinary case: a deadline ahead is in the future, one behind has passed.
    #[test]
    fn the_ordinary_case() {
        assert!(is_future(1000, 2000));
        assert!(!is_future(2000, 1000));
        assert!(has_passed(2000, 1000));
        assert_eq!(ticks_until(1000, 2000), 1000);
    }

    /// A deadline on the other side of the wrap is still in the future -- this is the whole point.
    #[test]
    fn a_deadline_across_the_wrap_is_in_the_future() {
        let now = u32::MAX - 100;
        let deadline = 200; // wraps
        assert!(
            is_future(now, deadline),
            "{now} -> {deadline} crosses the wrap"
        );
        assert_eq!(
            ticks_until(now, deadline),
            301,
            "101 ticks to the wrap and 200 after it"
        );
        assert!(!has_passed(now, deadline));
    }

    /// The defect, reproduced: a deadline that has just passed is *not* in the future, and a plain `<` comparison says it
    /// is. This is the test that fails on the old shape and passes on the new one.
    #[test]
    fn a_plain_comparison_is_wrong_across_the_wrap() {
        let now = 100u32;
        let deadline = u32::MAX - 50; // passed 151 ticks ago, but numerically much larger
        assert!(
            deadline > now,
            "the naive comparison sees a huge future deadline"
        );
        assert!(!is_future(now, deadline), "and it has actually passed");
        assert_eq!(
            ticks_until(now, deadline),
            u32::MAX - 150,
            "151 ticks behind"
        );
    }

    /// The horizon is half the range, and it is symmetric: at exactly half, either answer is a coin toss and the function
    /// must at least not claim the far side is near.
    #[test]
    fn the_horizon_is_half_the_range() {
        assert!(is_future(0, 0x7FFF_FFFF), "just inside the horizon");
        assert!(
            !is_future(0, 0x8000_0000),
            "exactly half is not in the future"
        );
        assert!(
            !is_future(0, 0x8000_0001),
            "beyond the horizon reads as past"
        );
    }

    /// Durations round-trip within a tick, over the ranges this device uses.
    #[test]
    fn durations_round_trip() {
        for ms in [1u32, 2, 10, 16, 100, 1000, 60_000] {
            let ticks = ticks_for_ms(ms);
            let back = ms_for_ticks(ticks);
            let tolerance = if ms >= 1000 { 1 } else { 1 };
            assert!(
                back.abs_diff(ms) <= tolerance,
                "{ms} ms -> {ticks} ticks -> {back} ms, more than {tolerance} ms off"
            );
        }
        assert_eq!(ticks_for_ms(0), 0);
        assert_eq!(ms_for_ticks(0), 0);
    }

    /// The wrap is 36.4 hours, and the horizon is half of it -- the number the review named.
    #[test]
    fn the_wrap_is_36_hours() {
        let hours = WRAP_TICKS as f64 / COUNTER_HZ as f64 / 3600.0;
        assert!((hours - 36.4).abs() < 0.1, "the wrap is {hours} hours");
        let horizon_hours = 0x8000_0000u64 as f64 / COUNTER_HZ as f64 / 3600.0;
        assert!(
            (horizon_hours - 18.2).abs() < 0.1,
            "the horizon is {horizon_hours} hours"
        );
    }
}
