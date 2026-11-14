//! PAC-only blink, baseline: one ON/OFF pulse group, 1_000_000 NOP iterations
//! per phase, no gap -- i.e. the continuous blink the board has been running.
//!
//! Compare against `pac_blink_double`. Verified over SWD, not by eye:
//! `scripts/read-led.sh` (mode7=4, DOUT toggles, DIN follows) and
//! `scripts/measure-blink-rate.sh` (toggle counter read from the mailbox twice,
//! divided by the wall-clock gap).

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

#[path = "common/blink.rs"]
mod blink;

const PULSES: u32 = 1;
const PULSE_ITERS: u32 = 1_000_000;
const GAP_ITERS: u32 = 0;

#[entry]
fn main() -> ! {
    blink::run_pattern(PULSES, PULSE_ITERS, GAP_ITERS)
}
