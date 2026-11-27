//! `ACMP0` as a ruler: the internal ladder swept against a fixed internal source, read three times.
//!
//! ## What is being proved
//!
//! The comparator is the one analogue block whose answer is a single bit, so a driver that configures nothing and
//! reads `STATUS.ACMPOUT` still returns a plausible 0 or 1. The defect this example exists to catch is a ladder that is
//! written and not applied: the mux latches `VREFDIV`, the register reads back what was written, and the output never
//! moves. Every check below is about the sweep *turning*, not about the block answering.
//!
//! 1. **A full sweep against the 1.25 V tap.** `VREFDIV` walks 1..=63 with `VSENSE0/4` on the positive side, so the
//!    ladder rises in 63 steps from 19.8 mV to 1.25 V. The output must be one contiguous run of high and one of low, with
//!    exactly one edge. A ladder that is ignored gives zero edges (all high or all low); a ladder with a stuck bit gives
//!    more than one.
//! 2. **The same sweep against the 2.5 V tap.** The reference is twice as high, so the crossing must land at half the
//!    tap -- the number the record's `c1` and `c2` carry. This is the check that the crossing belongs to the ladder and
//!    not to a fixed comparator offset: an offset would not halve.
//! 3. **The two windows must intersect.** Each crossing brackets the measured voltage between two adjacent taps
//!    ([`efr32mg2x_hal::acmp::window_mv`]); two independent references that disagree produce an empty intersection.
//! 4. **The first sweep repeated, within one tap.** The crossing is quantised by the ladder (19.8 mV a tap against the
//!    1.25 V reference), so a signal sitting near a boundary lands on 43 in one sweep and 44 in the next; the record shows
//!    which of the two each sweep produced, and any repetition that moved further than one tap fails. The bracket quoted
//!    is the intersection of all three sweeps, which is stable across that quantisation.
//! 5. **Control: the inputs swapped at the pass-through tap.** With the same two sources and `VREFDIV = 63`, swapping
//!    them must invert the answer. Without this, "high at low taps" would also be satisfied by a comparator that always
//!    returns 1 while enabled.
//! 6. **Control: the block disabled.** With `EN = 0` the status bit follows `CTRL.NOTRDYVAL` (0 then 1) with the inputs
//!    left where they were, which is the measurement that says a disabled block is not a comparator saying "low".
//!
//! What this does **not** claim: any calibrated voltage. The ladder and the reference come from the same bandgap, so
//! the record reports an interval in millivolts computed from the reference's name, not a voltmeter reading. `VSENSE0`
//! is named by the register list and nothing more is claimed about it.
//!
//! VERIFIED ON HARDWARE (2026-11-27): the record, the control and what each run rules out are in
//! `docs/04-reference/ACMP-LADDER-2026-11-27.md`.

#![no_std]
#![no_main]

use efr32mg2x_hal::acmp::{self, Acmp0, Source};
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::entry;
use efr32mg2x_hal::record::Record;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

/// The positive side: the internal sense node divided by four, a fixed voltage the ladder can be walked against.
const POSITIVE: Source = Source::Sense0Div4;

/// The nominal millivolts of each reference tap, as the register list names them.
const REF_1V25_MV: u32 = 1_250;
const REF_2V5_MV: u32 = 2_500;

/// How many identical reads one sample makes. A comparator output that flickers is not a comparison.
const READS: u32 = 8;

/// Progress, written at every step: "no record" then says something died and where.
#[used]
#[no_mangle]
static ETAPAS: [core::sync::atomic::AtomicU32; 8] =
    [const { core::sync::atomic::AtomicU32::new(0) }; 8];

fn etapa(i: usize, v: u32) {
    ETAPAS[i].store(v, core::sync::atomic::Ordering::Relaxed);
}

/// What one sweep of the ladder produced.
struct Sweep {
    /// Taps compared, `1..=63`.
    samples: u32,
    /// Taps where the eight reads disagreed. Any non-zero value fails the run.
    unstable: u32,
    /// Times the output changed between adjacent taps. One is a crossing; zero and two are defects.
    edges: u32,
    /// The first tap that read low, or 0 if none did.
    first_low: u8,
}

