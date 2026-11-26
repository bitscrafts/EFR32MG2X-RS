//! # Converting on a timer's overflow instead of on anything the CPU does
//!
//! The PRS trigger measured on 2026-11-26 was driven by `ASYNC_SWPULSE` — a CPU write. That proves the converter's
//! edge path, not the reflex system: a real producer is a block that keeps making edges while the core does nothing.
//! This example points asynchronous channel 0 at TIMER0's overflow and counts what arrives.
//!
//! ```text
//!   A  source Off, armed, timer running              -> zero conversions
//!   B  source TIMER0, producer not clocked, armed     -> zero conversions
//!   C  source TIMER0, running, **not** armed          -> zero conversions
//!   D  source TIMER0, running, armed, eight periods   -> eight conversions
//!   E  source Off again, armed, timer still running   -> zero conversions
//! ```
//!
//! A and E are the same configuration as D with one field changed (the channel's `SOURCESEL`), and B changes only
//! which timer is named — TIMER1 is never started, so it is a producer that exists in the encoding and produces
//! nothing. C removes the arm. Only together do the four say the timer's edges are doing the work.
//!
//! ## The producer
//!
//! [`Board::pace_microphone`](efr32mg2x_hal::board::Board::pace_microphone) starts TIMER0 free-running from
//! HFRCOEM23 with no pin and no PWM channel. The same routine paces the microphone's DMA, and it is reused here
//! rather than re-derived: one timer configuration, measured once. Its return value is the period in timer clocks, so
//! the example measures its own window in the timer's own units instead of in a spin count.
//!
//! ## What it does not prove
//!
//! Nothing about a GPIO producer: `Source::GpioPin` is encoded from the vendor's signal header and no pin routes it
//! here. And the count is bounded by one-deep FIFO pops, so this measures eight edges paced two milliseconds apart,
//! not the fastest rate the converter can follow.
//!
//! ## The record
//!
//! From two runs, `status=PASS`:
//!
//! ```text
//! adcprst: trig=prs0 hz=500 period=38000 src=0x000c2101 off=0 idle=0 noarm=0 ev=8 quiet=0 v0=0x0000026d vl=0x00000238 ovf=8 ch0=0x000c0000
//! adcprst: trig=prs0 hz=500 period=38000 src=0x000c2101 off=0 idle=0 noarm=0 ev=8 quiet=0 v0=0x0000026b vl=0x00000230 ovf=8 ch0=0x000c0000
//! ```
//!
//! The only fields that differ are the two sampled values, which are PC9's bias with the microphone's noise on it.
//! `src` is the readback of `ASYNC_CH0_CTRL` with TIMER0 connected (`FNSEL = 0xC`, `SOURCESEL = 0x21`,
//! `SIGSEL = 1`), and `ch0` is the same word after the channel is pointed back at nothing (`0x000c0000`, the reset
//! value).
//!
//! VERIFIED ON HARDWARE (2026-11-26).

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use efr32mg2x_hal as _;
use efr32mg2x_hal::adc::{Config, Reference, Trigger};
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::pin::Port;
use efr32mg2x_hal::prs::Source;
use efr32mg2x_hal::record::Record;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Timer overflows the treated window must contain. Eight is small enough that a one-deep FIFO cannot hide a lost
/// conversion and large enough that a single stray edge does not look like a working channel.
const EVENTS: u32 = 8;

/// Polls allowed for the last conversion of a window to land in the FIFO.
const DRAIN_POLLS: u32 = 1000;

/// The pace of the producer. 500 Hz is 38 000 timer clocks per period at 19 MHz: long enough that a conversion and a
/// pop fit inside one period many times over, short enough that the whole run is a few tens of milliseconds.
const PACE_HZ: u32 = 500;

/// What one window measured.
struct Window {
    /// Conversions popped while the window ran.
    conv: u32,
    /// The first conversion's value.
    first: u16,
    /// The last conversion's value.
    last: u16,
    /// TIMER0 overflows counted inside the window.
    ticks: u32,
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
    let mut adc = match board.adc(Config::default().with_reference(Reference::Vbgr)) {
        Ok(adc) => adc,
        Err(_) => fail(b"adcprst: the ADC is not available"),
    };
    let mut prs = match board.prs() {
        Ok(prs) => prs,
        Err(_) => fail(b"adcprst: the PRS is not available"),
    };

    if adc.use_prs_trigger(0).is_err() {
        fail(b"adcprst: the converter refused the PRS trigger");
    }
    let trigger = adc.trigger();

    // PC9, the microphone's output, biased to mid-scale: an input whose value is not zero, so a conversion is visible
    // in the record and not just in the count.
    board.allocate_odd_cd_bus();
    if adc.select_input(Port::C, 9).is_err() {
        fail(b"adcprst: the converter refused the input");
    }

    // The producer. TIMER0 now free-runs and overflows every `period` clocks, whether or not anything listens.
    let period = match board.pace_microphone(PACE_HZ) {
        Some(period) => period,
        None => fail(b"adcprst: TIMER0 could not be paced"),
    };

