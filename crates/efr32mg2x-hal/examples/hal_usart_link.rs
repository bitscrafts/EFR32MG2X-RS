//! USART on the header pads: a jumper D6–D7 for the on-board proof, or an adapter for an outside witness.
//!
//! ## Two roles, chosen in the source, never guessed at run time
//!
//! The first version of this example tried to detect which bench it was on and fall back to echoing when the loopback
//! did not close. Measured 2026-11-25, jumper fitted: the loopback was refused, the example entered echo mode, its own
//! banner came straight back on the jumper and was echoed again — the record reached `usart: echo n=4022 last=0a` in
//! seconds, a UART talking to itself for ever. The role is a constant now, and `Role::Echo` is only correct when
//! something outside the board is attached.
//!
//! ## The two ways to watch a UART
//!
//! A transmitter talking to nothing looks exactly like one talking to a listener, so this example is judged from
//! outside itself, in whichever of two ways the bench allows:
//!
//! - **`Role::Loopback`**: a jumper between D6 (PC6, TX) and D7 (PC7, RX). Six bytes go out, the receiver samples them
//!   back, and the verdict compares them one by one. Proves the pins, the routing, the divider, the shifter and the
//!   receiver — everything except that a *different* machine can read it.
//! - **`Role::Echo`**: a USB-serial adapter (`MG24 D6 -> adapter RXD`, `D7 <- adapter TXD`, GND common). The bytes the
//!   adapter reads prove TX and the bytes it sends come back echoed, which proves RX. That is the stronger record,
//!   because the witness is not the board.
//!
//! D6 and D7 are the pads Seeed's own board support exposes as the header serial. Routing them to USART0 is a
//! crossbar choice, not a restriction of the part.
//!
//! ## Measured: a floating RX line transmits
//!
//! 2026-11-25, with D6/D7 left unconnected and the receiver enabled, this example reported `usart: echo n=271 last=00`
//! within seconds: the input, with no pull and nothing driving it, hands back a byte of `0x00` often enough to look
//! like a partner. Any echo test that counts bytes therefore passes on a board with nothing attached. That is why the
//! loopback verdict compares bytes, and why the record says `last=00` when what arrived is noise.
//!
//! ## Why the banner goes out last
//!
//! In the loopback role anything transmitted comes straight back into the receiver, so a banner sent first would fill
//! the comparison with its own bytes. Order: drain, test, then talk.
//!
//! VERIFIED ON HARDWARE (2026-11-25), jumper D6–D7 fitted:
//! `PASS usart: baud=115200 out=6 in=6 ok=6 45 46 52 33 32 21` — `EFR32!` out, and the same six bytes back.

#![no_std]
#![no_main]

use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::pin::Port;
use efr32mg2x_hal::record::Record;
use efr32mg2x_hal::usart::{Config, Usart0};
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

/// What this build of the example is for. A jumper and an adapter are different benches, and an example that tries to
/// tell them apart at run time ends up talking to itself.
const ROLE: Role = Role::Loopback;

/// The pads USART0 is routed to: the board's header serial, D6 (TX) and D7 (RX).
///
/// Tried and rejected, 2026-11-25: routing the same USART0 to `PA8`/`PA9` — the pads Seeed's board support uses for its
/// console, on the theory that the SAMD11 beside the MCU bridges them to the host's `/dev/ttyACM0`. The host read 102
/// bytes of `0xC0`, which is a line held low, not a console: whatever that CDC is for on this board, it is not this
/// UART. The witness has to come from the header.
const PINS: ((Port, u8), (Port, u8)) = ((Port::C, 6), (Port::C, 7));

/// Which bench this build expects.
#[derive(PartialEq)]
enum Role {
    /// D6 joined to D7. The verdict is the comparison.
    Loopback,
    /// Something outside the board on D6/D7. The verdict is what that something reads.
    Echo,
}

