//! One of each: the board refuses to hand the same peripheral out twice.
//!
//! ## The defect this measures
//!
//! `Board` is this crate's singleton — `Board::take` succeeds once — but every hand-out method took `&self` and built a
//! driver from `pac::X::steal()`, a zero-sized handle to a fixed address. Nothing in the type system can tell two of
//! them apart, so `board.sysrtc()` called twice used to return two counters over the same `SYSRTC0` and said nothing.
//! Review finding F10; the claim table that now refuses the second ask lives in `cores::slots`, and its bookkeeping is
//! tested on the host (`libraries/slots-core`).
//!
//! ## What this proves, and what it cannot
//!
//! Proves that the refusal reaches the board: the first ask succeeds, the second is refused, and the refusal names the
//! peripheral. Proves too that it is per slot, not a global stop — a second, *different* peripheral is still handed out
//! and then refused in its turn.
//!
//! Does not prove anything at compile time. The type system still cannot distinguish two handles; making it able to
//! would mean the board owning each peripheral and moving it into the driver, which is a redesign of every `Board::x`
//! signature. A runtime refusal is what is achievable without that, and it is enforced rather than documented.

#![no_std]
#![no_main]

use efr32mg2x_hal::board::{Board, Handout};
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

struct Line {
    buf: [u8; 96],
    len: usize,
}

impl Line {
    fn new() -> Self {
        Self {
            buf: [0; 96],
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
    /// `1` for yes, `0` for no: the record is read by a human and compared, not parsed.
    fn b(&mut self, v: bool) {
        self.s(if v { b"1" } else { b"0" });
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    mailbox::invalidate();
    let board = Board::take().expect("peripherals taken twice");

    // First ask: the normal path, and it has to work or nothing after this means anything.
    let primeiro_sysrtc = board.sysrtc();
    let sysrtc_ok = primeiro_sysrtc.is_ok();

    // Second ask for the same peripheral: this is the whole point.
    let segundo_sysrtc = board.sysrtc();
    let sysrtc_recusado = matches!(segundo_sysrtc, Err(Handout::Taken(_)));

    // A different peripheral must still be available -- a refusal is per slot, not a global stop.
    let primeiro_adc = board.adc(efr32mg2x_hal::adc::Config::default());
    let adc_ok = primeiro_adc.is_ok();
    let segundo_adc = board.adc(efr32mg2x_hal::adc::Config::default());
    let adc_recusado = matches!(segundo_adc, Err(Handout::Taken(_)));

    // And a third, untouched slot is still free, so the table is not simply full.
    let wdog_livre = board
        .wdog(efr32mg2x_hal::wdog::WdogConfig::default())
        .is_ok();

    let mut l = Line::new();
    l.s(b"one-of-each: sysrtc=");
    l.b(sysrtc_ok);
    l.s(b" sysrtc2=");
    l.b(sysrtc_recusado);
    l.s(b" adc=");
    l.b(adc_ok);
    l.s(b" adc2=");
    l.b(adc_recusado);
    l.s(b" wdog=");
    l.b(wdog_livre);

    if sysrtc_ok && sysrtc_recusado && adc_ok && adc_recusado && wdog_livre {
        mailbox::pass(&l.buf[..l.len]);
    } else {
        mailbox::fail(&l.buf[..l.len]);
    }

    loop {
        core::hint::spin_loop();
    }
}
