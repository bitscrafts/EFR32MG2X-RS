//! Show which conditions keep the core awake, and what the watchdog's lock rule refuses.
//!
//! Run: `cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-emu-core --example sleep`
//! and `cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-wdog-core --example lock`

use efr32mg2x_emu_core::{may_enter_em2, Blocker, SleepError, VoltageScale};

fn main() {
    let mut failures = 0;

    println!("scale         blockers                                        verdict");
    let cases: [(&str, VoltageScale, &[Blocker]); 6] = [
        ("nothing pending", VoltageScale::LowPower, &[]),
        (
            "a DMA transfer",
            VoltageScale::LowPower,
            &[Blocker::DmaActive],
        ),
        (
            "a pending interrupt",
            VoltageScale::LowPower,
            &[Blocker::PendingInterrupt],
        ),
        (
            "a latched event",
            VoltageScale::LowPower,
            &[Blocker::PendingEvent],
        ),
        (
            "USART0 enabled",
            VoltageScale::LowPower,
            &[Blocker::PeripheralNeedsBusClock("USART0")],
        ),
        (
            "nothing pending, wrong scale",
            VoltageScale::Performance,
            &[],
        ),
    ];

    for (what, scale, blockers) in cases {
        let verdict = match may_enter_em2(scale, blockers) {
            Ok(()) => "sleeps".to_string(),
            Err(SleepError::Blocked(b)) => format!("blocked: {}", b.reason()),
            Err(SleepError::WrongVoltageScale) => "blocked: wrong voltage scale".to_string(),
        };
        println!("{scale:<13?} {what:<47} {verdict}");
    }

    // The one that matters: a quiet core must sleep, and a core with anything pending must not.
    if may_enter_em2(VoltageScale::LowPower, &[]).is_err() {
        println!("FAIL: a quiet core must be allowed to sleep");
        failures += 1;
    }
    for blocker in [
        Blocker::DmaActive,
        Blocker::PendingInterrupt,
        Blocker::PendingEvent,
        Blocker::PeripheralNeedsBusClock("EUSART0"),
    ] {
        if may_enter_em2(VoltageScale::LowPower, &[blocker]).is_ok() {
            println!("FAIL: {blocker:?} was allowed to sleep");
            failures += 1;
        }
    }

    println!(
        "\nthe 206 % case: a `wfi` with an interrupt pending returns at once, so the sleep loop and the work loop\n\
         are the same loop -- which is what the earlier measurement of 206 % of real time actually was"
    );

    if failures == 0 {
        println!("\nPASS: every blocker refuses, and a quiet core on the low-power scale sleeps");
    } else {
        std::process::exit(1);
    }
}
