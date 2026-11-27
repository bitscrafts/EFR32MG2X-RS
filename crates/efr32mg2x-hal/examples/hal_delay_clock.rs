//! The delay, measured against a clock that is not the delay.
//!
//! ## Why this instrument exists
//!
//! `delay_ms(10)` was once measured at 4.35 ms by `emlib_clock`, and every conclusion since then was an inference:
//! "SysTick must be counting at about 8.27 MHz", "the core is not in doubt because emlib says 19 MHz", "an attempted
//! fix made it 3.0x short". None of those timed a SysTick period or a delay with a clock outside the delay. This
//! example does both, with **TIMER0 counting the 39 MHz crystal** -- the only clock on the board that is traceable.
//!
//! The first version of this instrument reported every duration as zero because it converted counts to milliseconds
//! and labelled them microseconds: an instrument fault that looks exactly like a delay returning immediately. It is
//! recorded here because it is the failure the record has to survive -- every duration is printed beside a second,
//! independent statement of the same interval, and the control (`x`) fails loudly if the conversion is not being read.
//!
//! ## What each field means
//!
//! - `hz0` -- the core frequency the HAL declares on the part's reset clock, before the crystal is selected;
//!   `d0`, `u0` -- microseconds that `delay_ms(200)` and `delay_us(100_000)` actually took in that state.
//! - `hz1`, `d1`, `u1` -- the same with the core moved to the crystal, where the declared 39 000 000 is true by
//!   construction.
//! - `pw` -- microseconds of one SysTick period of 390 000 ticks, armed the way the delay arms it (`set_reload`,
//!   `clear_current`, wait for the wrap). `r1` is the rate that implies, in hertz: the honest answer to "what does
//!   SysTick count at here".
//! - `em` -- milliseconds the delay's own algorithm takes for `delay_ms(200)` when re-run here, so that "the code path"
//!   and "the primitive" are measured by one instrument in one run.
//! - `e1`, `eu` -- errors per mille of `d1` against 200 ms and of `u1` against 100 ms.
//! - `x`, `ex` -- the control: the same `delay_ms(200)` on a `Delay` told the core runs twice as fast. It asks for twice
//!   the ticks, so it must take 400 ms; a conversion that ignored the declared rate would return 200 ms here too.
//!
//! The verdict is the crystal state only: `d1` and `u1` within 20 per mille, and `x` within 20 per mille of 400 ms.
//!
//! VERIFIED ON HARDWARE (2026-11-27): record and what each number ruled out in
//! `docs/04-reference/DELAY-MEASURED-2026-11-27.md`.

#![no_std]
#![no_main]

use cortex_m::peripheral::SYST;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::clock::sources::{self, Em01Source};
use efr32mg2x_hal::delay::Delay;
use efr32mg2x_hal::entry;
use efr32mg2x_hal::record::Record;
use embedded_hal::delay::DelayNs;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

/// The crystal on the XIAO MG24.
const HFXO_HZ: u32 = 39_000_000;
/// `HFRCOEM23` measured on this part, the referee while the core is still on its reset clock. Not the core clock and
/// not claimed to be: a separate oscillator whose rate this project has measured nine times.
const HFRCOEM23_HZ: u32 = 19_173_200;
/// TIMER0's prescaler is 2^10, so one count is 26.3 us of the crystal and a full wrap is 1.72 s: a 200 ms delay fits
/// in one reading with no wrap to count.
const DIVIDE_LOG2: u8 = 10;
/// The free-running wrap.
const TOP: u16 = 0xFFFF;
/// A SysTick count worth measuring: 10 ms of the crystal, far above the instrument's 26 us resolution.
const PROBE_TICKS: u32 = 390_000;
/// How far a measured duration may sit from the wanted one, per mille.
const PER_MILLE_MAX: u64 = 20;
/// A period that takes longer than this many counts (about 4 s) is reported instead of hanging the run.
const GUARD_COUNTS: u64 = 150_000;

/// The TIMER0 counter, narrowed to the 16 bits the register has.
fn now(board: &Board) -> u16 {
    board.timer0_counter() as u16
}

/// Counts between two readings, accumulating the wrap in the subtraction.
fn between(from: u16, to: u16) -> u64 {
    u64::from(to.wrapping_sub(from))
}

/// Microseconds that `counts` of TIMER0 mean when it is counting `reference_hz` through the 2^10 prescaler.
fn micros(counts: u64, reference_hz: u32) -> u64 {
    counts * (1u64 << DIVIDE_LOG2) * 1_000_000 / u64::from(reference_hz)
}

/// Error of `measured` against `wanted`, per mille of `wanted`.
fn per_mille(measured: u64, wanted: u64) -> u64 {
    measured.abs_diff(wanted) * 1_000 / wanted
}

/// The rate, in hertz, that `ticks` SysTick ticks taking `us` microseconds imply.
fn rate_hz(ticks: u32, us: u64) -> u32 {
    if us == 0 {
        return 0;
    }
    (u64::from(ticks) * 1_000_000 / us) as u32
}

/// Microseconds a blocking call took, measured by TIMER0 and nothing else.
fn timed_us<F: FnOnce()>(board: &Board, reference_hz: u32, body: F) -> u64 {
    let start = now(board);
    body();
    let end = now(board);
    micros(between(start, end), reference_hz)
}

