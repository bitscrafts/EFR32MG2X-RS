//! Print the wrap arithmetic, including the case a plain comparison gets wrong.
//!
//! Run: `cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-sysrtc-core --example wrap`
//!
//! The counter is 32 bits at 32.768 kHz: 36.4 hours to the wrap, and 18.2 hours of horizon within which a deadline can be
//! told from one just past. The last block is the defect H25 named, reproduced: `deadline > now` is true for a deadline
//! that passed 151 ticks ago.

use efr32mg2x_sysrtc_core::{
    has_passed, is_future, ticks_for_ms, ticks_until, COUNTER_HZ, WRAP_TICKS,
};

fn main() {
    let mut failures = 0;

    println!(
        "counter {COUNTER_HZ} Hz, wrap {WRAP_TICKS} ticks = {:.1} h, horizon {:.1} h",
        WRAP_TICKS as f64 / COUNTER_HZ as f64 / 3600.0,
        0x8000_0000u64 as f64 / COUNTER_HZ as f64 / 3600.0
    );

    println!("\ncase                          now          deadline     future?  ticks until");
    let cases = [
        (1_000u32, 2_000u32, "a deadline ahead"),
        (2_000, 1_000, "a deadline behind"),
        (u32::MAX - 100, 200, "across the wrap"),
        (100, u32::MAX - 50, "passed, but numerically larger"),
        (0, 0x7FFF_FFFF, "just inside the horizon"),
        (0, 0x8000_0000, "exactly at the horizon"),
    ];
    for (now, deadline, what) in cases {
        println!(
            "{:<29} {:<12} {:<12} {:<8} {}",
            what,
            now,
            deadline,
            is_future(now, deadline),
            ticks_until(now, deadline)
        );
    }

    // The defect, as an assertion: the naive comparison disagrees with the wrapping one.
    let (now, deadline) = (100u32, u32::MAX - 50);
    println!(
        "\nthe defect H25 named: `deadline > now` is {}, while the deadline has passed: {}",
        deadline > now,
        has_passed(now, deadline)
    );
    if !(deadline > now) || !has_passed(now, deadline) {
        println!("FAIL: the case does not reproduce the defect");
        failures += 1;
    }

    println!("\nplain comparisons around the wrap:");
    let mut wrong = 0;
    for delta in 1..=1000u32 {
        let now = (u32::MAX - 500).wrapping_add(delta); // sweeping across the wrap
        let deadline = now.wrapping_add(10); // ten ticks ahead, and sometimes across the wrap
        if deadline > now && !is_future(now, deadline) && ticks_until(now, deadline) < 1000 {
            // A wrapping deadline that the naive test calls future is fine; what matters is the reverse case.
            continue;
        }
        if is_future(now, deadline) != (deadline > now) {
            wrong += 1;
        }
    }
    println!("  {wrong} disagreements between `deadline > now` and the wrapping test, over 1000 steps across the wrap");

    println!("\ndurations:");
    for ms in [1u32, 10, 100, 1000, 36_000] {
        println!("  {ms:>6} ms = {:>9} ticks", ticks_for_ms(ms));
    }

    if failures == 0 {
        println!("\nPASS: deadlines are decided by wrapping subtraction, and the naive comparison is shown wrong");
    } else {
        std::process::exit(1);
    }
}