/// 8N1 at the rate every adapter defaults to.
const BAUDRATE: u32 = 115_200;
/// The bytes the loopback has to return, in order. Two distinct values, so a dead line returning zeros cannot pass.
const PATTERN: &[u8] = b"EFR32!";
/// How long to give the receiver, in SYSRTC ticks (32.768 kHz): a quarter of a second, which is ~2800 byte times at
/// 115200 and therefore generous by three orders of magnitude.
const WINDOW_TICKS: u32 = 8_192;
/// Text for a listener that attached late.
const BANNER: &[u8] = b"EFR32MG24 USART0 @115200 - echo\r\n";

#[cortex_m_rt::entry]
fn main() -> ! {
    mailbox::invalidate();
    let board = Board::take().expect("peripherals taken twice");
    let rtc = board.sysrtc().expect("SYSRTC is handed out once");

    // Route first, build second: a driver enabled with its pins unrouted drives nothing while looking alive.
    if board.route_usart0(PINS.0, PINS.1).is_err() {
        mailbox::fail(b"usart: the board refused the pin pair");
        halt();
    }

    let mut usart = match board.usart0(Config::new(BAUDRATE)) {
        Ok(usart) => usart,
        Err(_) => {
            mailbox::fail(b"usart: the driver refused the configuration");
            halt();
        }
    };

    if ROLE == Role::Echo {
        return echo(&mut usart);
    }

    // Anything already in the receiver belongs to something that happened before this test.
    while usart.read_byte().is_some() {}

    // One byte at a time, and read each one before sending the next.
    //
    // Measured 2026-11-25, and it is the reason for the shape: sending all six back to back and then reading gave
    // `in=3 ok=2 45 46 21` — `E`, `F` and the final `!`, with everything in between lost. The receiver holds two
    // bytes, so a six-byte burst at 115200 overruns it before the loop can drain it. A request-response per byte
    // cannot overrun, and it is also the honest model of what a UART peer does.
    let mut volta = [0u8; 6];
    let mut recebidos = 0usize;
    let mut iguais = 0usize;
    for esperado in PATTERN {
        usart.write_byte(*esperado);
        usart.flush();

        let t0 = rtc.now();
        let mut chegou = false;
        while rtc.now().wrapping_sub(t0) < WINDOW_TICKS {
            if let Some(byte) = usart.read_byte() {
                volta[recebidos] = byte;
                if byte == *esperado {
                    iguais += 1;
                }
                recebidos += 1;
                chegou = true;
                break;
            }
        }
        // A byte that never came back ends the test: waiting out the remaining windows for a sequence that is already
        // lost only makes the example slower to say what it already knows.
        if !chegou {
            break;
        }
    }

    let mut l = Record::<80>::new();
    l.s(b"usart: baud=");
    l.u(BAUDRATE);
    l.s(b" out=");
    l.u(PATTERN.len() as u32);
    l.s(b" in=");
    l.u(recebidos as u32);
    l.s(b" ok=");
    l.u(iguais as u32);
    // What came back, byte by byte: `in=6 ok=4` and `in=0 ok=0` are different defects, and the record has to be able
    // to tell them apart without the session.
    for i in 0..recebidos {
        l.s(b" ");
        l.h(volta[i]);
    }

    if recebidos == PATTERN.len() && iguais == PATTERN.len() {
        mailbox::pass(l.bytes());
    } else {
        mailbox::fail(l.bytes());
    }

    halt()
}

/// Answer whatever arrives, so a second machine can prove both directions.
fn echo(usart: &mut Usart0) -> ! {
    usart.write(BANNER);
    usart.flush();
    mailbox::pass(b"usart: echo ready");

    let mut eco: u32 = 0;
    loop {
        if let Some(byte) = usart.read_byte() {
            usart.write_byte(byte);
            usart.flush();
            eco += 1;

            let mut l = Record::<80>::new();
            l.s(b"usart: echo n=");
            l.u(eco);
            l.s(b" last=");
            l.h(byte);
            mailbox::pass(l.bytes());
        }
    }
}

fn halt() -> ! {
    loop {
        core::hint::spin_loop();
    }
}
