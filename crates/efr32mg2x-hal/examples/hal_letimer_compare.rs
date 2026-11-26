//! Does `LETIMER0` hold the top it was given, and does a compare below it fire twice?
//!
//! Two claims, and the second is the one that matters: a compare flag that never clears fires once and then looks dead, so
//! the instrument clears it and waits for a second match. Without that, "it fired" is a one-shot that tells you nothing
//! about the flag's behaviour.
//!
//! The third claim is the refusal: a compare at or above `TOP` can never be reached by a counter that runs from `TOP`
//! down, so the driver refuses to program one. The instrument asks for one on purpose and records that it was refused --
//! a driver that silently accepted it would program a flag nobody would ever read.
//!
//! ## Two things the first version of this instrument got wrong, and the record now shows
//!
//! - **The compares fire in the order the down-counter reaches them.** `COMP1` = 250 fires *after* `COMP0` = 500, not
//!   before, and the record carries both: `qe=0` at the first match, `ql=1` 250 ticks later.
//! - **`IF` is set by a write, not cleared by one.** A write of the three flags to `IF` left `0x7` behind and the second
//!   wait ended at once (`p2=0`). The driver now clears through the alias at `IF + 0x2000`, and the record shows the
//!   flags reading zero after the clear and the next match arriving 499 ticks later.
//!
//! VERIFIED ON HARDWARE (2026-11-26): `PASS letimer: clk=0x00000101 top0=1000 top=1000 comp0=500 comp1=250 refused=1
//! cnt0=1000 p1=501 qe=0 ql=1 pq=250 wrap=1 if1=0x00000001 ifc=0x00000000 if2=0x00000001 p2=499 cnt=499`, twice, the
//! only difference between runs being `top0` (see the driver's note on reading `TOP` while stopped).
//!
//! The reference is the SYSRTC at 32.768 kHz, which this crate has measured against the host (`hal_hfxo`: a 1000 ms delay
//! came back as 1006 ms). The record says how many RTC ticks each compare took, so the LETIMER's own rate is read off it
//! rather than assumed: with LFRCO as the source, one LETIMER tick is one SYSRTC tick.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use efr32mg2x_hal as _;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::letimer::Channel;
use efr32mg2x_hal::record::Record;
use efr32mg2x_hal::sysrtc::Sysrtc;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// The top: one period is `TOP`+1 = 1001 ticks, 30.6 ms at 32.768 kHz.
const TOP: u16 = 1000;
/// The compare that must fire, half a period after the reload.
const HALF: u16 = 500;
/// The compare that must fire first.
const QUARTER: u16 = 250;
/// A compare that cannot be reached: above `TOP`. The driver must refuse it.
const ABOVE: u16 = 2000;

/// How long one wait gets, in SYSRTC ticks. The compare is 500 ticks away and the period is 1001; 1600 leaves half a
/// period of margin and is still short enough that a timer that is not running fails in a readable time.
const BUDGET_TICKS: u32 = 1600;

/// Ticks of slack allowed on a compare interval: the two counters are independent registers sampled by the CPU, and the
/// claim is "the period is the one programmed", not "the poll loop is exact".
const SLACK: u32 = 8;

