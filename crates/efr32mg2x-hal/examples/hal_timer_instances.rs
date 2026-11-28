//! Do the other four timer instances actually count? `TIMER1`–`TIMER4` were "compiled, not measured" for months, and
//! the reason was structural: `timer::TimerN`'s only constructor took a `FrozenClocks`, which the `Board`
//! keeps private behind the CMU it owns — so no `Board` hand-out could build one, and those four instances were reachable
//! from nowhere. `TimerN::new_at` (the shape `usart` and `i2c` had already adopted) closes that.
//!
//! ## Two defects this instrument found before it recorded anything
//!
//! The constructor was wrong in two ways that a single-timer instrument could never have shown, because it was never
//! called at all:
//!
//!   * it enabled the block *before* writing `CFG`, and `CFG` faults while the timer is enabled — the first run died
//!     with `Escalated BusFault (Precise data access error) at 0x4004c004`. Then, with the order fixed the other way,
//!     `TOP` turned out to fault with the block disabled (`... at 0x4004c01c`): `CFG` wants the timer stopped, `TOP`
//!     wants it running.
//!   * `clear_overflow()` cleared **`TIMER0`'s** flag from every instance, because the base address was hard-coded in
//!     the macro that generates all five drivers.
//!
//! ## What is measured, and what makes it mean something
//!
//! Each instance is built at 1000 Hz and the instrument counts **overflows**, not the raw counter: `CNT` advances at
//! the oscillator's rate and wraps at `TOP`, so reading it measures the clock while counting overflows measures the
//! *period the driver was asked for*. A 50 ms window taken from the `SYSRTC` — a different oscillator — must see about
//! 50 overflows.
//!
//! Two controls, because "it moved" is the easiest thing here to fake:
//!
//!   * `TIMER3` is stopped after the window and must not overflow again.
//!   * `TIMER4` is constructed and never started; it must never overflow at all.
//!
//! And a third: a second `board.timer1(...)` must be refused by name, because the claim table is what stands between
//! this crate and two owners of one counter.
//!
//! VERIFIED ON HARDWARE (2026-11-27): three flashes, three boots, `status=PASS` in all three, byte-identical.
//!
//! ## What this does not prove
//!
//! Interrupt delivery (`listen_overflow` and a vector), and `TIMER0` itself — that instance has its own records

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use efr32mg2x_hal as _;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::record::Record;
use efr32mg2x_hal::timer::Config;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// 1000 Hz: one overflow per millisecond, so a 50 ms window must see about fifty.
const FREQUENCY_HZ: u32 = 1000;

/// The window, in SYSRTC ticks: 50 ms at 32.768 kHz.
const WINDOW_TICKS: u32 = 50 * 32;

