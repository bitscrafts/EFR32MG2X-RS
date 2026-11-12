//! RTC on the panel: the PCF8563 calendar chip, read and written through the HAL.
//!
//! The Expansion Base carries a PCF8563 at 0x51 with a coin cell behind it, so it
//! keeps time while the MG24 is off. This example does the two things a clock demo
//! has to do to be believable:
//!
//! 1. **reads** the calendar and shows it — one seven-byte transaction, so the
//!    reading cannot straddle a rollover;
//! 2. **detects a clock that lost power.** The chip sets `VL` ("clock integrity not
//!    guaranteed") when its supply dropped; the driver reports that as an error
//!    instead of handing back a plausible-looking wrong time. If `VL` is set, the
//!    example writes a known time once, so the clock is honest from then on.
//!
//! On the panel: date on one line, time on the next. The mailbox carries the first
//! reading and where it came from, so the result is checkable without looking at
//! the board:
//!
//! ```text
//! rtc 2026-11-10 12:00:02 ref-set   <- written from REFERENCE_TIME (the chip had lost power)
//! rtc 2026-10-01 00:14:29 kept      <- the chip's own value, untouched
//! ```
//!
//! `REFERENCE_TIME` is a build-time constant, not a synchronised time: the board has
//! no network. It exists so a power-lost clock is *usable* (monotonic, and clearly
//! marked) instead of returning 2038, which is what an unset PCF8563 reads.

#![no_std]
#![no_main]

use core::cell::RefCell;

use panic_halt as _;
use xiao_expansion_base_bsp::hal::board::Board;
use xiao_expansion_base_bsp::hal::bus::Bus;
use xiao_expansion_base_bsp::hal::bus::SharedBus;
use xiao_expansion_base_bsp::hal::display::{append, Ssd1306};
use xiao_expansion_base_bsp::hal::entry;
use xiao_expansion_base_bsp::hal::pin::Port;
use xiao_expansion_base_bsp::hal::rtc::pcf8563::{DateTime, Pcf8563, RtcError};

#[path = "common/mailbox.rs"]
mod mailbox;

/// A fixed reference time, used only when the chip reports a lost clock.
const REFERENCE_TIME: DateTime = DateTime {
    year: 2026,
    month: 11,
    day: 10,
    weekday: 2,
    hours: 12,
    minutes: 0,
    seconds: 0,
};

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

/// Two digits, zero padded — a clock that prints 9:5:3 is unreadable.
fn push_pad2(buf: &mut [u8], mut pos: usize, value: u32) -> usize {
    if pos < buf.len() {
        buf[pos] = b'0' + ((value / 10) % 10) as u8;
        pos += 1;
    }
    if pos < buf.len() {
        buf[pos] = b'0' + (value % 10) as u8;
        pos += 1;
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
    let led = board.output_on(Port::A, 7, true);

    let base_bus = RefCell::new(board.i2c_on(Bus::new(Port::C, 1 << 4, 1 << 5)));
    let display_bus = RefCell::new(board.i2c_on(Bus::new(Port::C, 1 << 4, 1 << 5)));
    let mut clock = Pcf8563::new(SharedBus::new(&base_bus));
    let mut display = Ssd1306::new(SharedBus::new(&display_bus));

    // If the chip lost power, set it once. Reporting a wrong time silently would be
    // the one thing a clock must never do.
    let mut set_now = false;
    match clock.clock_valid() {
        Ok(true) => {}
        Ok(false) => set_now = true,
        Err(_) => {
            mailbox::fail(b"rtc: 0x51 did not answer");
            loop {
                led.toggle();
                cortex_m::asm::delay(2_000_000);
            }
        }
    }
    // `did_set` is what the report says, and it is set wherever the time is
    // written — including from the error path below. The first version printed
    // "kept" after the chip had in fact been written from the reference, because
    // the flag described the *intent* at boot instead of what happened.
    let mut did_set = false;
    if set_now {
        if clock.set(&REFERENCE_TIME).is_ok() {
            did_set = true;
        }
    }

    let mut reported = false;
    let mut last_second = 255u8;

    loop {
        match clock.read() {
            Ok(when) => {
                if !reported {
                    reported = true;
                    let mut msg = [0u8; 64];
                    let mut pos = push_bytes(&mut msg, 0, b"rtc ");
                    pos = push_dec(&mut msg, pos, u32::from(when.year));
                    pos = push_bytes(&mut msg, pos, b"-");
                    pos = push_pad2(&mut msg, pos, u32::from(when.month));
                    pos = push_bytes(&mut msg, pos, b"-");
                    pos = push_pad2(&mut msg, pos, u32::from(when.day));
                    pos = push_bytes(&mut msg, pos, b" ");
                    pos = push_pad2(&mut msg, pos, u32::from(when.hours));
                    pos = push_bytes(&mut msg, pos, b":");
                    pos = push_pad2(&mut msg, pos, u32::from(when.minutes));
                    pos = push_bytes(&mut msg, pos, b":");
                    pos = push_pad2(&mut msg, pos, u32::from(when.seconds));
                    pos = push_bytes(&mut msg, pos, if did_set { b" ref-set" } else { b" kept" });
                    mailbox::pass(&msg[..pos]);
                }

                if when.seconds != last_second {
                    last_second = when.seconds;
                    if let Some(ref mut d) = display {
                        d.clear();
                        let mut line = [0u8; 24];
                        let mut pos = push_dec(&mut line, 0, u32::from(when.year));
                        pos = push_bytes(&mut line, pos, b"-");
                        pos = push_pad2(&mut line, pos, u32::from(when.month));
                        pos = push_bytes(&mut line, pos, b"-");
                        pos = push_pad2(&mut line, pos, u32::from(when.day));
                        d.text(0, 1, &line[..pos]);

                        let mut line = [0u8; 24];
                        let mut pos = push_pad2(&mut line, 0, u32::from(when.hours));
                        pos = push_bytes(&mut line, pos, b":");
                        pos = push_pad2(&mut line, pos, u32::from(when.minutes));
                        pos = push_bytes(&mut line, pos, b":");
                        pos = push_pad2(&mut line, pos, u32::from(when.seconds));
                        d.text(0, 3, &line[..pos]);

                        d.text(0, 6, b"PCF8563 0x51");
                        d.flush();
                    }
                    led.toggle();
                }
            }
            Err(RtcError::ClockInvalid) => {
                // The chip lost power: its contents are a question mark. Write the
                // build-time reference so the clock is at least monotonic and
                // honest about where the value came from.
                mailbox::fail(b"rtc: clock lost power (VL set), set to reference");
                if clock.set(&REFERENCE_TIME).is_ok() {
                    did_set = true;
                }
            }
            Err(RtcError::Bus(_)) => {
                mailbox::fail(b"rtc: bus error");
            }
            Err(RtcError::Corrupt) => {
                mailbox::fail(b"rtc: corrupt reading");
            }
        }

        // The chip updates once a second; polling faster only wastes bus time.
        cortex_m::asm::delay(400_000);
    }
}
