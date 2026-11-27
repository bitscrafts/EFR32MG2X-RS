//! The clocks nobody had measured: `FSRCO`, `HFRCOEM23`, `ULFRCO` -- and the crystal as the referee.
//!
//! ## Why a new instrument and not another comment
//!
//! Three numbers for `HFRCOEM23` were in circulation (18.5 MHz from a blink, 18.961 MHz from the tick timer, "19.6 MHz"
//! in a register note) and none of them came from a run that could be repeated: two were derived from a delay whose own
//! calibration was in question, and the datasheet only promises a *band*. `FSRCO` and `ULFRCO` had no measurement at all.
//!
//! ## The referee, and why it cannot be the SYSRTC
//!
//! `LFRCO` is a 3 %-accurate RC oscillator (datasheet Table 4.29); using it to judge another oscillator's percent would
//! be judging one ruler with another ruler of the same kind. The one clock on the board that is traceable is the
//! **39 MHz crystal**, and the instrument is built so every number is a ratio to it:
//!
//! - `EM01GRPACLK` members are counted by **TIMER0**, free-running at 65536-count wraps. The same timer is pointed at
//!   the crystal before and after each source, so `f = 39 MHz x counts(source) / counts(crystal)`. The window itself --
//!   a count of SYSRTC ticks -- cancels, which is why a 3 %-accurate clock is allowed to set the duration.
//! - `ULFRCO` cannot clock `EM01GRPACLK` (the group has no entry for it), so it clocks `BURTC` through
//!   `EM4GRPACLK`, and TIMER0 on the crystal counts the window: `f = ticks(BURTC) x 39 MHz / counts(TIMER0)`. Here the
//!   window is the crystal itself, so nothing cancels and nothing needs to.
//!
//! ## What is checked
//!
//! 1. **The instrument, on the reference.** `EM01GRPACLK` = `HFXO`, counted twice with the others in between. The two
//!    counts must agree to 5 per mille; anything worse means the window drifted and the ratios between them are noise.
//! 2. **`HFRCOEM23`** inside the datasheet limits of the band `CAL.FREQRANGE` reports (Table 4.27 note 6).
//! 3. **`FSRCO`** inside 17.2..21.2 MHz (Table 4.28).
//! 4. **`ULFRCO`** inside 0.944..1.095 kHz (Table 4.30), counted by BURTC.
//! 5. **`LFRCO`** inside +-3 % of 32.768 kHz (Table 4.29), the same BURTC one selector earlier -- 32x away from
//!    `ULFRCO`, so a selector that was written and ignored cannot pass both.
//! 6. **Control: source removed.** With `EM4GRPACLK` pointed at nothing, BURTC must stand still (`ul0 = 0`) while the
//!    crystal keeps counting; then the source comes back and it counts again (`ur > 0`). Without this, "the counter
//!    advanced" does not distinguish a clocked counter from a register that keeps its old value.
//!
//! VERIFIED ON HARDWARE (2026-11-27): the record is in `docs/04-reference/CMU-MISSING-CLOCKS-2026-11-27.md`.

#![no_std]
#![no_main]

use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::burtc;
use efr32mg2x_hal::clock::sources::{self, Em01Source};
use efr32mg2x_hal::entry;
use efr32mg2x_hal::record::Record;
use efr32mg2x_hal::sysrtc::Sysrtc;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

/// The crystal on the XIAO MG24, and the only clock here that is traceable.
const HFXO_HZ: u32 = 39_000_000;
/// The window, in SYSRTC ticks of the nominal 32.768 kHz: about one second. It sets the duration only -- every number
/// below is a ratio of two counts taken over it, so its own frequency does not enter.
const WINDOW_TICKS: u32 = 32_768;
/// How far the two crystal counts may differ, per mille, before the ratios between them are noise.
const DRIFT_PER_MILLE_MAX: u64 = 5;
/// How far the LFRCO rate the window implies may sit from the LFRCO rate BURTC measured, per mille.
const WINDOW_AGREEMENT_PER_MILLE_MAX: u64 = 10;
/// The counter's wrap: `TOP = 0xFFFF` makes TIMER0 a free-running 16-bit counter.
const TOP: u16 = 0xFFFF;
/// A loop bound that a stopped SYSRTC hits instead of hanging the run.
const GUARD: u32 = 20_000_000;

/// Counts of TIMER0 over `want` SYSRTC ticks, together with the ticks actually elapsed (0 when the SYSRTC stalled).
///
/// The deltas are added up rather than read once, because the counter wraps every 65536 counts: at 39 MHz that is every
/// 1.7 ms, far more often than the window. Each delta is far smaller than a wrap, so no wrap can be missed.
fn count_timer(board: &Board, rtc: &Sysrtc, want: u32) -> (u64, u32) {
    let s0 = rtc.now();
    let mut prev = board.timer0_counter() as u16;
    let mut counts: u64 = 0;
    let mut guard: u32 = 0;
    loop {
        let now = board.timer0_counter() as u16;
        counts += u64::from(now.wrapping_sub(prev));
        prev = now;
        guard = guard.wrapping_add(1);
        if guard == GUARD {
            return (counts, 0);
        }
        if rtc.now().wrapping_sub(s0) >= want {
            break;
        }
    }
    let s1 = rtc.now();
    (counts, s1.wrapping_sub(s0))
}

