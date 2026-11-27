//! All three SPI masters on the same pads, one after the other, and each with its own control.
//!
//! ## Why this exists
//!
//! `spi::Spi0` (USART0) and `spi::Spi2` (EUSART1) were public, documented and **unbuildable**: their only constructors
//! took a `&FrozenClocks`, and the `Board` keeps the CMU private, so no hand-out could produce one. That is the same
//! defect `timer` and `eusart` each had, found by asking which concrete drivers still exist and who constructs them.
//! `Board::spi0_on` and `Board::spi2_on` close it, and this instrument is what turns "it builds" into a record.
//!
//! ## One jumper, three blocks
//!
//! All three are routed to the board's own SPI pads — D10 = PA5, D9 = PA4, D8 = PA3 — so the jumper between MOSI and
//! MISO that `hal_spi_loopback` fits is the same jumper for all three. Each block has its own route registers, and
//! the pads are re-routed per block before its transfer, so nothing is shared but the wire.
//!
//! ## The control, per block
//!
//! A block that echoes its own bytes without a jumper would prove nothing: a floating MISO reads `0xff` on this part,
//! which matches exactly one byte of the pattern. So after each block's loopback run, MISO is re-routed to a pad the
//! jumper does not reach and the same eight bytes are sent again; the record counts the matches, and a block that
//! reports eight in the second run has been reading something other than its own wire.
//!
//! ## Bench state, 2026-11-27: no record yet, and the reason is physical
//!
//! The first run read `spi3: n=8 eusart0=1 usart0=1 eusart1=1 control=1` — one byte matched everywhere, which is what a
//! floating MISO looks like against this pattern (it carries one `0xff`). The proven instrument was re-flashed to
//! settle it: `hal_spi_loopback`, whose record on 2026-11-25 was `PASS spi: n=8 ok=8 ... in=a5..34` with a jumper between
//! D9 and D10, now reports `FAIL spi: n=8 ok=1 err=0 in=00..00`. **The jumper is not fitted**, so nothing drives MISO
//! and all four runs here agree with each other and with the wire being absent.
//!
//! This instrument therefore carries **no record yet**. Fitting a jumper between D9 (MISO) and D10 (MOSI) is a bench
//! action, not a code one, and until it is done three masters cannot be told apart from three unrouted pads.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use efr32mg2x_hal as _;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::pin::Port;
use efr32mg2x_hal::record::Record;
use efr32mg2x_hal::spi::{Config, Mode};
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Eight distinct bytes, both rails included: a line that drives nothing reads `0xff` here, which is also one byte of
/// this pattern, so the verdict counts all eight rather than looking at the first.
const PATTERN: [u8; 8] = [0xA5, 0x5A, 0x00, 0xFF, 0x0F, 0xF0, 0x12, 0x34];

/// 1 MHz: slow enough that a jumper on a breadboard does not matter, fast enough that the divider maths is exercised.
const FREQUENCY_HZ: u32 = 1_000_000;

/// The jumpered pad, and one the jumper cannot reach for the control.
const MISO: (Port, u8) = (Port::A, 4); // D9
const MISO_OFF: (Port, u8) = (Port::A, 6); // not wired to anything in this run
const MOSI: (Port, u8) = (Port::A, 5); // D10
const SCLK: (Port, u8) = (Port::A, 3); // D8

fn matches(rx: &[u8; PATTERN.len()]) -> u32 {
    let mut n = 0u32;
    for (i, esperado) in PATTERN.iter().enumerate() {
        if rx[i] == *esperado {
            n += 1;
        }
    }
    n
}

#[entry]
fn main() -> ! {
    mailbox::invalidate();
    let board = match Board::take() {
        Some(board) => board,
        None => halt(),
    };
    let config = Config::new(Mode::Mode0, FREQUENCY_HZ);

    // --- Spi1, EUSART0: the block that already had a record, re-measured here so the three are comparable ---
    let mut spi1 = match board.spi_on(MOSI, MISO, SCLK, config) {
        Ok(spi) => spi,
        Err(_) => {
            mailbox::fail(b"spi3: EUSART0 refused");
            halt();
        }
    };
    let mut rx1 = [0u8; PATTERN.len()];
    let err1 = spi1.transfer(&mut rx1, &PATTERN).is_err();
    let ok1 = matches(&rx1);

    // --- Spi0, USART0: the first of the two that could not be built ---
    let mut spi0 = match board.spi0_on(MOSI, MISO, SCLK, config) {
        Ok(spi) => spi,
        Err(_) => {
            mailbox::fail(b"spi3: USART0 refused");
            halt();
        }
    };
    let mut rx0 = [0u8; PATTERN.len()];
    let _ = spi0.transfer(&mut rx0, &PATTERN);
    let ok0 = matches(&rx0);

    // --- Spi2, EUSART1: the second ---
    let mut spi2 = match board.spi2_on(MOSI, MISO, SCLK, config) {
        Ok(spi) => spi,
        Err(_) => {
            mailbox::fail(b"spi3: EUSART1 refused");
            halt();
        }
    };
    let mut rx2 = [0u8; PATTERN.len()];
    let _ = spi2.transfer(&mut rx2, &PATTERN);
    let ok2 = matches(&rx2);

    // --- The control: the same EUSART0 driver, with MISO re-routed to a pad the jumper does not reach. Routing is not
    // a claim: a second `spi_on` is refused by design, and re-pointing a pin is exactly what the route registers are
    // for. The transfer is the same eight bytes. ---
    if board.route_eusart0_spi(MOSI, MISO_OFF, SCLK).is_err() {
        mailbox::fail(b"spi3: the control could not re-route MISO");
        halt();
    }
    let mut rxc = [0u8; PATTERN.len()];
    let _ = spi1.transfer(&mut rxc, &PATTERN);
    let control = matches(&rxc);

    let mut l = Record::<160>::new();
    l.s(b"spi3: n=");
    l.u(PATTERN.len() as u32);
    l.s(b" eusart0=");
    l.u(ok1);
    l.s(b" usart0=");
    l.u(ok0);
    l.s(b" eusart1=");
    l.u(ok2);
    l.s(b" control=");
    l.u(control);
    l.s(b" err=");
    l.u(err1 as u32);

    // All three masters echo the whole pattern, and the unjumped control matches at most one byte (a floating MISO
    // reads `0xff`, which is one byte of this pattern -- so "at most one" is the honest limit, not "zero").
    let n = PATTERN.len() as u32;
    let passed = ok1 == n && ok0 == n && ok2 == n && control <= 1;
    if passed {
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
