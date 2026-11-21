//! What the device does when a phone that once bonded to it has forgotten it.
//!
//! Walks the rule through the three sequences that matter: the phone that forgot (the bond must go), a single
//! failure (the bond must stay), and a failure that is a real refusal rather than a stale bond (the bond must
//! stay).
//!
//! Run on the host, no radio:
//!
//! ```text
//! cargo run -p efr32mg2x-ble-bond-heal-core --target x86_64-unknown-linux-gnu --example a_phone_that_forgot
//! ```

use efr32mg2x_ble_bond_heal_core::{
    after_failure, continues_streak, Decision, SMP_AUTHENTICATION_FAILURE, SMP_PIN_OR_KEY_MISSING,
};

/// Feed a sequence of bonding failures to the rule, exactly as the firmware does, and print what it decided.
fn walk(title: &str, holds_bond: bool, reasons: &[u16]) {
    println!("{title}");
    println!("  bond held: {holds_bond}");
    let mut streak = 0u8;
    for (index, reason) in reasons.iter().enumerate() {
        streak = if continues_streak(*reason) {
            streak.saturating_add(1)
        } else {
            0
        };
        let decision = after_failure(streak, holds_bond);
        let what = if continues_streak(*reason) {
            "counts towards a stale bond"
        } else {
            "is a refusal, not a stale bond"
        };
        println!(
            "  failure {}: 0x{reason:04X} {what} -> streak {streak}, {}",
            index + 1,
            match decision {
                Decision::Keep => "keep the bond",
                Decision::DropBond => "DROP THE BOND and let the phone pair again",
            }
        );
        if decision == Decision::DropBond {
            // What the firmware does next, so the example shows the whole response and not half of it.
            println!("     -> wipe the external bond store, then advertise again");
            break;
        }
    }
    println!();
}

fn main() {
    walk(
        "The phone forgot the device (the 2026-11-21 demonstration):",
        true,
        &[SMP_AUTHENTICATION_FAILURE, SMP_AUTHENTICATION_FAILURE],
    );

    walk(
        "One failure, then the link recovers:",
        true,
        &[SMP_AUTHENTICATION_FAILURE],
    );

    walk(
        "A real refusal -- insufficient authentication, nothing to do with a stale bond:",
        true,
        &[0x0503, 0x0503, 0x0503],
    );

    walk(
        "Failures separated by an unrelated one are not a run:",
        true,
        &[SMP_PIN_OR_KEY_MISSING, 0x0503, SMP_PIN_OR_KEY_MISSING],
    );

    walk(
        "No bond held -- there is nothing to drop:",
        false,
        &[SMP_PIN_OR_KEY_MISSING, SMP_PIN_OR_KEY_MISSING],
    );
}
