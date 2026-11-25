//! SPI with a wire between MOSI and MISO: eight bytes out, the same eight bytes back.
//!
//! ## The instrument
//!
//! On the XIAO MG24 the board's SPI lives on D10 (PA5, MOSI), D9 (PA4, MISO) and D8 (PA3, SCLK) — the same
//! three pads Seeed's own board support routes for `xiao_spi`. A jumper between **D10 and D9** makes the
//! master talk to itself, which is the only way to watch this bus on a board with no SPI device on it.
//!
//! ## What it proves, and what it does not
//!
//! Proves: the clock reaches a pad, the shifter moves data out on MOSI, the receiver samples MISO, and the
//! divider produces a rate the loopback survives. Proves, too, that the routing is right: with the peripheral
//! unrouted — the state this HAL was in before `Board::spi_on` — the transfer completes and reads back zeros,
//! which is exactly what a dead bus looks like.
//!
//! Does not prove: anything about a real device's timing, chip-select handling (unused here), or a mode other
//! than 0. Those need a part on the other end, and there is none on this board.
//!
//! ## Why the pattern has to be more than zeros
//!
//! A bus that drives nothing reads back `0x00`, and so does a bus that works while transmitting zeros. The
//! pattern below has two distinct bytes and the verdict needs every one of them to come back, so neither the
//! jumper being absent nor a constant line can pass.
//!
//! VERIFIED ON HARDWARE (2026-11-25): `PASS spi: n=8 ok=8 err=0 out=a5..34 in=a5..34` with the jumper fitted,
//! and `FAIL spi: n=8 ok=1 err=0 out=a5..34 in=ff..ff` with MISO moved to a pin that no jumper reaches.

#![no_std]
#![no_main]

use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::pin::Port;
use efr32mg2x_hal::spi::{Config, Mode};
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

/// What goes out, and therefore what must come back.
const PATTERN: [u8; 8] = [0xA5, 0x5A, 0x00, 0xFF, 0x0F, 0xF0, 0x12, 0x34];
/// Slow enough that a jumper wire, not a transmission line, is the model.
const FREQUENCY_HZ: u32 = 1_000_000;

struct Line {
    buf: [u8; 120],
    len: usize,
}

impl Line {
    fn new() -> Self {
        Self {
            buf: [0; 120],
            len: 0,
        }
    }
    fn s(&mut self, text: &[u8]) {
        for b in text {
            if self.len < self.buf.len() {
                self.buf[self.len] = *b;
                self.len += 1;
            }
        }
    }
    fn u(&mut self, mut v: u32) {
        if v == 0 {
            self.s(b"0");
            return;
        }
        let mut digits = [0u8; 10];
        let mut n = 0;
        while v > 0 {
            digits[n] = b'0' + (v % 10) as u8;
            v /= 10;
            n += 1;
        }
        while n > 0 {
            n -= 1;
            let d = digits[n];
            self.s(&[d]);
        }
    }
    /// Two hex digits: a byte reads better as `a5` than as `165` when it is compared with a pattern.
    fn h(&mut self, v: u8) {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        self.s(&[HEX[(v >> 4) as usize], HEX[(v & 0xF) as usize]]);
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    mailbox::invalidate();
    let board = Board::take().expect("peripherals taken twice");

    let mut spi = match board.spi_on(
        (Port::A, 5), // MOSI: D10
        (Port::A, 4), // MISO: D9
        (Port::A, 3), // SCLK: D8
        Config::new(Mode::Mode0, FREQUENCY_HZ),
    ) {
        Ok(spi) => spi,
        Err(_) => {
            mailbox::fail(b"spi: the board refused the pin pair");
            loop {
                core::hint::spin_loop();
            }
        }
    };

    let mut rx = [0u8; PATTERN.len()];
    let transfer = spi.transfer(&mut rx, &PATTERN);

    let mut iguais = 0u32;
    for (i, esperado) in PATTERN.iter().enumerate() {
        if rx[i] == *esperado {
            iguais += 1;
        }
    }

    let mut l = Line::new();
    l.s(b"spi: n=");
    l.u(PATTERN.len() as u32);
    l.s(b" ok=");
    l.u(iguais);
    l.s(b" err=");
    l.u(transfer.is_err() as u32);
    l.s(b" out=");
    l.h(PATTERN[0]);
    l.s(b"..");
    l.h(PATTERN[PATTERN.len() - 1]);
    l.s(b" in=");
    l.h(rx[0]);
    l.s(b"..");
    l.h(rx[rx.len() - 1]);

    // Every byte, and the transfer itself, or nothing.
    if transfer.is_ok() && iguais as usize == PATTERN.len() {
        mailbox::pass(&l.buf[..l.len]);
    } else {
        mailbox::fail(&l.buf[..l.len]);
    }

    loop {
        core::hint::spin_loop();
    }
}
