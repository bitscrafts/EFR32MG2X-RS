//! The Sense microphone, through the module crate alone — no Expansion Base.
//!
//! The analog MEMS microphone on the Sense module is not PDM and is not on a bus: it is an
//! analog signal (`PC9`) read by `IADC0`, with a separate enable pin (`PC8`). That is a
//! hardware fact from Seeed's own MG24 backend, and it is the opposite of the usual
//! assumption about a digital microphone.
//!
//! What this example proves:
//!
//! - `Module::microphone()` powers the microphone and hands back a configured converter;
//! - the converter's `SINGLE` conversion sequence completes (it is a WSYNC register: it may
//!   only be written while the converter is enabled, and this is the bug that cost a session);
//! - a quiet room reads a small RMS with a larger peak, and a clap or a word raises both —
//!   the ratio is the point, not the absolute value.
//!
//! The LED follows the level, as the vendor's own demo does, so the board shows it is alive
//! without a display.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use xiao_mg24_bsp::hal::prelude::*;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

fn push_dec(out: &mut [u8], mut at: usize, mut value: u32) -> usize {
    let mut digits = [0u8; 10];
    let mut n = 0;
    loop {
        digits[n] = b'0' + (value % 10) as u8;
        value /= 10;
        n += 1;
        if value == 0 || n == digits.len() {
            break;
        }
    }
    for i in (0..n).rev() {
        out[at] = digits[i];
        at += 1;
    }
    at
}

#[entry]
fn main() -> ! {
    let mut module = Module::take().expect("peripherals taken twice");
    let mut delay = module.delay();
    let led = module.led();

    let mic = match module.microphone() {
        Ok(mic) => mic,
        Err(_) => {
            mailbox::write(
                mailbox::Status::Fail,
                b"mic: converter will not start (clock?)",
            );
            loop {
                led.toggle();
                delay.delay_ms(500);
            }
        }
    };

    // One window: 128 single conversions, read through the driver's level helper.
    let window = mic.level(128);

    let mut buf = [0u8; 64];
    let mut at = 4;
    buf[..6].copy_from_slice(b"mic ");
    buf[at..at + 4].copy_from_slice(b"rms=");
    at += 4;
    at = push_dec(&mut buf, at, u32::from(window.rms));
    buf[at..at + 4].copy_from_slice(b" pk=");
    at += 4;
    at = push_dec(&mut buf, at, u32::from(window.peak));
    buf[at..at + 4].copy_from_slice(b" n=");
    at += 3;
    at = push_dec(&mut buf, at, u32::from(window.samples));

    // A quiet room is a small number; a dead microphone is zero. Gravity has no analogue
    // here, so the invariant is "something moved".
    let ok = window.rms > 0 && window.peak >= window.rms;
    mailbox::write(
        if ok {
            mailbox::Status::Pass
        } else {
            mailbox::Status::Fail
        },
        &buf[..at],
    );

    loop {
        // Follow the level on the LED, as the vendor's own demo does: brighter when loud.
        let window = mic.level(128);
        let on_for = (window.rms / 8).min(100);
        led.on();
        delay.delay_ms(on_for.max(1));
        led.off();
        delay.delay_ms(100 - on_for.min(100));
    }
}
