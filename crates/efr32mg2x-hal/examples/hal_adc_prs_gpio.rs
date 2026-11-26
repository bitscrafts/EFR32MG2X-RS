//! # Converting on a GPIO edge instead of on anything the CPU starts
//!
//! `Source::GpioPin(n)` was an encoding nothing drove: the two producers measured before it were the CPU's own pulse
//! and TIMER0's overflow. This example drives it, moves the pad with the pin's own pull resistor and counts what the
//! converter makes of it — and finds that eight pad changes are four conversions, because the producer carries the
//! pad's level rather than its edges.
//!
//! ## How a GPIO producer is made without a jumper wire
//!
//! An external interrupt watches a pad, so the pad has to move. On a pin with nothing attached the pad can be moved
//! by the pin's own pull resistor: in the pull mode the output latch selects the resistor rather than a driver, so
//! writing the latch high ties the pad towards VDD and writing it low pulls it towards ground. That is a slow edge
//! and a real one, and it needs no wire, no hand on a button and no second pin. The record shows it working: eight
//! toggles, `mv=8`, and `din=1` on the finished phase.
//!
//! ## The phases
//!
//! ```text
//!   A  channel 3 bound, armed, pad left quiet                  -> zero conversions
//!   B  channel 3, eight toggles, no flag clearing               -> the same count as C
//!   C  channel 3, eight toggles, flag cleared after each edge   -> one per rising edge
//!   D  channel Off, eight toggles on the same pad               -> zero conversions
//!   E  channel 3, eight toggles on a *different* pad            -> zero conversions
//!   F  channel 0, eight toggles on that other pad               -> the same count on a second pin
//!   G  channel 3, sixteen toggles                               -> twice the count, twice the rising edges
//!   H  channel 3 armed for falling edges only, eight toggles    -> unchanged
//! ```
//!
//! A and D differ in one field (the channel's `SOURCESEL`); E is the pin binding under test, where the toggles happen
//! on a pad the bound channel is not watching; G is the scaling control; H is the polarity probe.
//!
//! ## What the hardware turned out to do
//!
//! The producer is a **level**, not an edge. Eight pad changes are four rising edges and four falling ones, and the
//! board converted four times — with the flag cleared between edges and without it (`c=4 b=4`). Sixteen changes gave
//! eight (`long=8 upg=8`), so it scales with one direction of movement rather than losing a fixed half.
//!
//! The decisive field is H: with `EXTIRISE` cleared for the channel and only `EXTIFALL` armed, the count stayed at
//! four (`fall=4`). `EXTIRISE`/`EXTIFALL` are therefore **not** in the PRS path — the asynchronous signal carries the
//! pad's level, and the converter's `PRSPOS` trigger takes its rising edge. What the masks do configure is the GPIO
//! interrupt flag, which is a separate consumer of the same edge detector.
//!
//! The flag is likewise not the signal: clearing it after every edge changed nothing (`b=4 c=4`), and the record
//! still shows the clear working — `ifb=0x9` (channels 0 and 3, the second one from the other pin's initial pull-up
//! write) against `ifc=0x1`, so the write-one-to-clear alias at `IF + 0x2000` clears exactly the channel it names and
//! leaves the other bit alone.
//!
//! ## The record, from two runs
//!
//! ```text
//! adcgpio: ch=3 sel=0x00002000 isel=0x00003000 din=1 a=0 b=4 c=4 mv=8 up=4 off=0 other=0 pin0=4 up0=4 long=8 upg=8 fall=4 ifb=0x00000009 ifc=0x00000001 v0=0x00000231 vl=0x0000023a
//! adcgpio: ch=3 sel=0x00002000 isel=0x00003000 din=1 a=0 b=4 c=4 mv=8 up=4 off=0 other=0 pin0=4 up0=4 long=8 upg=8 fall=4 ifb=0x00000009 ifc=0x00000001 v0=0x00000233 vl=0x0000023a
//! ```
//!
//! The only fields that differ are the sampled values, which are PC9's bias with the microphone's noise on it. `sel`
//! and `isel` are the readback of the two select words: channel 3 of port C is a port field of 2 at bits 12:13
//! (`0x2000`) and a pin field of 3 at bits 12:13 of `EXTIPINSELL` (`0x3000`), which is the two-bit-per-channel
//! encoding both the SVD and the vendor header describe.
//!
//! ## What it does not prove
//!
//! Nothing about a pad driven by a wire or by another peripheral, and nothing about rate: the pulls move the pad in
//! tens of microseconds at best, so this says which edges convert, not how fast a GPIO producer can pace a converter.
//! A pin in 8..15 is not covered either: the channel exists, but the vendor signal table stops at `GPIOPIN7`.
//!
//! Record: `docs/04-reference/PRS-GPIO-PRODUCER-2026-11-26.md`.
//!
//! VERIFIED ON HARDWARE (2026-11-26).

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use efr32mg2x_hal as _;
use efr32mg2x_hal::adc::{Config, Reference};
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::pin::{EdgeWatch, Input, Port};
use efr32mg2x_hal::prs::{Prs, Source};
use efr32mg2x_hal::record::Record;
use embedded_hal::delay::DelayNs;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Edges per treated window. Eight is small enough that a one-deep FIFO cannot hide a lost conversion and large
/// enough that one stray edge does not look like a working channel.
const EDGES: u32 = 8;

