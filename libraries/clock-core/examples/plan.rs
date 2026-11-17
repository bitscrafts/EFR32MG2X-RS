//! Print the plan for every band, and prove the ceilings hold over the whole range.
//!
//! Run: `cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-clock-core --example plan`
//!
//! The last block is the one that matters: it walks every megahertz from 1 to 80 and asserts that no plan puts the
//! peripheral bus over 50 MHz or the radio clock over 40 MHz. That is the invariant the hand-written boost in the HAL
//! broke at 80 MHz, and here it is a check rather than a memory.

use efr32mg2x_clock_core::{plan, Divider, MAX_PCLK_HZ, MAX_RHCLK_HZ};

fn divider(d: Divider) -> &'static str {
    match d {
        Divider::Div1 => "div1",
        Divider::Div2 => "div2",
    }
}

fn main() {
    let mut failures = 0;

    println!("core (MHz)  DEVINFO  WS   PCLK    radio");
    for hz in [19_000_000u32, 38_000_000, 56_000_000, 80_000_000] {
        match plan(hz) {
            Ok(plan) => println!(
                "{:<11} {:<8} {:<4} {:<7} {}",
                hz / 1_000_000,
                plan.devinfo_index,
                plan.wait_states,
                divider(plan.pclk),
                divider(plan.rhclk)
            ),
            Err(e) => {
                println!("{:<11} refused: {e:?}", hz / 1_000_000);
                failures += 1;
            }
        }
    }

    // A sweep that only looks for violations is vacuous when almost every frequency is refused -- the first version of
    // this program printed "0 frequencies plannable" and called that a pass. So the sweep also counts what it planned,
    // and the count has to match the table.
    println!("\ninvariants over every megahertz from 1 to 80:");
    let mut checked = 0;
    let mut worst_pclk = 0u32;
    let mut worst_rhclk = 0u32;
    // Whole megahertz from 0, which is where the calibrated bands live: a sweep from 1 with a 1 MHz step misses
    // every one of them (19 000 000 is not on that sequence), which is how the first version of this program measured
    // nothing and said PASS.
    for mhz in 0..=80u32 {
        let hz = mhz * 1_000_000;
        let Ok(plan) = plan(hz) else { continue };
        checked += 1;
        worst_pclk = worst_pclk.max(plan.pclk_hz());
        worst_rhclk = worst_rhclk.max(plan.rhclk_hz());
        if plan.pclk_hz() > MAX_PCLK_HZ || plan.rhclk_hz() > MAX_RHCLK_HZ {
            println!("FAIL: {hz} Hz puts a bus over its ceiling");
            failures += 1;
        }
    }
    if checked != 4 {
        println!("FAIL: {checked} frequencies planned, expected exactly the 4 bands -- the sweep is not measuring");
        failures += 1;
    }
    println!(
        "{checked} frequencies plannable; worst PCLK {} MHz (ceiling {}), worst radio {} MHz (ceiling {})",
        worst_pclk / 1_000_000,
        MAX_PCLK_HZ / 1_000_000,
        worst_rhclk / 1_000_000,
        MAX_RHCLK_HZ / 1_000_000
    );

    // The defect, named: at 80 MHz with both dividers undivided, the two buses are over their ceilings.
    println!("\nthe defect H10 left in place: 80 MHz undivided is 80 MHz on PCLK and 80 MHz to the radio");
    if 80_000_000 <= MAX_PCLK_HZ {
        println!("FAIL: the ceiling is not what this program assumes");
        failures += 1;
    }

    if failures == 0 {
        println!("\nPASS: every band has a plan, and no plan breaks a ceiling");
    } else {
        std::process::exit(1);
    }
}
