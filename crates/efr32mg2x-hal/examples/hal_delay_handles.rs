//! Two delay handles over one SysTick, with the counter's own configuration as the witness.
//!
//! ## The defect this measures
//!
//! Until 2026-11-28 every `delay_us`/`delay_ms` programmed the SysTick: set the reload, clear the value, enable the
//! counter, wait, disable it. Two `Delay` values -- from two threads, or from a handler that delays -- therefore reset
//! each other's countdown and switched the counter off under each other, and one of them returned early. An early
//! delay looks exactly like a correct one, which is why the claim survived in a SAFETY comment.
//!
//! ## Record, 2026-11-28 (two boots)
//!
//! ```text
//! delays: rvr=19000 want=19000 d1us=10040 d2us=10009      PASS
//! delays: rvr=19000 want=19000 d1us=10009 d2us=10040      PASS
//! ```
//!
//! 19 MHz is the clock in force, so `want=19000` is one millisecond; the hardware's reload is that same number after
//! two delays, which is what "the tick is configured once" means and what the old code could not produce. The two
//! delays land within 40 us of each other and of the 10 ms asked for, with both handles alive at once.
//!
//! What this does not say: that two *threads* cannot interfere. The board is single-threaded here; the fix's claim is
//! about the mechanism (nobody reprograms the counter), and this record shows the mechanism in place, not a race
//! reproduced. A race that needs preemption is measured with preemption or not at all.
//!
//! ## What is measured
//!
//! Since the fix the tick is configured **once**, as a free-running one millisecond counter, and a delay only observes
//! it. Three numbers follow, and each can be wrong:
//!
//! * `rvr` -- the reload the hardware is actually using, read back after two delays. It must be the one-millisecond
//!   reload for the clock in force (`hz / 1000`). The old code left whatever tick count the last call needed, so this
//!   single number tells the two versions apart;
//! * `d1us`, `d2us` -- ten milliseconds on each handle, timed against the SYSRTC as an independent witness, with both
//!   handles alive at once;
//! * ten milliseconds is 327.68 SYSRTC ticks, so the window is 9.5 to 10.8 ms: wide enough for the witness's own
//!   granularity, narrow enough to catch a delay that came back early.

#![no_std]
#![no_main]

use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::record::Record;
use embedded_hal::delay::DelayNs;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    mailbox::invalidate();
    let board = match Board::take() {
        Some(board) => board,
        None => halt(),
    };

    // Two handles over the same counter: the case the old code broke.
    let mut first = board.delay();
    let mut second = board.delay();

    let sysrtc = match board.sysrtc() {
        Ok(sysrtc) => sysrtc,
        Err(_) => {
            mailbox::fail(b"delays: sysystc refused");
            halt();
        }
    };

    let t0 = sysrtc.now();
    first.delay_ms(10);
    let t1 = sysrtc.now();
    second.delay_ms(10);
    let t2 = sysrtc.now();

    let d1 = ticks_to_us(t1.wrapping_sub(t0));
    let d2 = ticks_to_us(t2.wrapping_sub(t1));

    // The hardware's own configuration, read back: this is the number that distinguishes "observes the tick" from
    // "programs it".
    let rvr = cortex_m::peripheral::SYST::get_reload();
    let want = board.clock_hz().0 / 1000;

    let mut l = Record::<128>::new();
    l.s(b"delays: rvr=");
    l.u(rvr);
    l.s(b" want=");
    l.u(want);
    l.s(b" d1us=");
    l.u(d1);
    l.s(b" d2us=");
    l.u(d2);

    let in_window = |us: u32| (9_500..=10_800).contains(&us);
    if rvr == want && in_window(d1) && in_window(d2) {
        mailbox::pass(l.bytes());
    } else {
        mailbox::fail(l.bytes());
    }

    halt()
}

/// The SYSRTC counts at 32768 Hz.
fn ticks_to_us(ticks: u32) -> u32 {
    ((ticks as u64) * 1_000_000 / 32_768) as u32
}

fn halt() -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}
