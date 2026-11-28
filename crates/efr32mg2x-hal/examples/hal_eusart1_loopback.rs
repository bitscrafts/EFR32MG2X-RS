//! EUSART1 in asynchronous mode: eight bytes out and back through `CFG0.LOOPBK`, and a control that reads none.
//!
//! ## Why this needed its own instrument
//!
//! `EUSART1` was the last row of the coverage boundary that read "compiled, not measured", and the reason was in the
//! source rather than the bench: `eusart` held one concrete `Eusart0`, so the second block had no driver to measure. The
//! two blocks are the same hardware at two addresses -- each with its own `CLKEN1` bit, and only `EUSART0` with a clock
//! selector in front of it -- so the driver is now generated once per instance and this is the second instance's record.
//!
//! ## The two runs, and what each one rules out
//!
//! 1. **Control, loopback off.** The same eight bytes go to the transmitter, the receiver is drained, and the record
//!    must show **zero** bytes accepted. Without this run, "eight bytes came back" would not distinguish the internal
//!    loop from a receiver reading its own noise -- this crate has already measured a floating USART line producing 271
//!    bytes of nothing (`hal_usart_link`).
//! 2. **Loopback on.** Eight distinct bytes, written one at a time and drained one at a time, because the receiver's FIFO
//!    is two deep and the transmitter would otherwise outrun it. Every byte must come back identical and the error flags
//!    must stay clear.
//!
//! A third control rides along: a second `board.eusart1_async(...)` must be refused by name, because the claim table is
//! what stands between this crate and two owners of one block.
//!
//! ## What this does not prove
//!
//! * **The pads.** Nothing leaves the block, so this says nothing about a pin, a level on it, or a device at the other
//!   end. That needs a witness and is listed as absent in the boundary.
//! * **The baud on a wire.** The divider is computed and reported, but both ends of a loopback share it, so a rate that is
//!   wrong by 20 % still passes here. `hal_eusart_loopback` times `EUSART0` against the SYSRTC for that reason; this
//!   instrument does not, because `EUSART1` has no clock selector to read back and a ratio measured against an assumed
//!   source would be an assumption wearing a number.
//!
//! VERIFIED ON HARDWARE (2026-11-27): three flashes, three boots, `status=PASS` in all three, byte-identical.

#![no_std]
#![no_main]

use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::entry;
use efr32mg2x_hal::eusart::{Eusart1, IF_FERR, IF_PERR, IF_RXOF};
use efr32mg2x_hal::pin::Port;
use efr32mg2x_hal::record::Record;
use efr32mg2x_hal::sysrtc::Sysrtc;
use efr32mg2x_hal::usart::Config;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

/// What goes out, and therefore what must come back. Distinct values, including both rails: a line that drives nothing
/// reads `0x00`, and so does a working line sending zeros.
const PATTERN: [u8; 8] = [0xA5, 0x5A, 0x00, 0xFF, 0x0F, 0xF0, 0x12, 0x34];

/// The rate asked for. Both ends of the loopback share the divider, so this is about the arithmetic fitting the field,
/// not about a wire.
const BAUD: u32 = 115_200;

/// SYSRTC ticks in one millisecond: the part's 32.768 kHz branch.
const MS_TICKS: u32 = 33;

/// Polls to give one expected byte. A frame is ~1650 cycles at 19 MHz and a poll is a few, so this is far more than one
/// frame and still bounded: a wait that never ends is indistinguishable from a hang.
const RX_POLLS: u32 = 200_000;

/// Read everything the receiver holds, up to `polls` polls. Returns how many bytes it accepted.
fn drain(uart: &mut Eusart1, polls: u32) -> u32 {
    let mut seen = 0u32;
    let mut quiet = 0u32;
    for _ in 0..polls {
        match uart.read_byte() {
            Some(_) => {
                seen += 1;
                quiet = 0;
            }
            None => {
                quiet += 1;
                // Two thousand consecutive empty polls after a flush is far longer than a frame; stop rather than spend
                // the whole budget pretending to look.
                if quiet > 2_000 {
                    break;
                }
            }
        }
    }
    seen
}

