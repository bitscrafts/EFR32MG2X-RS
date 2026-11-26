//! # Converting on a PRS edge instead of on a CPU write
//!
//! An ADC that only converts when the CPU asks it to is an ADC the CPU has to babysit. This part can start a single
//! conversion on a positive edge of an asynchronous PRS channel, and this example proves that path on the board with
//! the two controls that make the claim mean something:
//!
//! ```text
//!   A  software trigger, armed, eight pulses    -> one conversion (the arm), the pulses add nothing
//!   B  PRS trigger on channel 0, armed, eight   -> eight conversions, one value each
//!   C  PRS trigger on channel 0, armed, none    -> zero conversions
//!   D  PRS trigger on channel 0, **not** armed, eight -> zero conversions
//! ```
//!
//! Phase A is the control that says a count is not an artefact of arming a queue; phase C is the control that says it
//! is not an artefact of the pulses being written at all; phase D says the edges need an armed queue behind them. Only
//! together do they show the edge doing the work.
//!
//! The record, from two runs:
//!
//! ```text
//! adcprs: trig=prs0 refused=1 armA=1 imm=0 armB=0 prs=8 v0=0x26f v7=0x2a7 quiet=0 noarm=0
//! ```
//!
//! and the second run differs only in the two sampled values (`v0` and `v7` are PC9's bias, noise around mid-scale).
//!
//! ## Which seams this reaches
//!
//! The converter through [`Adc`](efr32mg2x_hal::adc::Adc) — `use_prs_trigger`, `use_software_trigger`, `arm_single`,
//! `take_single`, `select_input` — and the pulse through [`Prs::pulse`](efr32mg2x_hal::prs::Prs::pulse), which is the
//! whole of what the PRS driver does today: one write, no channel configuration, because the measurement below showed
//! none was needed.
//!
//! ## What it does not prove
//!
//! Nothing about a producer other than the CPU: whether a timer or a GPIO edge through the same channel works is a
//! different measurement. And the value is PC9's microphone bias, not a known voltage — the count is the point, the
//! value only shows that each conversion really happened.
//!
//! VERIFIED ON HARDWARE (2026-11-26).

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use efr32mg2x_hal as _;
use efr32mg2x_hal::adc::{Channel, Config, Error, Reference, Trigger};
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::pin::Port;
use efr32mg2x_hal::record::Record;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Pulses in the treated phase. Small enough that a one-deep FIFO cannot hide a missed conversion.
const PULSES: u32 = 8;

/// Polls allowed for one conversion. A conversion at 19 MHz is a few hundred cycles; this is generous and finite, so a
/// phase that converts nothing ends.
const POLL_LIMIT: u32 = 100_000;

/// What one phase measured.
struct Phase {
    /// Conversions the arm alone produced, in a window with no pulse in it.
    arm: u32,
    /// Conversions popped, one per pulse.
    conv: u32,
    /// The first conversion's value.
    first: u16,
    /// The last conversion's value.
    last: u16,
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
        Err(_) => {
            mailbox::fail(b"adcprs: the ADC is not available");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    if adc.use_prs_trigger(0).is_err() {
        mailbox::fail(b"adcprs: the converter refused the PRS trigger");
        loop {
            cortex_m::asm::wfi();
        }
    }
    let trigger = adc.trigger();

    // The software entry point must refuse while PRS owns the start: arming and polling would wait for an edge that
    // never comes and be read as a dead channel.
    let refused = adc.read(Channel::Ch0) == Err(Error::Triggered);

    let mut prs = match board.prs() {
        Ok(prs) => prs,
        Err(_) => {
            mailbox::fail(b"adcprs: the PRS is not available");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // The input the triggered conversions read: PC9, the microphone's output, biased to mid-scale. The CD bus first,
    // because an unallocated pin converts to zero without complaining.
    board.allocate_odd_cd_bus();
    if adc.select_input(Port::C, 9).is_err() {
        mailbox::fail(b"adcprs: the converter refused the input");
        loop {
            cortex_m::asm::wfi();
        }
    }

    // Phase A: back on the software trigger, where arming converts and the pulses must not.
    if adc.use_software_trigger().is_err() {
        mailbox::fail(b"adcprs: the converter refused the software trigger");
        loop {
            cortex_m::asm::wfi();
        }
    }
    let phase_a = run_phase(&mut adc, &mut prs, true, true);

    // Phase B: PRS on channel 0, one conversion per edge.
    if adc.use_prs_trigger(0).is_err() {
        mailbox::fail(b"adcprs: the converter refused the PRS trigger");
        loop {
            cortex_m::asm::wfi();
        }
    }
    let phase_b = run_phase(&mut adc, &mut prs, true, true);

    // Phase C: the same configuration, armed, and no pulse.
    if adc.use_prs_trigger(0).is_err() {
        mailbox::fail(b"adcprs: the converter refused the PRS trigger");
        loop {
            cortex_m::asm::wfi();
        }
    }
    let phase_c = run_phase(&mut adc, &mut prs, true, false);

    // Phase D: pulses with no arm behind them. Disabling and re-enabling the converter resets the queue, so this
    // phase starts from an unarmed one.
    if adc.use_prs_trigger(0).is_err() {
        mailbox::fail(b"adcprs: the converter refused the PRS trigger");
        loop {
            cortex_m::asm::wfi();
        }
    }
    let phase_d = run_phase(&mut adc, &mut prs, false, true);

    let mut t = Record::<200>::new();
    t.s(b"adcprs: trig=");
    t.s(match trigger {
        Trigger::PrsPositiveEdge(0) => b"prs0",
        Trigger::PrsPositiveEdge(_) => b"prsN",
        Trigger::Software => b"sw",
    });
    t.s(b" refused=");
    t.u(refused as u32);
    t.s(b" armA=");
    t.u(phase_a.arm);
    t.s(b" imm=");
    t.u(phase_a.conv);
    t.s(b" armB=");
    t.u(phase_b.arm);
    t.s(b" prs=");
    t.u(phase_b.conv);
    t.s(b" v0=0x");
    t.h32(phase_b.first as u32);
    t.s(b" v7=0x");
    t.h32(phase_b.last as u32);
    t.s(b" quiet=");
    t.u(phase_c.conv);
    t.s(b" noarm=");
    t.u(phase_d.conv);

    let pass = refused
        && phase_a.arm == 1
        && phase_a.conv == 0
        && phase_b.arm == 0
        && phase_b.conv == PULSES
        && phase_c.conv == 0
        && phase_d.conv == 0;

    if pass {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}

/// Arm the single queue, wait a fixed window with nothing driving the trigger, then pulse `PULSES` times (or not at
/// all), popping one conversion after each pulse.
fn run_phase(
    adc: &mut efr32mg2x_hal::adc::Adc,
    prs: &mut efr32mg2x_hal::prs::Prs,
    arm: bool,
    do_pulse: bool,
) -> Phase {
    // Anything left over from the previous phase is not this phase's conversion.
    while adc.take_single(1).is_some() {}

    if arm {
        adc.arm_single();
    }

    let mut arm = 0u32;
    for _ in 0..POLL_LIMIT {
        if adc.take_single(1).is_some() {
            arm += 1;
        }
    }

    let mut conv = 0u32;
    let mut first = 0u16;
    let mut last = 0u16;
    for _ in 0..PULSES {
        if do_pulse {
            let _ = prs.pulse(0);
        }
        if let Some(v) = adc.take_single(POLL_LIMIT) {
            if conv == 0 {
                first = v;
            }
            conv += 1;
            last = v;
        }
    }

    Phase {
        arm,
        conv,
        first,
        last,
    }
}