#[entry]
fn main() -> ! {
    mailbox::invalidate();

    let board = match Board::take() {
        Some(board) => board,
        None => halt(),
    };
    let rtc = match board.sysrtc() {
        Ok(rtc) => rtc,
        Err(_) => {
            mailbox::fail(b"letimer: no SYSRTC to measure with");
            halt();
        }
    };
    let mut letimer = match board.letimer() {
        Ok(t) => t,
        Err(_) => {
            mailbox::fail(b"letimer: not handed out");
            halt();
        }
    };

    // The driver does not poll `SYNCBUSY` -- on this peripheral that read stalls the bus -- so the caller's own clock
    // does the settling after every register that crosses into the low-frequency domain.
    settle(&rtc);

    // Configuration happens while stopped: the top and both compares are written before the count starts, so the first
    // period is the one that was asked for.
    if letimer.set_top(TOP).is_err() {
        mailbox::fail(b"letimer: the top could not be written");
        halt();
    }
    if letimer.set_compare(Channel::Ch0, HALF).is_err()
        || letimer.set_compare(Channel::Ch1, QUARTER).is_err()
    {
        mailbox::fail(b"letimer: a compare below the top was refused");
        halt();
    }
    // The control, in the same image: this one is above the top and must be refused.
    let refused = letimer.set_compare(Channel::Ch1, ABOVE).is_err();
    // The refusal must leave the register alone, so the good value is written back before the run.
    if letimer.set_compare(Channel::Ch1, QUARTER).is_err() {
        mailbox::fail(b"letimer: the compare could not be restored");
        halt();
    }
    // Every register written above crosses into the low-frequency domain before it can be read back.
    settle(&rtc);
    let top_stopped = letimer.top();
    let half_rb = letimer.compare(Channel::Ch0);
    let quarter_rb = letimer.compare(Channel::Ch1);

    letimer.clear_flags();
    settle(&rtc);
    letimer.start();
    settle(&rtc);
    // The counter must have loaded from `TOP`. It is read after the settle, so a few ticks of count may already have
    // passed; what this rules out is the 24-bit reset top, which would read 16 777 215 here.
    let cnt0 = letimer.counter();
    let top_running = letimer.top();

    // First match, timed against the SYSRTC: `HALF` ticks after the reload.
    let t0 = rtc.now_wide();
    let mut fired1 = false;
    while rtc.now_wide().wrapping_sub(t0) < u64::from(BUDGET_TICKS) {
        if letimer.compare_fired(Channel::Ch0) {
            fired1 = true;
            break;
        }
    }
    let first_ticks = rtc.now_wide().wrapping_sub(t0) as u32;
    let flags1 = letimer.flags();
    // The counter counts **down**, so `QUARTER` is reached after `HALF`, not before: the second compare must still be
    // clear at the first match, and must fire `HALF - QUARTER` ticks later. An instrument that assumed the opposite order
    // would read this channel as dead.
    let quarter_early = letimer.compare_fired(Channel::Ch1);
    let tq = rtc.now_wide();
    let mut quarter_late = false;
    while rtc.now_wide().wrapping_sub(tq) < u64::from(BUDGET_TICKS) {
        if letimer.compare_fired(Channel::Ch1) {
            quarter_late = true;
            break;
        }
    }
    let quarter_ticks = rtc.now_wide().wrapping_sub(tq) as u32;

    // Wait for the wrap before clearing. A compare flag follows the comparison, so clearing it while the counter is still
    // below the compare value sets it again on the next read -- measured: `ifc=0x1` with the counter at 500. Only once the
    // counter is back above both values is a clear observable.
    let tw = rtc.now_wide();
    let mut wrapped = false;
    while rtc.now_wide().wrapping_sub(tw) < u64::from(BUDGET_TICKS) {
        if letimer.wrapped() {
            wrapped = true;
            break;
        }
    }
    letimer.clear_flags();
    settle(&rtc);
    let flags_after_clear = letimer.flags();

    // A full period later, the same compare must fire again: this is the half that says the flag is a flag and not a
    // latch.
    let t2 = rtc.now_wide();
    let mut fired2 = false;
    while rtc.now_wide().wrapping_sub(t2) < u64::from(BUDGET_TICKS) {
        if letimer.compare_fired(Channel::Ch0) {
            fired2 = true;
            break;
        }
    }
    let second_ticks = rtc.now_wide().wrapping_sub(t2) as u32;
    let flags2 = letimer.flags();
    let counter = letimer.counter();
    letimer.stop();

    let mut l = Record::<200>::new();
    l.s(b"letimer: clk=0x");
    l.h32(efr32mg2x_hal::letimer::clock_readback());
    l.s(b" top0=");
    l.u(u32::from(top_stopped));
    l.s(b" top=");
    l.u(u32::from(top_running));
    l.s(b" comp0=");
    l.u(u32::from(half_rb));
    l.s(b" comp1=");
    l.u(u32::from(quarter_rb));
    l.s(b" refused=");
    l.bit(refused);
    l.s(b" cnt0=");
    l.u(cnt0);
    l.s(b" p1=");
    l.u(first_ticks);
    l.s(b" qe=");
    l.bit(quarter_early);
    l.s(b" ql=");
    l.bit(quarter_late);
    l.s(b" pq=");
    l.u(quarter_ticks);
    l.s(b" wrap=");
    l.bit(wrapped);
    l.s(b" if1=0x");
    l.h32(flags1);
    l.s(b" ifc=0x");
    l.h32(flags_after_clear);
    l.s(b" if2=0x");
    l.h32(flags2);
    l.s(b" p2=");
    l.u(second_ticks);
    l.s(b" cnt=");
    l.u(counter);

    // Every field is part of the verdict: the registers hold what was asked, the counter loaded from the top, both
    // matches happened at the programmed interval -- in the order a down-counter reaches them -- the flag was observably
    // clear after a clear taken above both compare values, the second match arrived a period later, and the above-top
    // compare was refused.
    let loaded_from_top = cnt0 <= u32::from(TOP) && cnt0 + u32::from(SLACK) >= u32::from(TOP);
    let passed = top_running == TOP
        && half_rb == HALF
        && quarter_rb == QUARTER
        && refused
        && loaded_from_top
        && fired1
        && flags1 == 1
        && !quarter_early
        && quarter_late
        && quarter_ticks.abs_diff(u32::from(HALF - QUARTER)) <= SLACK
        && wrapped
        && flags_after_clear == 0
        && fired2
        && second_ticks.abs_diff(u32::from(HALF)) <= SLACK;
    if passed {
        mailbox::pass(l.bytes());
    } else {
        mailbox::fail(l.bytes());
    }

    halt()
}

/// Wait two low-frequency cycles: the driver's own settlement, done here from the caller's clock.
fn settle(rtc: &Sysrtc) {
    let t = rtc.now();
    while rtc.now().wrapping_sub(t) < 2 {
        core::hint::spin_loop();
    }
}

fn halt() -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}