#[entry]
fn main() -> ! {
    mailbox::invalidate();

    let board = match Board::take() {
        Some(board) => board,
        None => halt(),
    };
    let rtc = match board.sysrtc() {
        Ok(rtc) => rtc,
        Err(_) => {
            mailbox::fail(b"eusart1: no SYSRTC to time the wait with");
            halt();
        }
    };
    let config = Config {
        baudrate: BAUD,
        ..Config::default()
    };

    let mut uart = match board.eusart1_async(config) {
        Ok(uart) => uart,
        Err(_) => {
            mailbox::fail(b"eusart1: not handed out");
            halt();
        }
    };

    // The claim table's control.
    let refused = board.eusart1_async(config).is_err();

    // The pads, because the loopback on this block closes through the receiver's input path: with `LOOPBK` set and both
    // enables on, an unrouted block sends its bytes into nothing and the receiver never fills. Nothing is wired to
    // either pad; they only have to exist.
    let mut routed = true;
    if board
        .route_eusart1_async((Port::B, 0), (Port::B, 1))
        .is_err()
    {
        routed = false;
    }

    let divider = uart.divider();

    // Run 1, the control: loopback off, the same bytes written, nothing may come back.
    uart.set_internal_loopback(false);
    let _ = uart.write(&PATTERN);
    let _ = uart.flush();
    let control = drain(&mut uart, RX_POLLS / 2);

    // Run 2: loopback on, one byte at a time in each direction, because the receiver's FIFO is two deep.
    uart.set_internal_loopback(true);
    // What the block says it has, not what the instrument asked for: a masked or refused write would otherwise look
    // exactly like a loopback that does not work.
    let cfg0 = uart.cfg0();
    let mut matched = 0u32;
    let mut seen = 0u32;
    // `CFG0.LOOPBK` has to reach the block's own clock domain before the first frame is shifted, exactly like the
    // enables: a loopback written and used in the same microsecond is a loopback nobody has switched on yet.
    settle(&rtc, MS_TICKS);
    for &byte in PATTERN.iter() {
        let _ = uart.write_byte(byte);
        let mut got = None;
        // Ten milliseconds is a hundred bit times at 115 200: far more than one frame, and short enough that a
        // receiver which never fills ends the test instead of hanging it.
        let deadline = rtc.now().wrapping_add(10 * MS_TICKS);
        while rtc.now().wrapping_sub(deadline) > 0x8000_0000 {
            match uart.read_byte() {
                Some(Ok(value)) => {
                    seen += 1;
                    got = Some(value);
                    break;
                }
                Some(Err(_)) => {
                    seen += 1;
                    break;
                }
                None => core::hint::spin_loop(),
            }
        }
        if got == Some(byte) {
            matched += 1;
        }
    }
    let _ = uart.flush();
    // Whatever is still in the FIFO is drained and counted: a receiver that filled late is a different finding from one
    // that never filled, and the record has to be able to tell them apart.
    seen += drain(&mut uart, RX_POLLS / 4);

    let status = uart.status();
    let flags = uart.rx_flags();
    let errors = flags & (IF_RXOF | IF_PERR | IF_FERR);

    let mut l = Record::<200>::new();
    l.s(b"eusart1: n=");
    l.u(PATTERN.len() as u32);
    l.s(b" matched=");
    l.u(matched);
    l.s(b" seen=");
    l.u(seen);
    l.s(b" cfg0=0x");
    l.h32(cfg0);
    l.s(b" st=0x");
    l.h32(status);
    l.s(b" control=");
    l.u(control);
    l.s(b" div=");
    l.u(divider);
    l.s(b" want=");
    l.u(BAUD);
    l.s(b" err=0x");
    l.h32(errors);
    l.s(b" routed=");
    l.bit(routed);
    l.s(b" refused=");
    l.bit(refused);

    // Every field is part of the verdict: all eight bytes identical, the control at zero (a loopback that was already
    // on would make it non-zero), no error flag, a divider that fits, and the second hand-out refused.
    let passed = matched as usize == PATTERN.len()
        && control == 0
        && errors == 0
        && divider > 0
        && refused
        && routed;
    if passed {
        mailbox::pass(l.bytes());
    } else {
        mailbox::fail(l.bytes());
    }

    halt()
}

/// Wait, in SYSRTC ticks, on a clock that is not the one under test.
fn settle(rtc: &Sysrtc, ticks: u32) {
    let start = rtc.now();
    while rtc.now().wrapping_sub(start) < ticks {
        core::hint::spin_loop();
    }
}

fn halt() -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}
