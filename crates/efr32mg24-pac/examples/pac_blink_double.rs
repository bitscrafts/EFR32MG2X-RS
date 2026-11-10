//! PAC-only DOUBLE blink: `blink blink ______ blink blink ______ ...`
//!
//! Two short pulses per group, then one long gap with the LED off. The pulses
//! are twice as fast as the baseline (`pac_blink_base` waits 1_000_000 NOP
//! iterations per phase; here it is 500_000), and the gap is 2_000_000, so the
//! eye reads two quick blinks followed by a pause.
//!
//! On a NOP-loop delay the absolute milliseconds depend on the clock the
//! bootloader left running, which is why this shape is verified by counting
//! toggles over SWD rather than by claiming a frequency:
//!
//! ```bash
//! scripts/measure-blink-rate.sh 8
//! ```
//!
//! The counter increments on every toggle and lives in the mailbox, so
//! (seq_after - seq_before) / elapsed gives the toggle rate directly.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

#[path = "common/blink.rs"]
mod blink;

/// Two pulses per group: blink, blink.
const PULSES: u32 = 2;
/// Half the baseline phase: twice the toggle rate.
const PULSE_ITERS: u32 = 500_000;
/// The pause between groups (LED off).
const GAP_ITERS: u32 = 2_000_000;

#[entry]
fn main() -> ! {
    blink::run_pattern(PULSES, PULSE_ITERS, GAP_ITERS)
}
