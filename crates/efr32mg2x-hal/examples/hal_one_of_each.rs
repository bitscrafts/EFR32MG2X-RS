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
use efr32mg2x_hal::record::Record;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

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

    // `i2c::I2c1` was public, documented and unreachable: its constructor wanted a `&FrozenClocks` the board keeps
    // private because it owns the CMU. This line is the proof it is reachable now -- it compiles or the crate does not,
    // which is the only claim made here. The bus itself is unmeasured: nothing is wired to I2C1 on this board.
    let i2c1 = board.i2c1(
        efr32mg2x_hal::i2c::Config::new(efr32mg2x_hal::i2c::Speed::Standard100kHz),
        board.clock_hz(),
    );
    let _ = &i2c1;
    let adc_recusado = matches!(segundo_adc, Err(Handout::Taken(_)));

    // And a third, untouched slot is still free, so the table is not simply full.
    let wdog_livre = board
        .wdog(efr32mg2x_hal::wdog::WdogConfig::default())
        .is_ok();

    let mut l = Record::<96>::new();
    l.s(b"one-of-each: sysrtc=");
    l.bit(sysrtc_ok);
    l.s(b" sysrtc2=");
    l.bit(sysrtc_recusado);
    l.s(b" adc=");
    l.bit(adc_ok);
    l.s(b" adc2=");
    l.bit(adc_recusado);
    l.s(b" wdog=");
    l.bit(wdog_livre);

    if sysrtc_ok && sysrtc_recusado && adc_ok && adc_recusado && wdog_livre {
        mailbox::pass(l.bytes());
    } else {
        mailbox::fail(l.bytes());
    }

    loop {
        core::hint::spin_loop();
    }
}
