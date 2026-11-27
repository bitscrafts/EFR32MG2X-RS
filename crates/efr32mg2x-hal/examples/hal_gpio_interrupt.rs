//! # A GPIO edge that reaches the core, counted by the interrupt handler
//!
//! The pad moved by its own pull resistor, `IEN` and the NVIC armed, and a counter only the handler increments.
//!
//! The board could already act on a GPIO edge without the CPU: the PRS carries the pad's signal into a converter or a
//! counter (measured 2026-11-26). What it could not do was take the same edge as an **interrupt** — `CLAUDE.md` said
//! so in one line. This example closes that: a pad moved by its own pull resistor, `IEN` and the NVIC armed, and a
//! counter that only the handler increments.
//!
//! ## How the pad moves without a wire
//!
//! In the pull mode the output latch selects the resistor rather than driving the pad, so `set_pull_up(true)` ties a
//! bare pad towards VDD and `set_pull_up(false)` towards ground. Every write is a real edge (the PRS measurement used
//! the same trick) and no jumper, finger or second pin is involved.
//!
//! ## The phases
//!
//! ```text
//!   A  channel 3 armed, pad left quiet                       -> zero counts
//!   B  channel 3, eight pad changes                          -> one count per change
//!   C  channel 3, both edge masks off, eight changes          -> zero counts
//!   D  channel 3, interrupt disabled at the pin (`IEN` = 0)   -> zero counts
//!   E  channel 3, `IEN` set, vector masked in the NVIC        -> zero counts, flag latched
//!   F  channel 0 on PA0 (the even vector), eight changes      -> one count per change
//! ```
//!
//! C, D and E are the controls, and each one removes exactly one link of the chain: the edge masks (C), the per-pin
//! interrupt gate (D), and the vector (E). B and F are the two halves of the parity split — channel 3 is delivered on
//! `GPIO_ODD_IRQHandler`, channel 0 on `GPIO_EVEN_IRQHandler`, and the record carries how many times each vector
//! entered so the split is measured rather than assumed.
//!
//! ## What a pass means
//!
//! `cnt` equals `n`, where `n` is how many times the pad was **seen** to change level; a pad that never moved and a
//! channel that counted nothing agree at zero, so the pad's own observation is the denominator and not the loop's
//! counter. `if` is the flag word read with the vector masked, which is the one place the hardware's latch is visible
//! without the handler having already cleared it.
//!
//! ## What the hardware turned out to do
//!
//! Eight pad changes are eight interrupts and eight counts (`n=8 cnt=8`), on both parities: the odd channel 3 gave
//! `odd=8 even=0`, and channel 0 on PA0 gave `pin0=8 p0even=8 p0odd=0`. The three controls held at zero — masks off
//! (`off=0`), `IEN` cleared (`dis=0`), vector masked (`msk=0`) — and the fourth reading is what makes the masked one
//! worth anything: with the vector masked the flag still latched, `if=0x00000008`, which is channel 3's bit and
//! nothing else. So the edge does reach the block with the CPU out of the path, the count comes from the handler, and
//! the parity split is the vector's rather than the registry's.
//!
//! Two runs, byte-identical: the record carries no sampled value, so there is nothing for the board to vary.
//!
//! ## Record
//!
//! ```text
//! gpioirq: n=8 if=0x00000008 cnt=8 off=0 dis=0 msk=0 odd=8 even=0 pin0=8 p0odd=0 p0even=8 a=0 up=4 sel=0x00002000 isel=0x00003000
//! ```
//!
//! `sel`/`isel` are the two select words read back for channel 3 of port C: `0x2000` and `0x3000`, the same encoding
//! the PRS measurement found.
//!
//! ## What it does not prove
//!
//! Nothing about a pad driven by a wire at speed, and nothing about latency: the pull moves the pad in microseconds and
//! the window is spaced at 500 us, so this says which edges are delivered, not how fast the handler runs after one.
//! `IEN` is written here as a per-pin gate; the measurement confirms that clearing it silences the pin, and does not
//! say whether it also gates the flag (phase E keeps it set, so the two are not separated).
//!
//! Record: `docs/04-reference/GPIO-INTERRUPT-2026-11-27.md`.
//!
//! VERIFIED ON HARDWARE (2026-11-27).

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use efr32mg2x_hal as _;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::extint::{self, EdgeCounter, Parity};
use efr32mg2x_hal::pin::{Input, Port};
use efr32mg2x_hal::record::Record;
use embedded_hal::delay::DelayNs;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Pad changes per treated window. Eight is enough that one stray edge does not look like a working channel and small
/// enough that a lost edge is obvious.
const TOGGLES: u32 = 8;

/// Time between two pad writes. The pull slews a bare pad in microseconds; this is a hundred times that, so a window
/// can only fail for a reason other than the handler not having run yet.
const SPACING_US: u32 = 500;

/// The pin under test: PC3, channel 3, which the odd vector carries.
const TEST_PIN: u8 = 3;

/// The second pin: PA0, channel 0, which the even vector carries.
const OTHER_PIN: u8 = 0;