/// One SysTick period armed exactly the way the delay arms it, timed by TIMER0.
///
/// `None` when the wrap did not arrive inside [`GUARD_COUNTS`], which a SysTick that is not clocked would do.
fn wrap_period_us(syst: &mut SYST, board: &Board, ticks: u32, reference_hz: u32) -> Option<u64> {
    syst.disable_counter();
    syst.set_reload(ticks);
    syst.clear_current();
    let start = now(board);
    syst.enable_counter();
    while !syst.has_wrapped() {
        if between(start, now(board)) > GUARD_COUNTS {
            syst.disable_counter();
            return None;
        }
    }
    let end = now(board);
    syst.disable_counter();
    Some(micros(between(start, end), reference_hz))
}

/// The delay's own algorithm for a millisecond interval, run again so "the code path" and "the primitive" are measured
/// by one instrument in one run.
fn algorithm_ms(syst: &mut SYST, sysclk_hz: u32, ms: u32, board: &Board, reference_hz: u32) -> u64 {
    const MAX_RVR: u32 = 0x00FF_FFFF;
    let mut remaining = (u64::from(ms) * u64::from(sysclk_hz) / 1_000) as u32;
    let start = now(board);
    while remaining > 0 {
        let current = remaining.min(MAX_RVR);
        syst.set_reload(current);
        syst.clear_current();
        syst.enable_counter();
        while !syst.has_wrapped() {}
        syst.disable_counter();
        remaining -= current;
    }
    let end = now(board);
    micros(between(start, end), reference_hz) / 1_000
}

#[entry]
fn main() -> ! {
    mailbox::invalidate();

    let board = match Board::take() {
        Some(board) => board,
        None => {
            mailbox::fail(b"delayclk: peripherals taken twice");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // The referee, before anything is measured: TIMER0 counting `HFRCOEM23` through 2^10.
    let _ = sources::em01_select(Em01Source::HfrcoEm23);
    let running = matches!(
        board.timer0_free_running_prescaled(TOP, DIVIDE_LOG2),
        Ok(true)
    );
    if !running {
        mailbox::fail(b"delayclk: TIMER0 did not move, so there is no clock to measure with");
        loop {
            cortex_m::asm::wfi();
        }
    }
    mailbox::info(b"delayclk: 1 referee counting");

    // ---- 1. the reset clock ---------------------------------------------------------------------------------------------
    let hz0 = board.clock_hz().0;
    let mut delay = board.delay();
    let d0 = timed_us(&board, HFRCOEM23_HZ, || delay.delay_ms(200));
    let u0 = timed_us(&board, HFRCOEM23_HZ, || delay.delay_us(100_000));
    mailbox::info(b"delayclk: 2 reset clock measured");

    // ---- 2. the crystal: the declared frequency is true by construction -------------------------------------------------
    let hfxo_ok = board.select_hfxo(HFXO_HZ).is_ok();
    let _ = sources::em01_select(Em01Source::Hfxo);
    let hz1 = board.clock_hz().0;
    let mut syst = delay.free();

    let pw = wrap_period_us(&mut syst, &board, PROBE_TICKS, HFXO_HZ).unwrap_or(u64::MAX);
    let r1 = rate_hz(PROBE_TICKS, pw);
    let em = algorithm_ms(&mut syst, hz1, 200, &board, HFXO_HZ);

    let mut delay = Delay::with_sysclk(syst, hz1);
    let d1 = timed_us(&board, HFXO_HZ, || delay.delay_ms(200));
    let u1 = timed_us(&board, HFXO_HZ, || delay.delay_us(100_000));
    mailbox::info(b"delayclk: 3 crystal measured");

    // ---- 3. control: a Delay told the wrong rate must ask for the wrong number of ticks -----------------------------------
    let mut doubled = Delay::with_sysclk(delay.free(), hz1.saturating_mul(2));
    let x = timed_us(&board, HFXO_HZ, || doubled.delay_ms(200));

    let e1 = per_mille(d1, 200_000);
    let eu = per_mille(u1, 100_000);
    let ex = per_mille(x, 400_000);
    let ok = hfxo_ok && e1 <= PER_MILLE_MAX && eu <= PER_MILLE_MAX && ex <= PER_MILLE_MAX;

    let mut r = Record::<200>::new();
    r.s(b"delayclk: ok=")
        .bit(ok)
        .s(b" hfxo=")
        .bit(hfxo_ok)
        .s(b" hz0=")
        .u(hz0)
        .s(b" d0=")
        .u(d0)
        .s(b" u0=")
        .u(u0)
        .s(b" hz1=")
        .u(hz1)
        .s(b" pw=")
        .u(pw)
        .s(b" r1=")
        .u(r1)
        .s(b" em=")
        .u(em)
        .s(b" d1=")
        .u(d1)
        .s(b" e1=")
        .u(e1)
        .s(b" u1=")
        .u(u1)
        .s(b" eu=")
        .u(eu)
        .s(b" x=")
        .u(x)
        .s(b" ex=")
        .u(ex);

    if ok {
        mailbox::pass(r.bytes());
    } else {
        mailbox::fail(r.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
