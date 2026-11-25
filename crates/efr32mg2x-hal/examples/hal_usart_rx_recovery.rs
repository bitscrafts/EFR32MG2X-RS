//! Does the receiver recover after an overrun? It is the property a product depends on.
//!
//! ## The two faults this instrument was built to settle
//!
//! `hal_usart_link` once reported `echo n=271` with nothing on the RX pad: bytes nobody sent. The first fix refused any byte
//! whose `IF` flags said the frame was broken, and the part then showed two things wrong with that fix (2026-11-25):
//!
//! 1. **`IF`'s flags latch and nothing clears them.** Measured: draining the FIFO, a write-1 clear through the secure
//!    window, the same through the non-secure window, a write-zero, an `EN` cycle and `CMD.CLEARRX` -- six mechanisms, each
//!    leaving `RXOF` set at `1f`. A driver that refused on them refused every byte for the rest of the program's life.
//! 2. **The per-byte verdict is not in `IF` at all.** It is in `RXDATAX`, bits 14 and 15: that byte's own parity and
//!    framing error. `read_byte` reads `RXDATAX` now; reading `RXDATA` threw the verdict away with the byte.
//!
//! ## What this measures
//!
//! No external wiring and no jumper, because the noise is made inside the part:
//!
//! 1. **overrun**: the internal loopback is switched on and the banner is written without draining, so the two-deep FIFO
//!    overruns. `overran()` must then report the latch -- the half that says the flags are level latches, not a verdict
//!    about any one byte.
//! 2. **recovery**: the FIFO is drained and six bytes are sent one at a time, each read back before the next. Every one
//!    must be accepted and identical, with an overrun latched a moment earlier.
//!
//! Step 2 is the whole point. A receiver that answers `Err` for ever after one overrun would pass step 1 and fail the
//! product.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use efr32mg2x_hal as _;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::pin::Port;
use efr32mg2x_hal::record::Record;
use efr32mg2x_hal::usart::{Config, Usart0};
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// More bytes than a two-deep FIFO holds, so writing them without reading overruns it.
const BANNER: &[u8] = b"EFR32MG24 USART0 @115200 - overrun\r\n";

/// What comes back after the overrun, one byte at a time.
const PATTERN: &[u8] = b"EFR32!";

#[entry]
fn main() -> ! {
    mailbox::invalidate();

    let board = match Board::take() {
        Some(board) => board,
        None => halt(),
    };
    if board.route_usart0((Port::C, 6), (Port::C, 7)).is_err() {
        mailbox::fail(b"usartrec: route refused");
        halt();
    }
    let mut usart: Usart0 = match board.usart0(Config::default()) {
        Ok(usart) => usart,
        Err(_) => {
            mailbox::fail(b"usartrec: USART0 not handed out");
            halt();
        }
    };

    // Step 1: overrun on purpose. Write, never read.
    usart.set_internal_loopback(true);
    usart.write(BANNER);
    usart.flush();
    let overran = usart.overran();
    let errors_during = usart.rx_errors();

    // Step 2: drain what is left, then send one byte at a time and read it back before the next.
    let mut drained = 0u32;
    while usart.read_byte().is_some() && drained < 64 {
        drained += 1;
    }
    let errors_after_drain = usart.rx_errors();

    let mut ok = 0u32;
    let mut bad = 0u32;
    for &byte in PATTERN {
        usart.write_byte(byte);
        usart.flush();
        match usart.read_byte() {
            Some(Ok(back)) if back == byte => ok += 1,
            _ => bad += 1,
        }
    }

    let mut l = Record::<140>::new();
    l.s(b"usartrec: overran=");
    l.bit(overran);
    l.s(b" err_pre=");
    l.u(errors_during);
    l.s(b" drained=");
    l.u(drained);
    l.s(b" err_drain=");
    l.u(errors_after_drain);
    l.s(b" ok=");
    l.u(ok);
    l.s(b" bad=");
    l.u(bad);

    // Both halves are required: an overrun that did not happen would make the recovery untested, and a recovery that
    // refuses the good bytes is the defect this exists to catch.
    if !overran {
        mailbox::fail(b"usartrec: no overrun was latched, so nothing was recovered from");
    } else if ok == PATTERN.len() as u32 && bad == 0 {
        mailbox::pass(l.bytes());
    } else {
        mailbox::fail(l.bytes());
    }

    halt()
}

fn halt() -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}
