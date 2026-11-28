//! Does the MVP pay for itself? One number, measured both ways — with a clock that works without a probe.
//!
//! The MVP is a matrix/vector processor that reads and writes system memory. Whether it is worth using for the
//! dosimeter's model is not a matter of opinion: it is the cost of the same work done two ways, measured on this part.
//!
//! ## What changed, and why the old record was not usable
//!
//! This example used to time both paths with the DWT cycle counter. That counter only runs while a debug session is
//! attached (`docs/04-reference/DWT-NEEDS-A-DEBUG-SESSION.md`), so the numbers were true only under `probe-rs run` — and
//! a product has no probe. A benchmark a product cannot reproduce is not evidence about the product.
//!
//! The clock here is the **SYSRTC**: 32.768 kHz, no debugger needed, and measured against the host in `hal_hfxo` (a
//! 1000 ms delay came back as 1006 ms). A tick is 30.5 µs, far coarser than one 64-element addition (~15 µs), so the
//! measurement is a **batch**: `REPS` additions, one reading before and one after, and the report divides. The
//! quantisation is then one tick over the batch — a fraction of a percent at 256 repetitions — and the record states the
//! batch size so a reader can see how much of the number is resolution.
//!
//! ## What it measures, and what it does not
//!
//! The *arithmetic loop*: element-wise addition of two `binary16` vectors of `N` elements. It does not measure a matrix
//! multiply, and it does not measure getting a model's coefficients into the arrays. Treat it as the floor for the
//! MVP's advantage, not as the answer for a whole model.
//!
//! The self-check is separate from the timing on purpose: every input is `1.0`, so every output must be `2.0`
//! (`0x4000`), and a wrong result fails the record however good the numbers look.
//!
//! VERIFIED ON HARDWARE (2026-11-25), no probe attached, four boots:
//! `PASS mvp: n=64 reps=256 tick_mvp=123 tick_cpu=288 mvp_us_x100=1466 cpu_us_x100=3433 ratio_x100=234`
//! and three more within one tick (`123/288`, `123/288`, `123/289`) — 14.7 µs per 64-element add on the MVP
//! against 34.3 µs in Rust, 2.34x.

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

/// 64 elements: about the size of one band's worth of coefficients in the dosimeter's GMM.
const N: usize = 64;

/// Additions per batch. Large enough that one SYSRTC tick (30.5 µs) is a fraction of a percent of the total, small
/// enough that the whole batch stays far below the counter's wrap.
const REPS: u32 = 256;

/// `1.0` in IEEE binary16. The self-check expects `1.0 + 1.0 = 2.0` (`0x4000`) everywhere.
const ONE_FP16: u16 = 0x3C00;
const TWO_FP16: u16 = 0x4000;

/// A ceiling a wedged co-processor would breach: one 64-element addition must not take 100 µs. This is the part of the
/// verdict that can fail on a block that looks alive.
const MAX_US_X100: u64 = 10_000;

/// Microseconds per addition, in hundredths, from ticks over a batch.
fn us_x100(ticks: u64) -> u64 {
    // ticks * 1e6 µs / 32768 ticks-per-second / REPS, times 100 for the two decimal places.
    ticks.saturating_mul(1_000_000 * 100) / (32_768 * u64::from(REPS))
}

#[entry]
fn main() -> ! {
    mailbox::invalidate();

    let board = match Board::take() {
        Some(b) => b,
        None => loop {
            core::hint::spin_loop();
        },
    };
    let rtc = match board.sysrtc() {
        Ok(rtc) => rtc,
        Err(_) => {
            mailbox::fail(b"mvp: no SYSRTC to measure with");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let mut mvp = match board.mvp() {
        Ok(m) => m,
        Err(_) => {
            // Either the block refused to leave its disabled state, or this board already handed it out.
            mailbox::fail(b"mvp: not handed out (stuck disabling, or taken)");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let a = [ONE_FP16; N];
    let b = [ONE_FP16; N];
    let mut out_mvp = [0u16; N];
    let mut out_cpu = [0u16; N];

    // One warm-up of each, so the first measurement does not pay for a cold loop and an idle co-processor.
    let _ = mvp.add_fp16(&a, &b, &mut out_mvp);
    for i in 0..N {
        out_cpu[i] = core::hint::black_box(a[i]).wrapping_add(core::hint::black_box(b[i]));
    }

    // The MVP: one call per repetition, the whole loop in hardware.
    let t0 = rtc.now_wide();
    let mut mvp_ok = true;
    for _ in 0..REPS {
        mvp_ok &= mvp.add_fp16(&a, &b, &mut out_mvp).is_ok();
    }
    let mvp_ticks = rtc.now_wide().wrapping_sub(t0);

    // The same loop in Rust, over the same memory. `black_box` keeps the compiler from folding it away -- measured
    // 2026-11-17: without it the loop "took 2 cycles", which is what an optimised-away loop costs.
    let t1 = rtc.now_wide();
    for _ in 0..REPS {
        for i in 0..N {
            out_cpu[i] = core::hint::black_box(a[i]).wrapping_add(core::hint::black_box(b[i]));
        }
    }
    let cpu_ticks = rtc.now_wide().wrapping_sub(t1);

    // Keep the inputs alive past the measurement so the compiler cannot treat them as constants and fold the work
    // away. Writing into them would be a dead store (measured: rustc says so), and a dead store does not keep
    // anything alive; `black_box` is the hint that does.
    core::hint::black_box((&a, &b));
    core::hint::black_box((mvp_ticks, cpu_ticks));

    let mut correct = true;
    for v in &out_mvp {
        if *v != TWO_FP16 {
            correct = false;
        }
    }

    let mvp_us = us_x100(mvp_ticks);
    let cpu_us = us_x100(cpu_ticks);

    let mut t = Record::<200>::new();
    t.s(b"mvp: n=");
    t.u(N as u64);
    t.s(b" reps=");
    t.u(u64::from(REPS));
    t.s(b" tick_mvp=");
    t.u(mvp_ticks);
    t.s(b" tick_cpu=");
    t.u(cpu_ticks);
    t.s(b" mvp_us_x100=");
    t.u(mvp_us);
    t.s(b" cpu_us_x100=");
    t.u(cpu_us);
    t.s(b" ratio_x100=");
    if mvp_ticks > 0 {
        t.u(cpu_ticks.saturating_mul(100) / mvp_ticks);
    } else {
        t.s(b"?");
    }

    // The verdict: the arithmetic is right, both clocks advanced (a zero would mean a stopped counter, which is how this
    // instrument used to lie), and the co-processor is not wedged.
    let passed =
        correct && mvp_ok && mvp_ticks > 0 && cpu_ticks > 0 && mvp_us > 0 && mvp_us <= MAX_US_X100;
    if passed {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
