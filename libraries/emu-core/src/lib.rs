//! What stops the core from sleeping, and what has to be true before EM2 is entered.
//!
//! The defect this closes, from the 2026-11-17 review (H28): the HAL entered EM2 with a bare `wfi` and no check that
//! anything was pending, and without guarding the EM01 voltage rule. A `wfi` in EM2 with an interrupt already pending
//! returns immediately, so the loop spins instead of sleeping -- measured earlier in this project as a loop that consumed
//! 206 % of real time with the radio on, which is what a "sleep" that does not sleep looks like from outside.
//!
//! The list below is the one the reference manual gives for entering EM2 and it is modelled as an enum, so a caller has to
//! say *which* condition it cleared rather than assert a single boolean.
//!
//! # What refuses the sleep, and in which order it is reported
//!
//! ```
//! use efr32mg2x_emu_core::{may_enter_em2, Blocker, SleepError, VoltageScale};
//!
//! // A quiet core on the low-power scale: EM2 is allowed.
//! assert_eq!(may_enter_em2(VoltageScale::LowPower, &[]), Ok(()));
//!
//! // A DMA channel that is still transferring refuses, and the blocker is carried out so the log names it instead of
//! // reporting "the sleep failed".
//! let busy = [Blocker::PendingInterrupt, Blocker::DmaActive];
//! assert_eq!(
//!     may_enter_em2(VoltageScale::LowPower, &busy),
//!     Err(SleepError::Blocked(Blocker::DmaActive)),
//!     "the reported blocker is the first in the documented order, not the first the caller listed"
//! );
//! assert_eq!(
//!     Blocker::DmaActive.reason(),
//!     "a DMA channel is transferring and needs the bus clock"
//! );
//!
//! // The voltage scale is checked before any blocker: entering EM2 from the performance scale is not a scheduling
//! // question, so a pending interrupt must not be what the caller is told about.
//! assert_eq!(
//!     may_enter_em2(VoltageScale::Performance, &busy),
//!     Err(SleepError::WrongVoltageScale)
//! );
//! ```
#![deny(missing_docs)]
#![no_std]

/// Why EM2 cannot be entered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blocker {
    /// A DMA channel is still transferring: the bus must stay clocked.
    DmaActive,
    /// A peripheral that does not run in EM2 is enabled. The name is carried so the report says which.
    PeripheralNeedsBusClock(&'static str),
    /// An interrupt is pending in the core (`SCB->ICSR` is not clear), so `wfi` would return at once.
    PendingInterrupt,
    /// An event is latched (`SCB->SCR.SEVONPEND` with a pending set), same effect as an interrupt.
    PendingEvent,
}

impl Blocker {
    /// What to say about this blocker in a log.
    pub fn reason(self) -> &'static str {
        match self {
            Blocker::DmaActive => "a DMA channel is transferring and needs the bus clock",
            Blocker::PeripheralNeedsBusClock(_) => {
                "a peripheral that is not available in EM2 is enabled"
            }
            Blocker::PendingInterrupt => "an interrupt is pending, so wfi would return immediately",
            Blocker::PendingEvent => "an event is latched, so the core would not sleep",
        }
    }
}

/// The voltage scale the core must be on to enter EM2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoltageScale {
    /// The high-performance scale used while the core runs fast: EM2 is not allowed from here.
    Performance,
    /// The low-power scale EM2 requires.
    LowPower,
}

/// Why a sleep request was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SleepError {
    /// Something is still active; the blocker is carried so the caller can name it.
    Blocked(Blocker),
    /// The core is on the wrong voltage scale for EM2.
    WrongVoltageScale,
}

/// Whether EM2 may be entered, and why not if it may not.
///
/// The checks are ordered from the most expensive to the cheapest to clear, because the first one reported is the one a
/// person will act on: a DMA transfer that is still running is a bug, while a pending interrupt is the normal path and
/// usually means "do the interrupt handler first".
pub fn may_enter_em2(scale: VoltageScale, blockers: &[Blocker]) -> Result<(), SleepError> {
    if scale != VoltageScale::LowPower {
        return Err(SleepError::WrongVoltageScale);
    }
    // Every blocker in the list is a reason; the first in this order is the one reported.
    for candidate in [
        Blocker::DmaActive,
        Blocker::PendingInterrupt,
        Blocker::PendingEvent,
    ] {
        if let Some(found) = blockers.iter().find(|b| **b == candidate) {
            return Err(SleepError::Blocked(*found));
        }
    }
    if let Some(found) = blockers
        .iter()
        .find(|b| matches!(b, Blocker::PeripheralNeedsBusClock(_)))
    {
        return Err(SleepError::Blocked(*found));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    /// Nothing pending, on the low-power scale: sleep is allowed.
    #[test]
    fn a_quiet_core_may_sleep() {
        assert_eq!(may_enter_em2(VoltageScale::LowPower, &[]), Ok(()));
    }

    /// The wrong voltage scale is refused before anything else is looked at.
    #[test]
    fn the_voltage_scale_comes_first() {
        assert_eq!(
            may_enter_em2(VoltageScale::Performance, &[]),
            Err(SleepError::WrongVoltageScale)
        );
        assert_eq!(
            may_enter_em2(VoltageScale::Performance, &[Blocker::DmaActive]),
            Err(SleepError::WrongVoltageScale),
            "the scale is reported, not the blocker behind it"
        );
    }

    /// Every blocker refuses on its own, and the reason names it.
    #[test]
    fn every_blocker_refuses_alone() {
        let cases = [
            Blocker::DmaActive,
            Blocker::PendingInterrupt,
            Blocker::PendingEvent,
            Blocker::PeripheralNeedsBusClock("USART0"),
        ];
        for blocker in cases {
            let result = may_enter_em2(VoltageScale::LowPower, &[blocker]);
            assert_eq!(
                result,
                Err(SleepError::Blocked(blocker)),
                "{blocker:?} did not block sleep"
            );
            assert!(
                !blocker.reason().is_empty(),
                "a blocker has to be explainable"
            );
        }
    }

    /// The reported blocker is the first in the documented order, not whichever the caller listed first.
    #[test]
    fn the_order_of_the_report_is_the_documented_one() {
        let both = [
            Blocker::PendingInterrupt,
            Blocker::DmaActive,
            Blocker::PeripheralNeedsBusClock("EUSART0"),
        ];
        assert_eq!(
            may_enter_em2(VoltageScale::LowPower, &both),
            Err(SleepError::Blocked(Blocker::DmaActive)),
            "a transfer still running outranks a pending interrupt"
        );
    }

    /// The 206 % case, stated as the defect it is: a loop that calls `wfi` with an interrupt pending does not sleep, so the
    /// sleep loop and the work loop are the same loop.
    #[test]
    fn a_pending_interrupt_would_not_sleep() {
        // This is why the check exists rather than a bare `wfi`: with the flag set, `wfi` returns at once.
        let blocked = may_enter_em2(VoltageScale::LowPower, &[Blocker::PendingInterrupt]);
        assert!(blocked.is_err());
        assert_eq!(
            blocked.unwrap_err(),
            SleepError::Blocked(Blocker::PendingInterrupt)
        );
    }

    /// Duplicated blockers are still one refusal: the function does not care how many times something was reported.
    #[test]
    fn duplicates_do_not_change_the_answer() {
        let twice = [Blocker::DmaActive, Blocker::DmaActive];
        assert_eq!(
            may_enter_em2(VoltageScale::LowPower, &twice),
            Err(SleepError::Blocked(Blocker::DmaActive))
        );
    }
}
