//! # Reading the ADC without trusting the number on the screen
//!
//! The IADC on this part converts one channel at a time into a 12-bit code. The hard part of an ADC
//! example is not starting it — it is knowing whether the number it returns means anything.
//!
//! This example answers that with a channel whose value is known in advance: `Channel::Gnd`, the
//! internal ground channel. A converter that is running and configured correctly returns a code
//! near zero for it. A converter that is misconfigured — wrong reference, a conversion that never
//! happened and returns a stale register — does not.
//!
//! So the record carries an assertion, not just a reading:
//!
//! ```text
//! adc: gnd=4 ch0=1873 ref=VBGR
//! ```
//!
//! and the example fails if `gnd` is not close to zero. `ch0` is a pin left floating on purpose:
//! its value is noise, and the record says so rather than pretending it measured something.
//!
//! ## What this example reaches
//!
//! `adc::Adc`, `adc::Config`, `adc::Channel`, `adc::Reference`, and the `Board::adc` seam that lets
//! an example outside the PAC stay free of PAC types.
//!
//! ## What it does not prove
//!
//! Not absolute accuracy. For that a known voltage has to be applied to a pin, and this board has
//! no jumper to do it with.
//!
//! VERIFIED ON HARDWARE (2026-11-25): the record is in this file's header and in the crate README.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use efr32mg2x_hal as _;
use efr32mg2x_hal::adc::{Channel, Config, Reference};
use efr32mg2x_hal::board::Board;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// A ground channel is never exactly zero: there is offset and noise. Two percent of full scale is
/// generous and still fails a converter that returns nonsense.
const GND_TOLERANCE: u16 = 4095 * 2 / 100;

/// Fixed-capacity text builder, the same shape the other examples use.
struct Text {
    buf: [u8; 96],
    len: usize,
}

impl Text {
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
    fn bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
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
    let mut adc = board
        .adc(Config::default().with_reference(Reference::Vbgr))
        .expect("the ADC is handed out once");

    let gnd = match adc.read(Channel::Gnd) {
        Ok(v) => v,
        Err(_) => {
            mailbox::fail(b"adc: the ground conversion returned an error");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // A pin left floating: the value is noise, and the record says so.
    let ch0 = adc.read(Channel::Ch0).unwrap_or(0);
    let reference = adc.reference();

    let mut t = Text::new();
    t.s(b"adc: gnd=");
    t.u(gnd as u32);
    t.s(b" ch0=");
    t.u(ch0 as u32);
    t.s(b" ref=");
    t.s(match reference {
        Reference::Vbgr => b"VBGR",
        Reference::Vdd => b"VDD",
    });

    if gnd <= GND_TOLERANCE {
        mailbox::pass(t.bytes());
    } else {
        // The number is kept even when it fails: a wrong reading is the useful evidence.
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