    // A: the channel has no source at all.
    if prs.connect(0, Source::Off).is_err() {
        fail(b"adcprst: the PRS refused the channel");
    }
    let a = run_window(&mut adc, &board, true, EVENTS);

    // B: a producer that exists in the encoding and is not running.
    if prs.connect(0, Source::Timer1Overflow).is_err() {
        fail(b"adcprst: the PRS refused the channel");
    }
    let b = run_window(&mut adc, &board, true, EVENTS);

    // C: the real producer with no arm behind it. The trigger path is re-armed through `use_prs_trigger` first,
    // which cycles `EN` and leaves the queue empty; measured 2026-11-26 to be the way to get an unarmed queue back.
    if prs.connect(0, Source::Timer0Overflow).is_err() {
        fail(b"adcprst: the PRS refused the channel");
    }
    if adc.use_prs_trigger(0).is_err() {
        fail(b"adcprst: the converter refused the PRS trigger");
    }
    let c = run_window(&mut adc, &board, false, EVENTS);
    let src = prs.channel_ctrl(0).unwrap_or(0);

    // D: the same channel, armed, with the timer running.
    let d = run_window(&mut adc, &board, true, EVENTS);

    // E: back to no source, timer still running, armed.
    if prs.connect(0, Source::Off).is_err() {
        fail(b"adcprst: the PRS refused the channel");
    }
    let e = run_window(&mut adc, &board, true, EVENTS);
    let ch0 = prs.channel_ctrl(0).unwrap_or(0);

    let mut t = Record::<200>::new();
    t.s(b"adcprst: trig=");
    t.s(match trigger {
        Trigger::PrsPositiveEdge(0) => b"prs0",
        Trigger::PrsPositiveEdge(_) => b"prsN",
        Trigger::Software => b"sw",
    });
    t.s(b" hz=");
    t.u(PACE_HZ);
    t.s(b" period=");
    t.u(period);
    t.s(b" src=0x");
    t.h32(src);
    t.s(b" off=");
    t.u(a.conv);
    t.s(b" idle=");
    t.u(b.conv);
    t.s(b" noarm=");
    t.u(c.conv);
    t.s(b" ev=");
    t.u(d.conv);
    t.s(b" quiet=");
    t.u(e.conv);
    t.s(b" v0=0x");
    t.h32(d.first as u32);
    t.s(b" vl=0x");
    t.h32(d.last as u32);
    t.s(b" ovf=");
    t.u(d.ticks);
    t.s(b" ch0=0x");
    t.h32(ch0);

    // The source word is checked as well as the counts: eight conversions on a channel the record says is pointed
    // somewhere else would be a different measurement than the one this example claims.
    let pass = src == 0x000c_2101
        && a.conv == 0
        && b.conv == 0
        && c.conv == 0
        && d.conv == EVENTS
        && e.conv == 0
        && ch0 == 0x000c_0000;
    if pass {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}

/// Run one window: drain whatever is left from the phase before, optionally arm the single queue, then pop
/// conversions until TIMER0 has wrapped `wraps` times.
///
/// The window is counted in the producer's **overflows**, not in timer clocks: `CNT` is 16 bits and wraps at `TOP`
/// (`38 000` here), so a `wrapping_sub` of two `u32` reads of it walks backwards once per period — measured on this
/// board 2026-11-26, the first version of this example did exactly that and its window closed 90 clocks into the run
/// (`win=4294967197`). Counting a decrease of the counter is immune to that, because it is the wrap itself that is
/// being counted.
///
/// The last edge's conversion is popped after the last wrap is seen: the conversion takes microseconds and the next
/// wrap is two milliseconds away, so one bounded drain is enough.
fn run_window(adc: &mut efr32mg2x_hal::adc::Adc, board: &Board, arm: bool, wraps: u32) -> Window {
    // Anything already in the FIFO is not this window's conversion.
    while adc.take_single(1).is_some() {}

    if arm {
        adc.arm_single();
    }

    let mut conv = 0u32;
    let mut first = 0u16;
    let mut last = 0u16;
    let mut seen = 0u32;
    let mut prev = board.timer0_counter();
    while seen < wraps {
        if let Some(v) = adc.take_single(1) {
            if conv == 0 {
                first = v;
            }
            conv += 1;
            last = v;
        }
        let now = board.timer0_counter();
        if now < prev {
            seen += 1;
        }
        prev = now;
    }

    // The conversion started by the last overflow is still in flight when the wrap is seen.
    if let Some(v) = adc.take_single(DRAIN_POLLS) {
        if conv == 0 {
            first = v;
        }
        conv += 1;
        last = v;
    }

    Window {
        conv,
        first,
        last,
        ticks: seen,
    }
}

fn fail(message: &[u8]) -> ! {
    mailbox::fail(message);
    loop {
        cortex_m::asm::wfi();
    }
}
