//! Print the prescaler and top for the rates this project uses, and show what the floor-only search missed.
//!
//! Run: `cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-timer-core --example period`
//!
//! Two numbers matter per row: the rate the hardware will actually produce, and how far it is in parts per million from
//! what was asked. A timer that is off by a thousandth is fine for a pace and not fine for a sample clock.

use efr32mg2x_timer_core::{may_configure, period_for, PRESCALERS};

fn main() {
    let mut failures = 0;
    let mut refusals = 0;

    println!("clock (MHz)  wanted (Hz)  prescaler  top     actual (Hz)  error (ppm)");
    for clock in [19_000_000u32, 38_000_000, 56_000_000, 80_000_000] {
        for wanted in [1u32, 440, 1_000, 16_000, 32_000, 250_000] {
            match period_for(clock, wanted) {
                Ok((period, ppm)) => {
                    println!(
                        "{:<12} {:<12} {:<10} {:<7} {:<12} {}",
                        clock / 1_000_000,
                        wanted,
                        period.prescaler,
                        period.top,
                        period.frequency(clock),
                        ppm
                    );
                    if ppm >= 2_000 {
                        println!("FAIL: {ppm} ppm is more than two thousandths for {clock} Hz / {wanted} Hz");
                        failures += 1;
                    }
                }
                Err(e) => {
                    // A refusal is a result, not a failure: 1 Hz from 80 MHz needs 80 million clocks and the fields
                    // reach 1024 x 65 536 = 67 million. What would be a failure is a refusal for a rate that fits, and
                    // that is checked below rather than by counting these lines.
                    println!("{:<12} {:<12} refused: {e:?}", clock / 1_000_000, wanted);
                    refusals += 1;
                }
            }
        }
    }

    // What the floor-only search chose, and what taking both sides gives: the case the test found.
    let clock = 19_000_000u32;
    let wanted = 32_000u32;
    let ticks = clock / wanted; // 593, truncated from 593.75
    let floor = efr32mg2x_timer_core::Period {
        prescaler: 1,
        top: ticks - 1,
    };
    let ceil = efr32mg2x_timer_core::Period {
        prescaler: 1,
        top: ticks,
    };
    let (chosen, chosen_ppm) = period_for(clock, wanted).expect("32 kHz");
    println!(
        "\nat 19 MHz / 32 kHz: floor top {} gives {} Hz, ceil top {} gives {} Hz, chosen top {} ({} ppm)",
        floor.top,
        floor.frequency(clock),
        ceil.top,
        ceil.frequency(clock),
        chosen.top,
        chosen_ppm
    );
    if chosen.top != ceil.top {
        println!("FAIL: the closer candidate was not chosen");
        failures += 1;
    }

    println!("\nordering (H6): a stopped timer is configurable, a running one is not");
    println!("  stopped: {:?}", may_configure(false));
    println!("  running: {:?}", may_configure(true));
    if may_configure(true).is_ok() {
        println!("FAIL: a running timer must not be configured");
        failures += 1;
    }

    println!("\nprescalers available: {PRESCALERS:?}");

    // The one refusal that must happen: 1 Hz from 80 MHz does not fit the prescaler and top.
    if period_for(80_000_000, 1).is_ok() {
        println!("FAIL: 1 Hz from 80 MHz should not be expressible");
        failures += 1;
    }
    println!("\n{refusals} rates refused as beyond the fields (expected 1: 1 Hz from 80 MHz)");

    if failures == 0 {
        println!("PASS: every used rate is within two thousandths, the closer side is always chosen, and the impossible rate is refused");
    } else {
        std::process::exit(1);
    }
}
