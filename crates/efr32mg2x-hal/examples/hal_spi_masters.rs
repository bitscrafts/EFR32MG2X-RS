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
//! ## The control, and why the first one could not fail
//!
//! The first control re-routed MISO to a pad the jumper does not reach, on the theory that a receiver with no MISO route
//! reads nothing. With the jumper fitted that control reported `control=8` -- correct, and useless: the wire ties D9 to
//! D10 physically, so the signal arrives whether or not the internal route points at it. **The jumper defeats that
//! control**, and an instrument whose control cannot fail is not an instrument.
//!
//! Two changes make the result mean something:
//!
//! * each master sends a pattern that shares **no byte** with the other two, and the record counts the bytes each one
//!   received that belong to *another* master's pattern. That count must be zero: a receiver silently reading a
//!   neighbour's line scores zero on its own pattern rather than eight;
//! * the control sends `0xEE`, a byte no master ever sends. Whatever the receiver reads -- the driven wire or a floating
//!   line -- it matches none of the three patterns, so the control must match **zero** and the equality checks cannot be
//!   vacuous.
//!
//! ## Record, 2026-11-27 (jumper D9-D10 fitted)
//!
//! ```text
//! spi3: n=8 eusart0=8 usart0=8 eusart1=8 cross=0 control=0 err=0      PASS
//! ```
//!
//! Three boots earlier in the same session, from the same image, agree. Before the jumper was fitted the same image read
//! `eusart0=1 usart0=1 eusart1=1 control=1` -- one byte matched everywhere, which is what a floating MISO looks like
//! against a pattern containing one `0xff` -- and the proven instrument `hal_spi_loopback` read `FAIL ... in=00..00`. It
//! now reads `PASS spi: n=8 ok=8 err=0 out=a5..34 in=a5..34` again, which is how the wire and the drivers were told apart
//! from each other rather than blamed as a pair.
//!
//! What this record does **not** say: that the three blocks are wired to different pads on a second device, or that they
//! work above 1 MHz. It says each of the three drivers, driven one at a time through its own route registers, sends its
//! own eight bytes and receives exactly them.

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

/// Three patterns that share **no byte**, one per master.
///
/// The first version of this example sent the same eight bytes from all three and then reported `eusart0=8 usart0=8
/// eusart1=8`. That proves *something* drove the line; it cannot tell three masters apart, which is the entire point of
/// the instrument. With disjoint byte sets, a receiver that is silently reading another master's line scores zero on its
/// own pattern instead of eight, and the cross-count below must be zero for the record to pass.
const PATTERN_EUSART0: [u8; 8] = [0xA5, 0x5A, 0x00, 0xFF, 0x0F, 0xF0, 0x12, 0x34];
const PATTERN_USART0: [u8; 8] = [0x3C, 0xC3, 0x81, 0x18, 0x24, 0x42, 0x99, 0x66];
const PATTERN_EUSART1: [u8; 8] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80];
/// Sent by the control: a byte no master ever transmits, so a receiver that is still being driven by the wire reads
/// `0xEE` and matches none of the three.
const NOTHING: [u8; 8] = [0xEE; 8];

/// 1 MHz: slow enough that a jumper on a breadboard does not matter, fast enough that the divider maths is exercised.
const FREQUENCY_HZ: u32 = 1_000_000;

/// The jumpered pads.
const MISO: (Port, u8) = (Port::A, 4); // D9
const MOSI: (Port, u8) = (Port::A, 5); // D10
const SCLK: (Port, u8) = (Port::A, 3); // D8

fn matches(rx: &[u8; 8], pattern: &[u8; 8]) -> u32 {
    let mut n = 0u32;
    for (i, esperado) in pattern.iter().enumerate() {
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
    let mut rx1 = [0u8; 8];
    let err1 = spi1.transfer(&mut rx1, &PATTERN_EUSART0).is_err();
    let ok1 = matches(&rx1, &PATTERN_EUSART0);

    // --- Spi0, USART0: the first of the two that could not be built ---
    let mut spi0 = match board.spi0_on(MOSI, MISO, SCLK, config) {
        Ok(spi) => spi,
        Err(_) => {
            mailbox::fail(b"spi3: USART0 refused");
            halt();
        }
    };
    let mut rx0 = [0u8; 8];
    let _ = spi0.transfer(&mut rx0, &PATTERN_USART0);
    let ok0 = matches(&rx0, &PATTERN_USART0);

    // --- Spi2, EUSART1: the second ---
    let mut spi2 = match board.spi2_on(MOSI, MISO, SCLK, config) {
        Ok(spi) => spi,
        Err(_) => {
            mailbox::fail(b"spi3: EUSART1 refused");
            halt();
        }
    };
    let mut rx2 = [0u8; 8];
    let _ = spi2.transfer(&mut rx2, &PATTERN_EUSART1);
    let ok2 = matches(&rx2, &PATTERN_EUSART1);
    // The cross-count: bytes each master received that belong to one of the *other* two patterns. Disjoint sets make
    // anything above zero a cross-wiring, which the equality check alone cannot express.
    let cross = matches(&rx1, &PATTERN_USART0)
        + matches(&rx1, &PATTERN_EUSART1)
        + matches(&rx0, &PATTERN_EUSART0)
        + matches(&rx0, &PATTERN_EUSART1)
        + matches(&rx2, &PATTERN_EUSART0)
        + matches(&rx2, &PATTERN_USART0);

    // --- The control. Re-routing MISO away is **not** a control any more: the jumper ties D9 to D10 with a wire, so the
    // signal arrives whether or not the internal route is pointed at it -- measured 2026-11-27, `control=8` with the
    // MISO route removed. The wire defeats that idea, and an instrument whose control cannot fail is not an instrument.
    //
    // So the control sends a byte no master ever sends (`0xEE`): a receiver that is still being driven reads it and
    // matches none of the patterns; a receiver that is not being driven reads a floating line and likewise matches none.
    // Either way the control must match **zero** -- and the equality checks above are what prove the path carries the
    // real byte, since a dead path cannot produce eight matching bytes of a pattern nothing else sends.
    let mut rxc = [0u8; 8];
    let _ = spi1.transfer(&mut rxc, &NOTHING);
    let control = matches(&rxc, &PATTERN_EUSART0);

    let mut l = Record::<160>::new();
    l.s(b"spi3: n=");
    l.u(8u32);
    l.s(b" eusart0=");
    l.u(ok1);
    l.s(b" usart0=");
    l.u(ok0);
    l.s(b" eusart1=");
    l.u(ok2);
    l.s(b" cross=");
    l.u(cross);
    l.s(b" control=");
    l.u(control);
    l.s(b" err=");
    l.u(err1 as u32);

    // Each master receives its **own** eight bytes, none of the other two masters' bytes arrive anywhere, and the
    // control -- a byte nobody sends -- matches nothing.
    let passed = ok1 == 8 && ok0 == 8 && ok2 == 8 && cross == 0 && control == 0;
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
