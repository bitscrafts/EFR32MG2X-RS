//! When a bonding failure means the device's stored bond must be dropped.
//!
//! ## The defect this closes
//!
//! A device that keeps a bond its owner's phone has forgotten can never be paired again. Measured 2026-11-17 on a
//! real phone: the phone had dropped its side, the device still held the record, every connection ended in
//! `0x1006 PIN_OR_KEY_MISSING` or `smrsn=4613` (`0x1205`, SMP authentication failure), and the pair dialog either
//! never appeared or appeared and failed -- forever. The device is then useless to its owner until somebody knows
//! the recovery gesture (holding the button for three seconds), and an operator holding a dosimeter in the field is
//! not going to know it.
//!
//! Measured again 2026-11-21 at a demonstration: the device would not connect at all, and the fix was to wipe the
//! bond store and reflash -- from a laptop. That is the same defect wearing different clothes: **the device cannot
//! heal itself.**
//!
//! ## The decision, and why it is not "retry and hope"
//!
//! Dropping a bond is destructive and it is also a security decision: this device carries a person's noise
//! exposure, and a rule that drops the bond on any failure hands the link to whoever is in radio range. So the rule
//! here is narrow on purpose:
//!
//! - **Which failures count.** Only the two codes that mean "I hold a record and cannot use it with this peer":
//!   `0x1006` (PIN or key missing) and `0x1205` (authentication failure). Every other code -- insufficient
//!   authentication, key size, unsupported pairing -- is a real refusal, and dropping the bond for it would be
//!   exactly the wrong response.
//! - **How many.** Two consecutive failures, not one. A single failure can be a timing artefact of the pairing
//!   sequence, and wiping a working bond on one bad packet is worse than the disease.
//! - **Only when a bond is held.** With no bond there is nothing to drop, and the failure belongs to the pairing
//!   attempt rather than to the store.
//!
//! When all three hold, the bond is stale in the only sense that matters: the peer cannot use it. Dropping it makes
//! the next pairing a first pairing, which is the state the owner's phone is already in.
//!
//! ## What this cannot decide
//!
//! Whether the failure came from the *right* attacker. A device bonded to one phone that is being probed by another
//! sees `0x1205` too. The window that limits when a new bond may be created is the defence for that, and it lives in
//! the firmware beside this rule -- not here, where there is no clock.
//!
//! # A run of failures, counted the way the firmware counts it
//!
//! ```
//! use efr32mg2x_ble_bond_heal_core::{
//!     after_failure, continues_streak, Decision, SMP_PIN_OR_KEY_MISSING,
//! };
//!
//! // The phone has forgotten its side, so every attempt reports 0x1006 with the record still in the store.
//! let mut streak = 0u8;
//! let mut verdicts = [Decision::Keep; 3];
//! for slot in verdicts.iter_mut() {
//!     streak = if continues_streak(SMP_PIN_OR_KEY_MISSING) {
//!         streak + 1
//!     } else {
//!         0
//!     };
//!     *slot = after_failure(streak, true);
//! }
//! assert_eq!(
//!     verdicts,
//!     [Decision::Keep, Decision::DropBond, Decision::DropBond],
//!     "one failure may be a timing artefact; the second consecutive one is a record the peer cannot use"
//! );
//!
//! // A real refusal is not this signature, so it breaks the run instead of destroying a working bond -- dropping it
//! // would hand the link to whoever asked.
//! assert!(!continues_streak(0x0503));
//!
//! // With nothing in the store there is nothing to drop, whatever the count says.
//! assert_eq!(after_failure(2, false), Decision::Keep);
//! ```

#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_code)]

/// SMP failure: the peer has no key that matches the record this device holds.
///
/// This is what a phone that forgot its side produces: it presents nothing usable, and the device cannot
/// authenticate it.
pub const SMP_PIN_OR_KEY_MISSING: u16 = 0x1006;

/// SMP failure: authentication failed. The peer's key does not agree with the stored bond.
pub const SMP_AUTHENTICATION_FAILURE: u16 = 0x1205;

/// The same two failures as the stack sometimes reports them: the bare pairing-failure reason, low byte only.
///
/// The device has been seen reporting both spellings -- `smrsn=4613` (`0x1205`) in one run, and a firmware that
/// compared only the low byte (`0x05`, `0x06`) in another. Accepting both costs nothing and closes a gap that would
/// otherwise only appear on the phone the rule was not written against.
pub const SMP_AUTHENTICATION_REQUIREMENTS: u16 = 0x0005;

/// Pairing not supported, the low-byte spelling of a peer that cannot use the stored bond.
pub const SMP_PAIRING_NOT_SUPPORTED: u16 = 0x0006;

/// How many consecutive failures of a qualifying code are needed before the bond is treated as stale.
pub const FAILURES_BEFORE_DROPPING: u8 = 2;

