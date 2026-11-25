//! Does the compare channel fire? Measured against the counter itself, with a wait long enough to matter.
//!
//! The earlier run reported `fired=0` and it was almost certainly the instrument: the deadline was 1600 ticks
//! (48.8 ms at the measured 32.968 kHz) and the bounded wait was a spin count whose duration nobody knew. So
//! this example schedules a deadline and waits on the *counter* rather than on a spin count: if the counter
//! passes the deadline, the flag must have been set, and the run says whether it was.
//!
//! No DWT anywhere: the counter is the clock. A separate absolute measurement (two reads from the host with a
//! stopwatch between them) established 32 968 Hz, so `ticks_per_ms = 32` is right to within 0.6%.
//!
//! VERIFIED ON HARDWARE (2026-11-24): the record is in this file's header and in the crate README's dated table.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::record::Record;
use efr32mg2x_hal::sysrtc::Sysrtc;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

#[entry]
fn main() -> ! {
    mailbox::invalidate();
    let board = match Board::take() {
        Some(b) => b,
        None => loop {
            core::hint::spin_loop();
        },
    };
    let rtc = board.sysrtc().expect("SYSRTC is handed out once");

    // The compare flag survives a reset in this part's block, so a stale one would be taken as "fired" before the counter
    // had moved at all. Measured: without this clear the record read `reached == start`, `fired=1` and `late = -3200`
    // (4294964096 unsigned) -- a pass that proved nothing. Discard whatever is pending before scheduling anything.
    while rtc.take_compare_event() {}

    // 100 ms is 3277 ticks at 32.768 kHz. The wait is bounded by the counter itself: once `now()` passes the
    // deadline, a flag that is still clear is a real failure, not an impatient timeout.
    let start = rtc.now();
    let deadline = start.wrapping_add(100 * Sysrtc::ticks_per_ms());
    rtc.schedule_at(deadline);

    let mut fired = false;
    let mut guard = 0u32;
    while guard < 20_000_000 {
        if rtc.take_compare_event() {
            fired = true;
            break;
        }
        // Wraparound-safe "has the counter passed the deadline": the difference is small when it has, and
        // huge (in the unsigned sense) when it has not. Comparing the raw subtraction to 1000 was wrong and
        // made the guard fire immediately with the counter still at zero.
        let now = rtc.now();
        let after = now.wrapping_sub(deadline) < 0x8000_0000;
        if after && now.wrapping_sub(deadline) > 1000 {
            break; // passed the deadline and the flag never set: a real failure, not impatience
        }
        guard = guard.wrapping_add(1);
    }
    let reached = rtc.now();

    let mut l = Record::<200>::new();
    l.s(b"cmp: start=");
    l.u(start);
    l.s(b" deadline=");
    l.u(deadline);
    l.s(b" reached=");
    l.u(reached);
    l.s(b" late=");
    l.u(reached.wrapping_sub(deadline));
    l.s(b" fired=");
    l.u(fired as u32);
    l.s(b" en=");
    l.u(rtc.is_enabled() as u32);
    l.s(b" run=");
    l.u(rtc.is_running() as u32);

    if fired {
        mailbox::pass(l.bytes());
    } else {
        mailbox::fail(l.bytes());
    }

    loop {
        core::hint::spin_loop();
    }
}