/// The two vectors. Without these the weak default handler loops for ever on the first edge, so they are the whole
/// contract [`extint`] asks of the application.
#[no_mangle]
pub extern "C" fn GPIO_ODD_IRQHandler() {
    extint::service(Parity::Odd);
}

#[no_mangle]
pub extern "C" fn GPIO_EVEN_IRQHandler() {
    extint::service(Parity::Even);
}

/// What one window measured.
struct Window {
    /// Edges the handler counted.
    count: u32,
    /// Times the pad was seen at a different level than after the previous write.
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

    let tested = board.input_on(Port::C, TEST_PIN);
    let counter = board.count_edges(Port::C, TEST_PIN, true, true);
    let other = board.input_on(Port::A, OTHER_PIN);
    let other_counter = board.count_edges(Port::A, OTHER_PIN, true, true);

    // A: armed and undisturbed. The counter must not move on its own.
    counter.arm();
    let a = window(&board, &tested, &counter, 0);
    counter.disarm();

    // B: the real thing. Both edges armed, both masks on, eight writes.
    counter.arm();
    let (even_0, odd_0) = extint::entries();
    let b = window(&board, &tested, &counter, TOGGLES);
    let (even_1, odd_1) = extint::entries();
    counter.disarm();
    let odd_b = odd_1 - odd_0;
    let even_b = even_1 - even_0;

    // C: the same eight writes with both edge masks off. Nothing should reach the flag, so nothing is counted.
    counter.set_edges(false, false);
    counter.arm();
    let c = window(&board, &tested, &counter, TOGGLES);
    counter.disarm();
    counter.set_edges(true, true);

    // D: edges armed, but the pin is not allowed to request an interrupt.
    counter.reset();
    let d = window(&board, &tested, &counter, TOGGLES);

    // E: the pin requests, the vector is masked in the NVIC. The flag latches and stays; the counter does not move.
    counter.arm();
    counter.mask_vector();
    let e = window(&board, &tested, &counter, TOGGLES);
    let if_latched = counter.flags();
    counter.reset();
    counter.unmask_vector();
    counter.disarm();

    // F: the even half of the parity split, on a second pad.
    other_counter.arm();
    let (even_2, odd_2) = extint::entries();
    let f = window(&board, &other, &other_counter, TOGGLES);
    let (even_3, odd_3) = extint::entries();
    other_counter.disarm();
    let even_f = even_3 - even_2;
    let odd_f = odd_3 - odd_2;

    let (sel, isel) = counter.selection();

    let mut t = Record::<200>::new();
    t.s(b"gpioirq: n=");
    t.u(b.moved);
    t.s(b" if=0x");
    t.h32(if_latched);
    t.s(b" cnt=");
    t.u(b.count);
    t.s(b" off=");
    t.u(c.count);
    t.s(b" dis=");
    t.u(d.count);
    t.s(b" msk=");
    t.u(e.count);
    t.s(b" odd=");
    t.u(odd_b);
    t.s(b" even=");
    t.u(even_b);
    t.s(b" pin0=");
    t.u(f.count);
    t.s(b" p0odd=");
    t.u(odd_f);
    t.s(b" p0even=");
    t.u(even_f);
    t.s(b" a=");
    t.u(a.count);
    t.s(b" up=");
    t.u(b.rising);
    t.s(b" sel=0x");
    t.h32(sel);
    t.s(b" isel=0x");
    t.h32(isel);

    // The denominator is the pad, not the loop: `b.moved` is what the input register reported, and the count is
    // compared to it. The flag must have latched in E — that is the edge reaching the block with the vector masked —
    // and each vector's entry count must equal the edges it served, which is the parity claim measured.
    let pass = b.moved == TOGGLES
        && b.rising >= 1
        && b.count == b.moved
        && a.count == 0
        && c.count == 0
        && d.count == 0
        && e.count == 0
        && if_latched & (1u32 << TEST_PIN) != 0
        && odd_b == b.count
        && even_b == 0
        && f.moved >= 1
        && f.count == f.moved
        && even_f == f.count
        && odd_f == 0
        && sel == 0x0000_2000
        && isel == 0x0000_3000;
    if pass {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}

/// Move the pad `toggles` times, then read the counter the handler kept.
fn window(board: &Board, input: &Input<'_>, counter: &EdgeCounter<'_>, toggles: u32) -> Window {
    let mut moved = 0u32;
    let mut rising = 0u32;
    let mut level = input.is_high();
    for i in 0..toggles {
        // Odd writes pull up, even ones pull down: every write moves the pad.
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
    }

    // A quiet window still waits as long as a treated one, and every window waits once more after its last write: a
    // phase has to end with the handler having had a chance to run, not with the CPU reading a count mid-flight.
    let settle_us = if toggles == 0 {
        TOGGLES * SPACING_US
    } else {
        SPACING_US
    };
    let mut settle = board.delay();
    settle.delay_us(settle_us);

    Window {
        count: counter.count(),
        moved,
        rising,
    }
}
