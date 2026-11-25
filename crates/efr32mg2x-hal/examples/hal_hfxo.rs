//! HFXO: run from the crystal, and prove the rest of the system followed it.
//!
//! ## What this measures
//!
//! Not "the crystal started" — `select_hfxo` returns that. It is that **the rest of the system knows it started**:
//! after the core moves to 39 MHz, a `delay_ms(1000)` has to last one second by an independent clock.
//!
//! The referee is the SYSRTC (32.768 kHz, itself measured against the host's clock), and the check is the one that
//! distinguishes the two failures: with `SysTick` still calibrated for the 19 MHz of reset, 1000 requested ms would
//! take about 2050 ms, i.e. ~67 000 ticks instead of 32 768; with the crystal not starting at all, `select_hfxo`
//! returns an error and the frequency never moves.
//!
//! This example used to have no record a harness could read: it printed the marks over RTT with host timestamps, so
//! its evidence required an attached session and it was filed as "undated". The four numbers below are in the mailbox,
//! which is what lets it carry a date.
//!
//! ## What happens if the crystal does not start
//!
//! `ClockError::HfxoTimeout`, and the `Board` keeps the clock it had. The example says so and continues — a board that
//! goes silent is indistinguishable from a broken one.
//!
//! VERIFIED ON HARDWARE (2026-11-25): the record is in this file's header and in the crate README.

#![no_std]
#![no_main]

use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::prelude::*;
use efr32mg2x_hal::record::Record;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

/// The crystal on the XIAO MG24.
const HFXO_HZ: u32 = 39_000_000;
/// The tick clock, used only to turn ticks into milliseconds.
const TICK_HZ: u32 = 32_768;
/// One second, the interval the delay is asked for.
const REQUESTED_MS: u32 = 1000;
/// Two percent of the interval. Wider than the oscillators and narrower than a delay calibrated for the wrong clock,
/// which is off by 105 %.
const TOLERANCE_TICKS: u32 = TICK_HZ / 50;

#[cortex_m_rt::entry]
fn main() -> ! {
    mailbox::invalidate();
    let board = Board::take().expect("peripherals taken twice");
    let before = board.clock_hz().0;

    // The referee is set up before the clock changes, so a failure to reach the crystal cannot also cost the
    // instrument.
    let rtc = board.sysrtc().expect("SYSRTC is handed out once");

    let selected = board.select_hfxo(HFXO_HZ).is_ok();
    let after = board.clock_hz().0;

    // Built AFTER the clock change on purpose: that is what proves the delay calibrates from the clock in force and
    // not from the reset one.
    let mut delay = board.delay();

    let t0 = rtc.now();
    delay.delay_ms(REQUESTED_MS);
    let t1 = rtc.now();

    let elapsed = t1.wrapping_sub(t0);
    let expected = REQUESTED_MS * TICK_HZ / 1000;
    let error = elapsed as i32 - expected as i32;

    let mut l = Record::<120>::new();
    l.s(b"hfxo: sel=");
    l.u(selected as u32);
    l.s(b" before=");
    l.u(before / 1_000_000);
    l.s(b"M after=");
    l.u(after / 1_000_000);
    l.s(b"M tick=");
    l.u(elapsed);
    l.s(b" err=");
    l.i(error);

    let ok = selected && after == HFXO_HZ && error.unsigned_abs() <= TOLERANCE_TICKS;
    if ok {
        mailbox::pass(l.bytes());
    } else {
        mailbox::fail(l.bytes());
    }

    loop {
        core::hint::spin_loop();
    }
}