/// Read the output `READS` times; `None` when they do not all agree.
fn read_stable(acmp: &Acmp0) -> Option<bool> {
    let first = acmp.output();
    for _ in 1..READS {
        if acmp.output() != first {
            return None;
        }
    }
    Some(first)
}

/// Walk the ladder with the given negative source, taking `LADDER_FULL` comparisons.
///
/// `fixed` holds the tap at one value instead of walking it. That is the control for the edge counter itself: the same
/// number of comparisons, the same source pair, and a tap that never moves. If that run also showed an edge, the
/// single edge of a real sweep would say nothing about the ladder.
fn walk(acmp: &mut Acmp0, neg: Source, fixed: Option<u8>) -> Result<Sweep, acmp::Error> {
    let mut out = Sweep {
        samples: 0,
        unstable: 0,
        edges: 0,
        first_low: 0,
    };
    let mut previous: Option<bool> = None;
    for step in 1..=acmp::LADDER_FULL {
        let div = fixed.unwrap_or(step);
        acmp.compare(POSITIVE, neg, div)?;
        // The mux is in the analogue domain; give it time to settle before believing the output.
        cortex_m::asm::delay(20_000);
        match read_stable(acmp) {
            Some(high) => {
                out.samples += 1;
                if let Some(was) = previous {
                    if was != high {
                        out.edges += 1;
                    }
                }
                if !high && out.first_low == 0 {
                    out.first_low = div;
                }
                previous = Some(high);
            }
            None => out.unstable += 1,
        }
    }
    Ok(out)
}

/// Walk every tap of the ladder with the given negative source.
fn sweep(acmp: &mut Acmp0, neg: Source) -> Result<Sweep, acmp::Error> {
    walk(acmp, neg, None)
}

/// The same number of comparisons with the tap held at pass-through.
fn flat_sweep(acmp: &mut Acmp0, neg: Source) -> Result<Sweep, acmp::Error> {
    walk(acmp, neg, Some(acmp::LADDER_FULL))
}