/// Time between two toggles. The converter needs microseconds; this is four hundred times that, so a phase can only
/// fail for a reason other than the FIFO being read too late.
const SPACING_US: u32 = 500;

/// Polls allowed for one conversion to land in the FIFO.
const DRAIN_POLLS: u32 = 1000;

/// The pin under test: PC3, channel 3, port and pin fields both non-zero.
const TEST_PIN: u8 = 3;

/// The second pin: PA0, channel 0, both fields zero. The same experiment on the encoding's most boring value.
const OTHER_PIN: u8 = 0;

/// What one window measured.
struct Window {
    /// Conversions popped while the window ran.
    conv: u32,
    /// The first conversion's value.
    first: u16,
    /// The last conversion's value.
    last: u16,
    /// Times the pad read a different level than it had after the previous toggle — the edge the hardware had a
    /// chance to see. A pin that never moved cannot have produced an edge, and this is the field that says so.
    moved: u32,
    /// Of those changes, the ones that went low to high.
    rising: u32,
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
        Err(_) => fail(b"adcgpio: the ADC is not available"),
    };
    let mut prs = match board.prs() {
        Ok(prs) => prs,
        Err(_) => fail(b"adcgpio: the PRS is not available"),
    };

    if adc.use_prs_trigger(0).is_err() {
        fail(b"adcgpio: the converter refused the PRS trigger");
    }

    // PC9, the microphone's output, biased to mid-scale: an input whose value is not zero, so a conversion is
    // visible in the record and not just in the count.
    board.allocate_odd_cd_bus();
    if adc.select_input(Port::C, 9).is_err() {
        fail(b"adcgpio: the converter refused the input");
    }

    // The two pads. Both are inputs; the pull resistor is what moves them.
    let tested = board.input_on(Port::C, TEST_PIN);
    let tested_watch: EdgeWatch<'_> = board.watch_edges(Port::C, TEST_PIN, true, true);
    let other = board.input_on(Port::A, OTHER_PIN);
    let other_watch: EdgeWatch<'_> = board.watch_edges(Port::A, OTHER_PIN, true, true);

    // A: the channel is bound to the pin under test and the pin is left alone.
    let a = phase(
        &mut adc,
        &board,
        &mut prs,
        &tested,
        &tested_watch,
        TEST_PIN,
        0,
        false,
    );

    // B: eight edges, and nobody touches the flag between them.
    let b = phase(
        &mut adc,
        &board,
        &mut prs,
        &tested,
        &tested_watch,
        TEST_PIN,
        EDGES,
        false,
    );
    let ifb = tested_watch.flags();

    // C: the same eight edges, with the flag cleared after each one. If B and C disagree, the PRS is carrying the
    // latched flag rather than the edge, and clearing is what makes the second edge visible at all.
    let c = phase(
        &mut adc,
        &board,
        &mut prs,
        &tested,
        &tested_watch,
        TEST_PIN,
        EDGES,
        true,
    );
    let ifc = tested_watch.flags();

    // D: the channel pointed at nothing, with the pin still moving.
    let d = phase(
        &mut adc,
        &board,
        &mut prs,
        &tested,
        &tested_watch,
        0xff,
        EDGES,
        true,
    );

    // E: channel 3 armed, but the pad that moves is the other one.
    let e = phase(
        &mut adc,
        &board,
        &mut prs,
        &other,
        &tested_watch,
        TEST_PIN,
        EDGES,
        true,
    );

    // F: the other pad, with the channel that watches it.
    let f = phase(
        &mut adc,
        &board,
        &mut prs,
        &other,
        &other_watch,
        OTHER_PIN,
        EDGES,
        true,
    );

    // G: twice the edges. If the converter is following one direction of the pad's movement, sixteen toggles must
    // give eight conversions and eight rising edges; if the first run lost half of them to something else, the count
    // will not scale.
    let g = phase(
        &mut adc,
        &board,
        &mut prs,
        &tested,
        &tested_watch,
        TEST_PIN,
        2 * EDGES,
        true,
    );

    // H: the same eight edges with only the falling mask armed. This is the polarity probe: a count that stays at
    // zero says the mask gates what the PRS sees, a count that follows the rising edges says the mask is not in the
    // PRS path at all and the converter is watching the pad's level.
    let falling_only = board.watch_edges(Port::C, TEST_PIN, false, true);
    let h = phase(
        &mut adc,
        &board,
        &mut prs,
        &tested,
        &falling_only,
        TEST_PIN,
        EDGES,
        true,
    );

    let (sel, isel) = tested_watch.selection();

    let mut t = Record::<200>::new();
    t.s(b"adcgpio: ch=");
    t.u(tested_watch.channel());
    t.s(b" sel=0x");
    t.h32(sel);
    t.s(b" isel=0x");
    t.h32(isel);
    t.s(b" din=");
    t.bit(tested.is_high());
    t.s(b" a=");
    t.u(a.conv);
    t.s(b" b=");
    t.u(b.conv);
    t.s(b" c=");
    t.u(c.conv);
    t.s(b" mv=");
    t.u(c.moved);
    t.s(b" up=");
    t.u(c.rising);
    t.s(b" off=");
    t.u(d.conv);
    t.s(b" other=");
    t.u(e.conv);
    t.s(b" pin0=");
    t.u(f.conv);
    t.s(b" up0=");
    t.u(f.rising);
    t.s(b" long=");
    t.u(g.conv);
    t.s(b" upg=");
    t.u(g.rising);
    t.s(b" fall=");
    t.u(h.conv);
    t.s(b" ifb=0x");
    t.h32(ifb);
    t.s(b" ifc=0x");
    t.h32(ifc);
    t.s(b" v0=0x");
    t.h32(c.first as u32);
    t.s(b" vl=0x");
    t.h32(c.last as u32);

    // The count is compared to the rising edges the pad actually made, not to the edges the loop asked for: a pad
    // that never moved and a channel that converted nothing agree at zero, and that agreement is not a working
    // producer. B is not required to equal C — whether the flag has to be drained between edges is what C exists to
    // measure — but B cannot exceed C.
    let pass = sel == 0x0000_2000
        && isel == 0x0000_3000
        && a.conv == 0
        && c.conv == c.rising
        && c.rising >= 1
        && d.conv == 0
        && e.conv == 0
        && f.conv == f.rising
        && f.rising >= 1
        && g.conv == g.rising
        && b.conv <= c.conv;
    if pass {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}

