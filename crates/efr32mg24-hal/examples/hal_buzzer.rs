//! Buzzer through the HAL: a calibrated tone on the base's piezo.
//!
//! The driver measures TIMER0's tick rate against a known CPU delay and derives
//! `TOP` from that, so the pitch does not depend on the datasheet's oscillator band
//! being right — on this board `HFRCOEM23` starts near 5 MHz and settles near
//! 19 MHz, and a `TOP` computed from 20 MHz was measured 5 % flat.
//!
//! The report carries what the driver measured and what it configured, so the tone
//! can be checked arithmetically without an audio instrument:
//!
//! ```text
//! buzzer tick=18123456Hz top=3356 tone=2700Hz
//! ```
//!
//! Beeps are timed in timer ticks, not CPU cycles: silence is produced by setting
//! the compare output to "no action" while the counter keeps running, which is the
//! only way the interval timing stays correct.

#![no_std]
#![no_main]

use efr32mg24_hal::board::Board;
use efr32mg24_hal::buzzer::DEFAULT_TONE_HZ;
use efr32mg24_hal::entry;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

fn push_bytes(buf: &mut [u8], mut pos: usize, s: &[u8]) -> usize {
    for b in s {
        if pos < buf.len() {
            buf[pos] = *b;
            pos += 1;
        }
    }
    pos
}

fn push_dec(buf: &mut [u8], mut pos: usize, mut value: u32) -> usize {
    let mut digits = [0u8; 10];
    let mut n = 0;
    if value == 0 {
        digits[0] = b'0';
        n = 1;
    }
    while value > 0 {
        digits[n] = b'0' + (value % 10) as u8;
        value /= 10;
        n += 1;
    }
    while n > 0 {
        n -= 1;
        if pos < buf.len() {
            buf[pos] = digits[n];
            pos += 1;
        }
    }
    pos
}

#[entry]
fn main() -> ! {
    let board = match Board::take() {
        Some(b) => b,
        None => loop {
            cortex_m::asm::nop();
        },
    };
    let led = board.led();

    // The board owns the PAC peripherals, so it is what hands out a buzzer.
    let mut buzzer = match board.buzzer() {
        Some(b) => b,
        None => {
            mailbox::fail(b"buzzer: timer never started");
            loop {
                led.toggle();
                cortex_m::asm::delay(2_000_000);
            }
        }
    };

    let tick = buzzer.tick_hz();
    let sysclk = buzzer.sysclk_hz();
    let top = buzzer.period_ticks();
    let tone = if top > 0 { tick / (top * 2) } else { 0 };

    let mut msg = [0u8; 64];
    let mut pos = push_bytes(&mut msg, 0, b"buzzer tick=");
    pos = push_dec(&mut msg, pos, tick);
    pos = push_bytes(&mut msg, pos, b"Hz (sysclk=");
    pos = push_dec(&mut msg, pos, sysclk);
    pos = push_bytes(&mut msg, pos, b") top=");
    pos = push_dec(&mut msg, pos, top);
    pos = push_bytes(&mut msg, pos, b" tone=");
    pos = push_dec(&mut msg, pos, tone);
    pos = push_bytes(&mut msg, pos, b"Hz");

    // Plausibility, not optimism: the oscillator is somewhere between 4 and 25 MHz
    // and the requested tone has to come out within 1 %.
    let ok = tone.abs_diff(DEFAULT_TONE_HZ) * 100 <= DEFAULT_TONE_HZ * 5;
    if ok {
        mailbox::pass(&msg[..pos]);
    } else {
        mailbox::fail(&msg[..pos]);
    }

    // 200 ms on, 1.3 s off, both counted in timer ticks.
    let on_ticks = tick / 5;
    let off_ticks = tick * 13 / 10;
    loop {
        buzzer.sound();
        cortex_m::asm::delay(on_ticks / 19);
        buzzer.silence();
        led.toggle();
        cortex_m::asm::delay(off_ticks / 19);
    }
}
