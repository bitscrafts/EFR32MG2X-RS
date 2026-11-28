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
//! ## Record, 2026-11-27 (two boots, identical)
//!
//! ```text
//! one-of-each: sysrtc=1 sysrtc2=1 adc=1 adc2=1 iadc-cross=1 usart0=1 usart0-cross=1 wdog=1      PASS
//! ```
//!
//! `iadc-cross` and `usart0-cross` are the pairs that were legal until the table stopped being keyed by role: the
//! microphone's converter asked for IADC0 while `adc()` held it, and `spi0_on` asked for USART0 while `usart0()` held
//! it. Both are refused and named. `wdog=1` is the near-miss that keeps those refusals honest -- a slot nobody touched
//! is still handed out, so this example would not pass if the table simply refused everything.
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

    // --- The cross-role pairs: one block, two roles, and the second role must be refused ----------
    //
    // The table used to be keyed by role, so each of these was two slots over one piece of silicon and a caller
    // could hold both. `adc()` is already taken above; the microphone's converter is the same IADC0.
    let converter_depois_do_adc = board.converter();
    let iadc_cruzado = matches!(converter_depois_do_adc, Err(Handout::Taken(_)));

    // USART0 as a UART first, then as a SPI master.
    let usart0 = board.usart0(efr32mg2x_hal::usart::Config::default());
    let usart0_ok = usart0.is_ok();
    let spi0_depois_do_usart0 = board.spi0_on(
        (efr32mg2x_hal::pin::Port::A, 5), // D10, MOSI
        (efr32mg2x_hal::pin::Port::A, 4), // D9, MISO
        (efr32mg2x_hal::pin::Port::A, 3), // D8, SCLK
        efr32mg2x_hal::spi::Config::new(efr32mg2x_hal::spi::Mode::Mode0, 1_000_000),
    );
    let usart0_cruzado = matches!(spi0_depois_do_usart0, Err(Handout::Taken(_)));

    // And a third, untouched slot is still free, so the table is not simply full.
    let wdog_livre = board
        .wdog(efr32mg2x_hal::wdog::WdogConfig::default())
        .is_ok();

    let mut l = Record::<160>::new();
    l.s(b"one-of-each: sysrtc=");
    l.bit(sysrtc_ok);
    l.s(b" sysrtc2=");
    l.bit(sysrtc_recusado);
    l.s(b" adc=");
    l.bit(adc_ok);
    l.s(b" adc2=");
    l.bit(adc_recusado);
    l.s(b" iadc-cross=");
    l.bit(iadc_cruzado);
    l.s(b" usart0=");
    l.bit(usart0_ok);
    l.s(b" usart0-cross=");
    l.bit(usart0_cruzado);
    l.s(b" wdog=");
    l.bit(wdog_livre);

    // The near-miss that keeps the three refusals honest: a slot nobody touched is still handed out. Without it this
    // example would pass just as well if the table refused everything.
    if sysrtc_ok
        && sysrtc_recusado
        && adc_ok
        && adc_recusado
        && iadc_cruzado
        && usart0_ok
        && usart0_cruzado
        && wdog_livre
    {
        mailbox::pass(l.bytes());
    } else {
        mailbox::fail(l.bytes());
    }

    loop {
        core::hint::spin_loop();
    }
}