/// The same window, with `BURTC` as the counter under test and TIMER0 on the crystal as the clock that times it.
///
/// Returns `(burtc ticks, TIMER0 counts, SYSRTC ticks)`.
fn count_burtc(board: &Board, counter: &burtc::Burtc, rtc: &Sysrtc, want: u32) -> (u32, u64, u32) {
    let b0 = counter.counter();
    let s0 = rtc.now();
    let mut prev = board.timer0_counter() as u16;
    let mut counts: u64 = 0;
    let mut guard: u32 = 0;
    loop {
        let now = board.timer0_counter() as u16;
        counts += u64::from(now.wrapping_sub(prev));
        prev = now;
        guard = guard.wrapping_add(1);
        if guard == GUARD {
            return (0, 0, 0);
        }
        if rtc.now().wrapping_sub(s0) >= want {
            break;
        }
    }
    let s1 = rtc.now();
    let b1 = counter.counter();
    (b1.wrapping_sub(b0), counts, s1.wrapping_sub(s0))
}

/// `reference_hz x counts / reference_counts`, rounded down to a whole hertz. Zero when the reference did not move,
/// which no band below accepts.
fn hz_from(reference_hz: u64, counts: u64, reference_counts: u64) -> u32 {
    if reference_counts == 0 {
        return 0;
    }
    (reference_hz * counts / reference_counts) as u32
}

/// Per mille of difference between two counts of the same clock.
fn per_mille(a: u64, b: u64) -> u64 {
    let diff = a.abs_diff(b);
    if b == 0 {
        return u64::MAX;
    }
    diff * 1_000 / b
}