/// Run one window: drain what the previous phase left, point the channel at `channel` (`0xff` for none), arm, move
/// the pad `toggles` times, then pop until the FIFO stays empty.
fn phase(
    adc: &mut efr32mg2x_hal::adc::Adc,
    board: &Board,
    prs: &mut Prs,
    input: &Input<'_>,
    watch: &EdgeWatch<'_>,
    channel: u8,
    toggles: u32,
    clear_each: bool,
) -> Window {
    while adc.take_single(1).is_some() {}

    let source = if channel > 7 {
        Source::Off
    } else {
        Source::GpioPin(channel)
    };
    if prs.connect(0, source).is_err() {
        fail(b"adcgpio: the PRS refused the channel");
    }
    // `use_prs_trigger` cycles `EN` and leaves the single queue empty, which is the only measured way to get an
    // unarmed queue back before arming it again.
    if adc.use_prs_trigger(0).is_err() {
        fail(b"adcgpio: the converter refused the PRS trigger");
    }
    watch.clear();
    let mut hold = board.delay();
    hold.delay_us(SPACING_US);

    adc.arm_single();

    let mut moved = 0u32;
    let mut rising = 0u32;
    let mut level = input.is_high();
    for i in 0..toggles {
        // Odd toggles pull up, even ones pull down: every write moves the pad, so every write is one edge.
        input.set_pull_up(i % 2 == 1);
        let mut gap = board.delay();
        gap.delay_us(SPACING_US);
        let now = input.is_high();
        if now != level {
            moved += 1;
            if now {
                rising += 1;
            }
        }
        level = now;
        if clear_each {
            watch.clear();
        }
    }

    // A window with no toggles still waits as long as a treated one: a phase that fails has to fail while armed,
    // not because it ended early.
    if toggles == 0 {
        let mut wait = board.delay();
        wait.delay_us(EDGES * SPACING_US);
    }

    let mut conv = 0u32;
    let mut first = 0u16;
    let mut last = 0u16;
    while conv < 4 * EDGES + 4 {
        match adc.take_single(DRAIN_POLLS) {
            Some(v) => {
                if conv == 0 {
                    first = v;
                }
                conv += 1;
                last = v;
            }
            None => break,
        }
    }

    Window {
        conv,
        first,
        last,
        moved,
        rising,
    }
}

fn fail(message: &[u8]) -> ! {
    mailbox::fail(message);
    loop {
        cortex_m::asm::wfi();
    }
}
