//! Show the watchdog's timeout encoding, and the lock rule that H27 broke.
//!
//! Run: `cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-wdog-core --example lock`
//!
//! The last block is the defect: the driver wrote `LOCK` with the watchdog disabled, and on this family that leaves it
//! locked with no timeout programmed -- recoverable only by a reset.

use efr32mg2x_wdog_core::{may_lock, period_for_ms, Period, State, MAX_TOP};

fn main() {
    let mut failures = 0;

    println!("clock (Hz)  wanted (ms)  prescaler  top     actual (ms)  error (per thousand)");
    for clock in [32_768u32, 19_000_000] {
        for wanted in [10u32, 100, 500, 1000, 5000] {
            match period_for_ms(clock, wanted) {
                Ok((period, error)) => println!(
                    "{:<11} {:<12} {:<10} {:<7} {:<12} {}",
                    clock,
                    wanted,
                    period.prescaler,
                    period.top,
                    period.period_ms(clock),
                    error
                ),
                Err(e) => {
                    println!("{clock:<11} {wanted:<12} refused: {e:?}");
                    failures += 1;
                }
            }
        }
    }

    // The longest period the fields express, and the refusal beyond it.
    let longest = Period {
        prescaler: 32_768,
        top: MAX_TOP,
    };
    println!(
        "\nlongest expressible period: {} ms ({:.1} h)",
        longest.period_ms(32_768),
        longest.period_ms(32_768) as f64 / 3_600_000.0
    );
    if period_for_ms(32_768, 20 * 3600 * 1000).is_ok() {
        println!("FAIL: twenty hours does not fit the fields and must be refused, not truncated");
        failures += 1;
    } else {
        println!("twenty hours: refused, not truncated");
    }

    println!("\nthe lock rule (H27):");
    println!("  running: {:?}", may_lock(State::Running));
    println!("  disabled: {:?}", may_lock(State::Disabled));
    if may_lock(State::Disabled).is_ok() {
        println!("FAIL: locking a disabled watchdog leaves it locked with no timeout");
        failures += 1;
    }

    if failures == 0 {
        println!("\nPASS: periods are exact or refused, and the lock needs a running watchdog");
    } else {
        std::process::exit(1);
    }
}
