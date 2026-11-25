//! Input capture: the three ways a captured value lies, as arithmetic.
//!
//! A capture is the counter's value at an edge, latched by the hardware. It lies in three ways, and each one is a decision
//! the driver has to make rather than a register it has to write:
//!
//! 1. **It wraps.** `TIMER0` counters are 16 bits. An edge at `0xFFF0` followed by one at `0x0010` is 32 ticks apart, not
//!    `-65504`; the arithmetic has to be wrapping, and a driver that subtracts into an `i32` reports a negative interval.
//! 2. **The register keeps the newest edge.** Two edges arriving before the code reads leave one value — the second. The
//!    first is gone and nobody is told, so the driver has to count the misses or it reports a period it never measured.
//! 3. **A value read twice is not two measurements.** A capture register is not a queue; reading it again returns what it
//!    returned before, and a caller that treats that as a new edge builds a flat line out of nothing.
//!
//! The first is a subtraction, the second and third are a small state machine, and all three can be tested on the host.

/// The counter's width for every `TIMERn` on this part.
pub const COUNTER_BITS: u32 = 16;

/// Ticks in one full wrap of the counter.
pub const WRAP_TICKS: u32 = 1 << COUNTER_BITS;

/// How far apart two captured values are, in ticks, with the wrap accounted for.
///
/// Wrapping subtraction on the 16-bit values, widened after: `from = 0xFFF0` to `to = 0x0010` is 32 ticks.
#[must_use]
pub fn elapsed(from: u16, to: u16) -> u32 {
    u32::from(to.wrapping_sub(from))
}

/// The same, extended by a count of whole wraps observed between the two captures.
///
/// The count comes from the overflow interrupt or a polling loop; it is the caller's, because only the caller knows whether
/// it was watching.
#[must_use]
pub fn elapsed_with_wraps(from: u16, to: u16, wraps: u32) -> u32 {
    wraps.saturating_mul(WRAP_TICKS) + elapsed(from, to)
}

/// Why a captured value cannot be turned into an interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureError {
    /// Nothing has been captured since the last read.
    Nothing,
    /// The counter wrapped, the caller was not counting wraps, and the interval is therefore unknowable rather than large.
    ///
    /// The distinction matters: a delta that came back smaller than the last one can mean "faster" or "wrapped", and the
    /// two are opposite conclusions from the same number.
    Ambiguous,
}

/// A one-deep capture slot, as the hardware has it.
///
/// The hardware keeps the **newest** edge, which is the opposite of a received word in a UART: there, the byte that
/// arrived first is the one the other side already moved past; here, the latest edge is the one the caller is timing.
/// The slot reports the miss either way, so a period computed from a value that replaced another is not presented as a
/// clean measurement.
#[derive(Debug, Clone, Copy, Default)]
pub struct CaptureSlot {
    value: u16,
    fresh: bool,
}

impl CaptureSlot {
    /// An empty slot.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            value: 0,
            fresh: false,
        }
    }

    /// Record an edge the hardware captured.
    ///
    /// Returns whether this edge **replaced an unread one**, which is the miss a caller has to know about: the interval it
    /// is about to compute is from the newer edge, not from the one it expected.
    pub fn mark(&mut self, value: u16) -> bool {
        let replaced = self.fresh;
        self.value = value;
        self.fresh = true;
        replaced
    }

    /// Take the captured value. A second call returns `Err(CaptureError::Nothing)` rather than the same number again.
    pub fn take(&mut self) -> Result<u16, CaptureError> {
        if !self.fresh {
            return Err(CaptureError::Nothing);
        }
        self.fresh = false;
        Ok(self.value)
    }

    /// Whether an edge is waiting.
    #[must_use]
    pub fn is_fresh(&self) -> bool {
        self.fresh
    }
}

#[cfg(test)]
mod capture_tests {
    use super::*;

    /// The wrap, which is the reason the subtraction is on `u16` and not on `i32`: an edge before the wrap and an edge
    /// after it are close together, and the numbers do not say so.
    #[test]
    fn an_interval_across_the_wrap_is_positive_and_small() {
        assert_eq!(elapsed(0xFFF0, 0x0010), 32);
        assert_eq!(elapsed(0x0010, 0xFFF0), WRAP_TICKS - 32);
        assert_eq!(elapsed(0x1234, 0x1234), 0);
    }

    /// Whole wraps add up, and a caller that watched them gets the long interval it expects.
    #[test]
    fn wraps_are_added_to_the_wrapped_delta() {
        assert_eq!(elapsed_with_wraps(0xFFF0, 0x0010, 1), WRAP_TICKS + 32);
        assert_eq!(elapsed_with_wraps(0x1000, 0x1000, 0), 0);
    }

    /// The normal path: one edge, read once.
    #[test]
    fn an_edge_is_taken_once() {
        let mut slot = CaptureSlot::new();
        assert_eq!(slot.take(), Err(CaptureError::Nothing));
        assert!(
            !slot.mark(0x0100),
            "nothing was waiting, so nothing was replaced"
        );
        assert_eq!(slot.take(), Ok(0x0100));
        assert!(!slot.is_fresh());
    }

    /// The defect a capture slot exists to expose: two edges before the read. The hardware keeps the second, so the
    /// interval is measured from the wrong one unless the miss is reported.
    #[test]
    fn an_edge_that_replaced_an_unread_one_is_reported_and_the_newest_is_kept() {
        let mut slot = CaptureSlot::new();
        assert!(!slot.mark(0x0100));
        assert!(
            slot.mark(0x0200),
            "the first edge was never read and is gone"
        );
        assert_eq!(
            slot.take(),
            Ok(0x0200),
            "the hardware keeps the newest edge"
        );
    }

    /// And the control for the test above: read in between, and neither edge is reported as a replacement. A slot that
    /// reported a miss on every second mark would also pass the test above.
    #[test]
    fn edges_read_in_between_are_not_misses() {
        let mut slot = CaptureSlot::new();
        assert!(!slot.mark(0x0100));
        assert_eq!(slot.take(), Ok(0x0100));
        assert!(!slot.mark(0x0200));
        assert_eq!(slot.take(), Ok(0x0200));
    }

    /// A value read twice is not two measurements.
    #[test]
    fn reading_twice_does_not_invent_an_edge() {
        let mut slot = CaptureSlot::new();
        slot.mark(0x0BAD);
        assert_eq!(slot.take(), Ok(0x0BAD));
        assert_eq!(slot.take(), Err(CaptureError::Nothing));
    }
}
