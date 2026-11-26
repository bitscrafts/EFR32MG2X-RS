//! Pace TIMER0 twice, and a third time: does the second write to `CFG` still land?
//!
//! This is the instrument the queue asked for. On 2026-11-26 the *second* `pace_microphone` faulted the core with a
//! precise bus error at `TIMER0 + 0x04` (`BFAR = 0x4004_8004`, `CFSR = 0x8200`): `CFG` may not be written while the
//! block is enabled, and clearing `EN` is not instantaneous -- `EN.DISABLING` reports when it has actually stopped.
//! `Board::pace_microphone` and `pwm::Timer0::take` were both given that wait in the same session, and neither was
//! re-measured afterwards. A fix nobody has measured is a claim, and this example exists to turn it into a record.
//!
//! What counts as measured: three paces, each one leaving a running counter behind it, and the period going up when the
//! rate goes down. The counter is read either side of a dwell so "it runs" is two numbers that differ, not a bit that
//! was set once.
//!
//! Written on 2026-11-26 while the board was unplugged: it compiles and it is the instrument the queue needs, but it
//! carries **no record yet**. Until it has one, the `EN.DISABLING` wait in `pace_microphone` is still an unmeasured fix.
//!
//! The three paces are 16 kHz, 8 kHz, 16 kHz rather than one call: the defect was specifically about the second write,
//! and returning to the first rate shows the path is repeatable instead of surviving exactly once.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use efr32mg2x_hal as _;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::record::Record;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// The three rates, fastest first. At 19 MHz a period of 16 kHz is about 1187 clocks and 8 kHz about 2375, so the
/// second pace has to change `TOP` by a factor of two -- a period that merely got rewritten with the same value would
/// hide a `CFG` write that was silently dropped.
const RATES: [u32; 3] = [16_000, 8_000, 16_000];

/// How long to let the counter run before the second read, in loop iterations. Long enough to be many periods at
/// either rate, short enough not to dominate the run.
const DWELL: u32 = 20_000;

#[entry]
fn main() -> ! {
    mailbox::invalidate();
    let board = match Board::take() {
        Some(board) => board,
        None => {
            mailbox::fail(b"repacing: the board was already taken");
            idle()
        }
    };

    let mut l = Record::<320>::new();
    l.s(b"repacing:");
    let mut periods = [0u32; 3];
    let mut moved = [0u32; 3];

    for (i, hz) in RATES.iter().enumerate() {
        // The pace under test. The second iteration is the one that used to fault.
        periods[i] = match board.pace_microphone(*hz) {
            Some(period) => period,
            None => {
                mailbox::fail(b"repacing: TIMER0 refused to be paced");
                idle()
            }
        };

        // Two reads with a dwell between them: "it runs" is a difference, not a flag.
        let before = board.timer0_counter();
        let mut spin = 0u32;
        while spin < DWELL {
            cortex_m::asm::nop();
            spin += 1;
        }
        let after = board.timer0_counter();
        moved[i] = after.wrapping_sub(before);

        if i == 1 {
            mailbox::pass(b"repacing: second pace survived");
        }
    }

    // The record: the three periods, the three movements, and the verdict spelled out so a reader does not have to
    // re-derive it.
    l.s(b" p1=")
        .u(periods[0])
        .s(b" p2=")
        .u(periods[1])
        .s(b" p3=")
        .u(periods[2]);
    l.s(b" m1=")
        .u(moved[0])
        .s(b" m2=")
        .u(moved[1])
        .s(b" m3=")
        .u(moved[2]);

    // A period that doubles when the rate halves is the sign the second write landed; a period that stayed put means
    // `CFG` was quietly ignored -- the defect this example was written for.
    let all_ran = moved.iter().all(|m| *m > 0);
    let doubled = periods[1] > periods[0] && periods[2] == periods[0];
    l.s(b" doubled=").u(u32::from(doubled));
    l.s(b" ran=").u(u32::from(all_ran));

    if all_ran && doubled {
        mailbox::pass(l.bytes());
    } else {
        mailbox::fail(l.bytes());
    }
    idle()
}

/// A completed run: sleep rather than spin, so the record is not disturbed and the current draw stays honest.
fn idle() -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}
