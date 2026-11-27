//! Print the ladder table the comparator is read against, and the window the board's record sits in.
//!
//! Run: `cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-acmp-core --example ladder-table`
//!
//! Two numbers on the part's record came from this arithmetic rather than from the register: the millivolt value of a
//! tap, and the interval a crossing brackets. Printing them here, on a host, keeps the board's line auditable -- the
//! record claims `833..853`, and this says where that came from.

use efr32mg2x_acmp_core::{ladder_mv, overlap, window_mv, LADDER_FULL};

fn main() {
    let mut failures = 0;

    println!("the 1.25 V reference, every eighth tap:");
    for div in (0..=LADDER_FULL).step_by(8) {
        println!("  tap {div:2} -> {:4} mV", ladder_mv(1_250, div));
    }

    // The two crossings measured on the board on 2026-11-27 (docs/04-reference/ACMP-LADDER-2026-11-27.md).
    let measured = [(1_250u32, 43u8), (2_500, 22)];
    let mut windows = [(0u32, 0u32); 2];
    for (i, (reference, crossing)) in measured.iter().enumerate() {
        match window_mv(*reference, *crossing) {
            Some(window) => {
                println!(
                    "{reference} mV reference, first low tap {crossing}: the signal is in {}..{} mV",
                    window.0, window.1
                );
                windows[i] = window;
            }
            None => {
                println!("{reference} mV reference, tap {crossing}: not a crossing");
                failures += 1;
            }
        }
    }

    match overlap(windows[0], windows[1]) {
        Some(common) => println!("the two crossings agree on {}..{} mV", common.0, common.1),
        None => {
            println!("the two crossings disagree -- one of them is not the ladder");
            failures += 1;
        }
    }

    // The control the crate exists for: the arithmetic must refuse a sweep that never turned.
    if window_mv(1_250, 0).is_none() && ladder_mv(1_250, LADDER_FULL) == 1_250 {
        println!("a sweep that never turned is not a window, and the pass-through tap is the reference itself");
    } else {
        println!("the ladder arithmetic is wrong");
        failures += 1;
    }

    if failures == 0 {
        println!("PASS ladder-table: the arithmetic behind the board's 833..853 mV window");
    } else {
        println!("FAIL ladder-table: {failures} disagreement(s)");
        std::process::exit(1);
    }
}