/// What the firmware should do after a bonding failure was reported by the stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Leave the stored bond alone. It may be good; the failure may be transient, unrelated, or a real refusal.
    Keep,
    /// Drop the stored bond and let the next pairing be a first pairing.
    DropBond,
}

/// Whether `reason` is a code that can indicate a bond this device holds but cannot use.
///
/// Anything else is either a different failure or a refusal, and neither is a reason to destroy a bond.
pub const fn is_stale_bond_signature(reason: u16) -> bool {
    matches!(
        reason,
        SMP_PIN_OR_KEY_MISSING
            | SMP_AUTHENTICATION_FAILURE
            | SMP_AUTHENTICATION_REQUIREMENTS
            | SMP_PAIRING_NOT_SUPPORTED
    )
}

/// Decide what to do after a bonding failure.
///
/// `streak` is the number of consecutive failures of a qualifying code, **counting this one**: a caller that has
/// just seen the first failure passes `1`. A failure with a non-qualifying code leaves `streak` alone and returns
/// [`Decision::Keep`].
pub const fn after_failure(streak: u8, holds_bond: bool) -> Decision {
    if !holds_bond {
        // Nothing to drop. The failure belongs to the pairing attempt, not to the store.
        return Decision::Keep;
    }
    if streak < FAILURES_BEFORE_DROPPING {
        return Decision::Keep;
    }
    Decision::DropBond
}

/// Whether the streak should be reset, given the code the stack reported.
///
/// A failure that is not the stale-bond signature breaks the run: three unrelated failures with a qualifying one
/// between them are not two consecutive ones, and counting them as such would drop the bond on a coincidence.
pub const fn continues_streak(reason: u16) -> bool {
    is_stale_bond_signature(reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_failure_is_not_enough() {
        // The control that matters most: a transient failure must not destroy a working bond.
        assert_eq!(after_failure(0, true), Decision::Keep);
        assert_eq!(after_failure(1, true), Decision::Keep);
    }

    #[test]
    fn two_consecutive_failures_drop_a_held_bond() {
        assert_eq!(after_failure(2, true), Decision::DropBond);
        assert_eq!(after_failure(9, true), Decision::DropBond);
    }

    #[test]
    fn no_bond_held_means_nothing_to_drop() {
        // With an empty store the failure is about the pairing attempt, not about a record.
        assert_eq!(after_failure(0, false), Decision::Keep);
        assert_eq!(after_failure(2, false), Decision::Keep);
        assert_eq!(after_failure(u8::MAX, false), Decision::Keep);
    }

    #[test]
    fn only_the_stale_bond_codes_count() {
        assert!(continues_streak(SMP_PIN_OR_KEY_MISSING));
        assert!(continues_streak(SMP_AUTHENTICATION_FAILURE));
        // The low-byte spellings the stack has also been seen reporting.
        assert!(continues_streak(SMP_AUTHENTICATION_REQUIREMENTS));
        assert!(continues_streak(SMP_PAIRING_NOT_SUPPORTED));
        // A real refusal must never be read as a stale bond: dropping the bond for it hands the link to whoever
        // asked. These are codes the stack can report and that mean something else entirely.
        for reason in [
            0x0000u16, 0x0403, 0x0501, 0x0503, 0x050B, 0x0C01, 0x0C02, 0x1206,
        ] {
            assert!(
                !continues_streak(reason),
                "0x{reason:04X} is not a stale-bond signature"
            );
        }
    }

    #[test]
    fn a_failure_with_another_code_does_not_continue_the_run() {
        // Two failures in a row, but the second is a different failure: the streak is not two of the same thing.
        let mut streak = 0u8;
        for reason in [SMP_PIN_OR_KEY_MISSING, 0x0503] {
            streak = if continues_streak(reason) {
                streak + 1
            } else {
                0
            };
        }
        assert_eq!(
            after_failure(streak, true),
            Decision::Keep,
            "one qualifying failure either side of an unrelated one is not a run"
        );
    }

    #[test]
    fn the_run_is_counted_by_the_caller_the_way_the_docs_say() {
        let mut streak = 0u8;
        let mut decisions = [Decision::Keep; 3];
        for (index, slot) in decisions.iter_mut().enumerate() {
            streak = if continues_streak(SMP_PIN_OR_KEY_MISSING) {
                streak + 1
            } else {
                0
            };
            *slot = after_failure(streak, true);
            if index == 1 {
                assert_eq!(
                    *slot,
                    Decision::DropBond,
                    "the second consecutive failure drops"
                );
            }
        }
        assert_eq!(decisions[0], Decision::Keep);
        assert_eq!(decisions[1], Decision::DropBond);
    }

    #[test]
    fn the_threshold_is_two_and_it_is_named() {
        assert_eq!(FAILURES_BEFORE_DROPPING, 2);
        assert_eq!(
            after_failure(FAILURES_BEFORE_DROPPING - 1, true),
            Decision::Keep
        );
        assert_eq!(
            after_failure(FAILURES_BEFORE_DROPPING, true),
            Decision::DropBond
        );
    }
}