/// Overflows of slack on a count of 50: the window is taken from a different oscillator and the flag is polled, not
/// interrupted, so the edges of the window are soft. Three in fifty is far tighter than a stopped counter (0) or a
/// prescaler that is off by two (100).
const SLACK: u32 = 3;

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
            mailbox::fail(b"timers: no SYSRTC to measure with");
            halt();
        }
    };
    let config = Config {
        frequency: FREQUENCY_HZ,
        pwm_mode: None,
    };

    let mut t1 = match board.timer1(config) {
        Ok(t) => t,
        Err(_) => {
            mailbox::fail(b"timers: TIMER1 not handed out");
            halt();
        }
    };
    let mut t2 = match board.timer2(config) {
        Ok(t) => t,
        Err(_) => {
            mailbox::fail(b"timers: TIMER2 not handed out");
            halt();
        }
    };
    let mut t3 = match board.timer3(config) {
        Ok(t) => t,
        Err(_) => {
            mailbox::fail(b"timers: TIMER3 not handed out");
            halt();
        }
    };
    let mut t4 = match board.timer4(config) {
        Ok(t) => t,
        Err(_) => {
            mailbox::fail(b"timers: TIMER4 not handed out");
            halt();
        }
    };

    // The claim table's control: a second ask for TIMER1 must be refused, or two owners would pace one counter.
    let refused = board.timer1(config).is_err();

    // A flag left over from a previous run would be counted as a first overflow, so every instance starts clean. This is
    // also the check that `clear_overflow` reaches the instance it was called on rather than TIMER0.
    t1.clear_overflow();
    t2.clear_overflow();
    t3.clear_overflow();
    t4.clear_overflow();
    let clean = !t1.is_overflow() && !t2.is_overflow() && !t4.is_overflow();

    let top = t1.get_top_value();
    let want = t1.get_frequency();
    // What the handle says it really produces, and how far that is from the request. These two travel with the handle
    // precisely so this instrument can compare them against the count it is about to observe.
    let actual = t1.get_actual_frequency();
    let ppm = t1.get_frequency_error_ppm();

    t1.start();
    t2.start();
    t3.start();
    // t4 is deliberately never started.

    let start = rtc.now();
    let (mut o1, mut o2, mut o3, mut o4) = (0u32, 0u32, 0u32, 0u32);
    loop {
        if t1.is_overflow() {
            t1.clear_overflow();
            o1 += 1;
        }
        if t2.is_overflow() {
            t2.clear_overflow();
            o2 += 1;
        }
        if t3.is_overflow() {
            t3.clear_overflow();
            o3 += 1;
        }
        if t4.is_overflow() {
            t4.clear_overflow();
            o4 += 1;
        }
        if rtc.now().wrapping_sub(start) >= WINDOW_TICKS {
            break;
        }
    }
    let elapsed = rtc.now().wrapping_sub(start);

    // Control one: a stopped instance must not overflow again. The window here is a fifth of the first, which at 1 kHz
    // would still be ten overflows if the stop had not taken.
    t3.stop();
    t3.clear_overflow();
    let stop_start = rtc.now();
    let mut o3_stopped = 0u32;
    while rtc.now().wrapping_sub(stop_start) < WINDOW_TICKS / 5 {
        if t3.is_overflow() {
            t3.clear_overflow();
            o3_stopped += 1;
        }
    }

    // SYSRTC ticks -> milliseconds at 32.768 kHz.
    let expected = elapsed / 32;

    let mut l = Record::<280>::new();
    l.s(b"timers: ms=");
    l.u(expected);
    l.s(b" o1=");
    l.u(o1);
    l.s(b" o2=");
    l.u(o2);
    l.s(b" o3=");
    l.u(o3);
    l.s(b" o4=");
    l.u(o4);
    l.s(b" stopped=");
    l.u(o3_stopped);
    l.s(b" top=");
    l.u(top);
    l.s(b" want=");
    l.u(want);
    l.s(b" actual=");
    l.u(actual);
    l.s(b" ppm=");
    match ppm {
        Some(v) => l.u(v),
        None => l.s(b"none"),
    };
    // The rate the polled witness saw. This reads LOW, and the reason is the instrument, not the timer: at 1 kHz the
    // loop clears four flags and reads the SYSRTC between polls, which takes about a millisecond, so an overflow now
    // and then is set and cleared before anyone looks. Measured 2026-11-28: 48 of 50 counts in a 49 ms window (4%),
    // and 487 of 500 in a 488 ms one (2.6%) -- the loss is a per-poll event, not a proportional one. The counter was
    // tried instead and cannot help: it wraps once per period, 500 times in that window, so two reads of it say
    // nothing (`counts=235`).
    //
    // So `measured` is a cross-check that the configuration took effect, to within the loop's own loss; the claim about
    // the rate itself is `actual`/`ppm`, which is arithmetic on `(prescaler, TOP, clock)` and can be checked by hand.
    let measured = if expected == 0 {
        0
    } else {
        o1.saturating_mul(1_000) / expected
    };
    l.s(b" measured=");
    l.u(measured);
    // Within 5%: the measured bias above is 4% in this window on this hardware.
    let close = measured.abs_diff(actual) * 20 <= actual.max(1);
    l.s(b" clean=");
    l.bit(clean);
    l.s(b" refused=");
    l.bit(refused);

    // Every field is part of the verdict: the two free-running instances must match the window and each other, the
    // third must have counted while running and stopped when asked, the fourth must never have counted at all, the
    // registers must hold what was asked, the second hand-out must have been refused -- and, since 2026-11-28, the
    // handle's claim about the rate it produces must agree with the rate the counter observed.
    let passed = refused
        && clean
        && want == FREQUENCY_HZ
        && top > 0
        && o1.abs_diff(expected) <= SLACK
        && o2.abs_diff(o1) <= SLACK
        && o3 > 0
        && o4 == 0
        && o3_stopped == 0
        && close;
    if passed {
        mailbox::pass(l.bytes());
    } else {
        mailbox::fail(l.bytes());
    }

    halt()
}

fn halt() -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}