#[entry]
fn main() -> ! {
    mailbox::invalidate();

    let board = match Board::take() {
        Some(board) => {
            etapa(0, 1);
            board
        }
        None => {
            etapa(0, 9);
            mailbox::fail(b"acmp: peripherals taken twice");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let mut acmp = match board.acmp() {
        Ok(acmp) => {
            etapa(1, 1);
            acmp
        }
        Err(_) => {
            etapa(1, 9);
            mailbox::fail(b"acmp: ACMP0 already handed out");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let ready = acmp.is_ready();

    // ---- sweep 1: the 1.25 V tap ---------------------------------------------------------------------------------
    let s1 = match sweep(&mut acmp, Source::Divided1v25) {
        Ok(s) => s,
        Err(_) => {
            etapa(2, 9);
            mailbox::fail(b"acmp: the mux refused the 1V25 sweep");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    etapa(2, 1);

    // ---- sweep 2: the 2.5 V tap ----------------------------------------------------------------------------------
    let s2 = match sweep(&mut acmp, Source::Divided2v5) {
        Ok(s) => s,
        Err(_) => {
            etapa(3, 9);
            mailbox::fail(b"acmp: the mux refused the 2V5 sweep");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    etapa(3, 1);

    // ---- sweep 3: sweep 1 again ----------------------------------------------------------------------------------
    let s3 = match sweep(&mut acmp, Source::Divided1v25) {
        Ok(s) => s,
        Err(_) => {
            etapa(4, 9);
            mailbox::fail(b"acmp: the mux refused the repeat sweep");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    etapa(4, 1);

    // ---- control: the same 63 comparisons with the tap held at pass-through --------------------------------------
    let flat = match flat_sweep(&mut acmp, Source::Divided1v25) {
        Ok(s) => s,
        Err(_) => {
            etapa(5, 9);
            mailbox::fail(b"acmp: the mux refused the flat control");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    etapa(5, 1);

    // ---- control: swap the inputs at the pass-through tap ---------------------------------------------------------
    let swap_ok = acmp
        .compare(POSITIVE, Source::Divided1v25, acmp::LADDER_FULL)
        .is_ok();
    // `SYNCBUSY` clears when the mux accepted the write; the analogue path settles after that, which is why the
    // immediate read at this line returned the previous comparison on the first run of this example.
    cortex_m::asm::delay(20_000);
    let swap_a = acmp.output();
    let swap_back = acmp
        .compare(Source::Divided1v25, POSITIVE, acmp::LADDER_FULL)
        .is_ok();
    cortex_m::asm::delay(20_000);
    let swap_b = acmp.output();
    etapa(6, 1 + u32::from(swap_ok && swap_back));

    // ---- control: disabled, output follows NOTRDYVAL --------------------------------------------------------------
    let disabled = acmp.disable().is_ok();
    acmp.set_not_ready_value(false);
    let off_low = acmp.output();
    acmp.set_not_ready_value(true);
    let off_high = acmp.output();
    acmp.set_not_ready_value(false);
    etapa(7, 1 + u32::from(disabled));

    // ---- verdict -------------------------------------------------------------------------------------------------
    let window1 = acmp::window_mv(REF_1V25_MV, s1.first_low);
    let window3 = acmp::window_mv(REF_1V25_MV, s3.first_low);
    let window2 = acmp::window_mv(REF_2V5_MV, s2.first_low);
    // All three sweeps bracket the same node, so the honest interval is the intersection of all three. A signal that
    // sits within one tap of a boundary makes the crossing land on 43 in one sweep and 44 in the next -- the tap is
    // 19.8 mV wide, so that is the instrument's resolution, not a defect, and intersecting the windows absorbs it.
    let common = window1
        .and_then(|a| window3.and_then(|b| acmp::overlap(a, b)))
        .and_then(|a| window2.and_then(|b| acmp::overlap(a, b)));

    let taps = s1.samples + s2.samples + s3.samples + flat.samples;
    let unstable = s1.unstable + s2.unstable + s3.unstable + flat.unstable;
    let one_edge =
        s1.edges == 1 && s2.edges == 1 && s3.edges == 1 && s1.first_low > 1 && s2.first_low > 1;
    // One tap of tolerance: the crossing is quantised by the ladder, and a signal within a tap of a boundary flips
    // between two adjacent taps from one sweep to the next. Two taps would be a moving signal.
    let repeatable = i32::from(s1.first_low).abs_diff(i32::from(s3.first_low)) <= 1;
    // The flat control takes the same number of comparisons at one tap, so it must not turn at all: if it did, the
    // edge counter is measuring something other than the ladder and the sweeps above prove nothing.
    let flat_is_flat = flat.samples == u32::from(acmp::LADDER_FULL) && flat.edges == 0;
    // The 2.5 V reference is twice the 1.25 V one, so the crossing must be at half the tap -- within two taps, because
    // the two references are separate trims and the sweep is quantised.
    let halves = (i32::from(s1.first_low) - 2 * i32::from(s2.first_low)).abs() <= 2;

    let ok = ready
        && swap_ok
        && swap_back
        && disabled
        && taps == 4 * u32::from(acmp::LADDER_FULL)
        && unstable == 0
        && one_edge
        && repeatable
        && flat_is_flat
        && halves
        && common.is_some()
        && !swap_a
        && swap_b
        && !off_low
        && off_high;

    etapa(7, 1);

    let mut record: Record<200> = Record::new();
    record
        .s(b"acmp: ok=")
        .u(ok as u32)
        .s(b" n=")
        .u(taps)
        .s(b" c1=")
        .u(s1.first_low)
        .s(b" c2=")
        .u(s2.first_low)
        .s(b" c3=")
        .u(s3.first_low)
        .s(b" e=")
        .u(s1.edges)
        .u(s2.edges)
        .u(s3.edges)
        .s(b" f=")
        .u(flat.edges)
        .s(b" u=")
        .u(unstable)
        .s(b" lo=")
        .u(common.map(|w| w.0).unwrap_or(0))
        .s(b" hi=")
        .u(common.map(|w| w.1).unwrap_or(0))
        .s(b" sw=")
        .bit(swap_a)
        .bit(swap_b)
        .s(b" off=")
        .bit(off_low)
        .bit(off_high)
        .s(b" st=0x")
        .h32(acmp.status())
        .s(b" ic=0x")
        .h32({
            let (p, n, d) = acmp.inputs();
            u32::from(p) | (u32::from(n) << 8) | (u32::from(d) << 16)
        });

    if ok {
        mailbox::pass(record.bytes());
    } else {
        mailbox::fail(record.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