#[entry]
fn main() -> ! {
    mailbox::invalidate();

    let board = match Board::take() {
        Some(board) => board,
        None => {
            mailbox::fail(b"cmu: peripherals taken twice");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let rtc = match board.sysrtc() {
        Ok(rtc) => rtc,
        Err(_) => {
            mailbox::fail(b"cmu: SYSRTC already handed out, so the window is not available");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let counter = match board.burtc(0) {
        Ok(counter) => counter,
        Err(_) => {
            mailbox::fail(b"cmu: BURTC refused, so ULFRCO has no counter");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // Every number below is a ratio to this one, so it comes first and the run stops if it does not take.
    let hfxo_ok = board.select_hfxo(HFXO_HZ).is_ok();
    let clock_hz = board.clock_hz().0;

    let timer_ok = match board.timer0_free_running(TOP) {
        Ok(moved) => moved,
        Err(_) => false,
    };

    // ---- 1. the reference, measured by the instrument that measures everything else -----------------------------
    let hfxo_selected = sources::em01_select(Em01Source::Hfxo).is_ok();
    let (hx0, win0) = count_timer(&board, &rtc, WINDOW_TICKS);
    mailbox::info(b"cmu: 1 reference counted");

    // ---- 2. HFRCOEM23 ------------------------------------------------------------------------------------------
    let band = sources::hfrcoem23_band();
    let hb_selected = sources::em01_select(Em01Source::HfrcoEm23).is_ok();
    let (hb_counts, win_hb) = count_timer(&board, &rtc, WINDOW_TICKS);
    let hfrco_hz = hz_from(u64::from(HFXO_HZ), hb_counts, hx0);
    mailbox::info(b"cmu: 2 hfrcoem23 counted");

    // ---- 3. FSRCO ----------------------------------------------------------------------------------------------
    let fs_selected = sources::em01_select(Em01Source::Fsrco).is_ok();
    let (fs_counts, win_fs) = count_timer(&board, &rtc, WINDOW_TICKS);

    // ---- 4. back to the crystal: the bracket that says the window did not drift ----------------------------------
    let _ = sources::em01_select(Em01Source::Hfxo);
    let (hx1, win1) = count_timer(&board, &rtc, WINDOW_TICKS);
    let fsrco_hz = hz_from(u64::from(HFXO_HZ), fs_counts, hx1);
    let drift = per_mille(hx1, hx0);
    mailbox::info(b"cmu: 3 fsrco counted");

    // ---- 5. ULFRCO, counted by BURTC with the crystal counting the window ---------------------------------------
    // LFRCO first, on the same counter and the same window: it is the source BURTC has been measured on, it is 32x
    // away from ULFRCO, and the datasheet's +-3 % band is wide enough to accept the die's own trim.
    burtc::select_lfrco();
    let (lfrco_ticks, win_l, _) = count_burtc(&board, &counter, &rtc, WINDOW_TICKS);
    let lfrco_hz = hz_from(u64::from(HFXO_HZ), u64::from(lfrco_ticks), win_l);

    burtc::select_ulfrco();
    let (ulfrco_ticks, win_u, _) = count_burtc(&board, &counter, &rtc, WINDOW_TICKS);
    let ulfrco_hz = hz_from(u64::from(HFXO_HZ), u64::from(ulfrco_ticks), win_u);
    mailbox::info(b"cmu: 4 ulfrco counted");

    // The window's own rate, in hertz: 32768 SYSRTC ticks took `hx0` counts of the crystal. It must be the rate BURTC
    // measured on LFRCO -- two counters, two windows, one oscillator -- and it is what says the window really is the
    // LFRCO branch and not a stale count. A window that nobody can name would make every ratio above an assumption.
    let window_hz = hz_from(u64::from(HFXO_HZ), u64::from(WINDOW_TICKS), hx0);

    // ---- 6. control: the same counter with its source removed ----------------------------------------------------
    burtc::remove_source();
    let (stopped, win_stop, _) = count_burtc(&board, &counter, &rtc, WINDOW_TICKS);

    // ---- 7. the source comes back, so the control runs in both directions ---------------------------------------
    burtc::select_ulfrco();
    let (restored, _, _) = count_burtc(&board, &counter, &rtc, WINDOW_TICKS);

    let em01 = sources::em01_readback();
    let burtc_clocks = burtc::clock_readback();
    let hfrco_status = sources::hfrcoem23_status();
    let hfrco_cal = sources::hfrcoem23_cal();

    let (lo, hi) = band
        .ok()
        .and_then(sources::hfrco_band_limits)
        .unwrap_or((0, 0));
    let band_known = hi != 0;
    let band_ok = band_known && (lo..=hi).contains(&hfrco_hz);

    let fsrco_ok = (sources::FSRCO_MIN_HZ..=sources::FSRCO_MAX_HZ).contains(&fsrco_hz);
    let ulfrco_ok = (sources::ULFRCO_MIN_HZ..=sources::ULFRCO_MAX_HZ).contains(&ulfrco_hz);
    let lfrco_band = u32::from(sources::LFRCO_TOLERANCE_PER_CENT);
    let lfrco_min = sources::LFRCO_TYP_HZ - sources::LFRCO_TYP_HZ / 100 * lfrco_band;
    let lfrco_max = sources::LFRCO_TYP_HZ + sources::LFRCO_TYP_HZ / 100 * lfrco_band;
    let lfrco_ok = (lfrco_min..=lfrco_max).contains(&lfrco_hz);
    let window_ok = window_hz > 0
        && per_mille(u64::from(lfrco_hz), u64::from(window_hz)) <= WINDOW_AGREEMENT_PER_MILLE_MAX;

    let ok = hfxo_ok
        && clock_hz == HFXO_HZ
        && timer_ok
        && hfxo_selected
        && hb_selected
        && fs_selected
        && win0 > 0
        && win_hb > 0
        && win_fs > 0
        && win1 > 0
        && drift <= DRIFT_PER_MILLE_MAX
        && band_known
        && band_ok
        && fsrco_ok
        && lfrco_ok
        && window_ok
        && ulfrco_ok
        && stopped == 0
        && win_stop > 0
        && restored > 0;

    let mut record: Record<200> = Record::new();
    record
        .s(b"cmu: ok=")
        .u(ok as u32)
        .s(b" hx=")
        .u(hx0)
        .s(b" hx2=")
        .u(hx1)
        .s(b" hb=")
        .u(hfrco_hz)
        .s(b" bnd=")
        .u(band.unwrap_or(0xFF) as u32)
        .s(b" fs=")
        .u(fsrco_hz)
        .s(b" ul=")
        .u(ulfrco_hz)
        .s(b" bu=")
        .u(ulfrco_ticks)
        .s(b" tu=")
        .u(win_u)
        .s(b" lf=")
        .u(lfrco_hz)
        .s(b" bl=")
        .u(lfrco_ticks)
        .s(b" tl=")
        .u(win_l)
        .s(b" ul0=")
        .u(stopped)
        .s(b" ur=")
        .u(restored)
        .s(b" dr=")
        .u(drift)
        .s(b" em=")
        .u(u32::from(em01))
        .s(b" bc=")
        .u(burtc_clocks)
        .s(b" hs=0x")
        .h32(hfrco_status)
        .s(b" hc=0x")
        .h32(hfrco_cal);

    // A record that did not fit was cut, and a cut number reads as a different number: that is a failed run, not a
    // pass with a shorter line.
    if record.truncated() {
        mailbox::fail(
            b"cmu: the record did not fit in the mailbox, so no number in it is a measurement",
        );
    } else if ok {
        mailbox::pass(record.bytes());
    } else {
        mailbox::fail(record.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
