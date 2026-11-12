//! bsp-env-clock: the whole Expansion Base, used through the BSP only.
//!
//! This example is the test of the facade: it never names an I2C address, a pin, a
//! bus or a PAC register. It asks the board for its panel, its environment sensor
//! and its clock, and puts them on screen together.
//!
//! ```text
//!   ExpansionBase::take()   the board, with both buses and both power rails
//!   .probe()                which devices answered (OLED 0x3C, RTC 0x51, BME68x)
//!   .oled()                 the 128x64 panel
//!   .environment()          temperature, pressure, humidity
//!   .rtc()                  the calendar chip
//! ```
//!
//! The mailbox gets one line per boot with what answered and the first readings, so
//! the result is checkable without looking at the board.
//!
//! Why the BSP exists at all: the drivers know how to talk to a BME68x, but only the
//! board knows that *this* BME68x sits at 0x76 on PC04/PC05 behind the base's
//! pull-ups, and that the IMU is somewhere else entirely. That knowledge is here.

#![no_std]
#![no_main]

use panic_halt as _;
use xiao_expansion_base_bsp::hal::display::{append, format_fixed};
use xiao_expansion_base_bsp::hal::entry;
use xiao_expansion_base_bsp::ExpansionBase;

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
    let board = match ExpansionBase::take() {
        Some(b) => b,
        None => loop {
            cortex_m::asm::nop();
        },
    };
    let led = board.led();

    // What is actually on the base?
    let devices = board.probe();
    let mut msg = [0u8; 64];
    let mut pos = push_bytes(&mut msg, 0, b"bsp oled=");
    pos = push_bytes(&mut msg, pos, if devices.oled { b"1" } else { b"0" });
    pos = push_bytes(&mut msg, pos, b" rtc=");
    pos = push_bytes(&mut msg, pos, if devices.rtc { b"1" } else { b"0" });
    pos = push_bytes(&mut msg, pos, b" env=");
    pos = push_bytes(&mut msg, pos, if devices.environment { b"1" } else { b"0" });
    // Report once, at the end, with everything in it: the mailbox keeps only the
    // last record, so a probe line followed by a reading line hides the probe.
    let devices_ok = devices.complete();
    let _ = &msg[..pos];

    let mut display = board.oled();
    let mut environment = match board.environment() {
        Ok(e) => e,
        Err(_) => {
            mailbox::fail(b"bsp: no environment sensor");
            loop {
                led.toggle();
                cortex_m::asm::delay(2_000_000);
            }
        }
    };
    let mut clock = board.rtc();

    let mut shown_env = 0u32;
    loop {
        // Environment: one conversion, no gas heater.
        // The driver takes its delay from the caller: on bare metal that is NOPs, in
        // a scheduler it would be a sleep. `read_with` is why the BSP can stay thin.
        let reading = environment.read_with(|| cortex_m::asm::delay(400_000));
        let when = clock.read();

        if let Some(mut d) = display.take() {
            d.clear();

            match &when {
                Ok(t) => {
                    let mut line = [0u8; 24];
                    let mut p = push_dec(&mut line, 0, u32::from(t.year));
                    p = push_bytes(&mut line, p, b"-");
                    p = push_pad2(&mut line, p, u32::from(t.month));
                    p = push_bytes(&mut line, p, b"-");
                    p = push_pad2(&mut line, p, u32::from(t.day));
                    d.text(0, 0, &line[..p]);

                    let mut line = [0u8; 24];
                    let mut p = push_pad2(&mut line, 0, u32::from(t.hours));
                    p = push_bytes(&mut line, p, b":");
                    p = push_pad2(&mut line, p, u32::from(t.minutes));
                    p = push_bytes(&mut line, p, b":");
                    p = push_pad2(&mut line, p, u32::from(t.seconds));
                    d.text(0, 1, &line[..p]);
                }
                Err(_) => d.text(0, 0, b"rtc: no time"),
            }

            match &reading {
                Ok(r) => {
                    let mut line = [0u8; 24];
                    let mut p = append(&mut line, 0, b"T ");
                    // hundredths of a degree, printed with one decimal
                    p += format_fixed(&mut line[p..], r.temperature_centi_c, 1);
                    p = append(&mut line, p, b"C");
                    d.text(0, 3, &line[..p]);

                    let mut line = [0u8; 24];
                    let mut p = append(&mut line, 0, b"P ");
                    p = push_dec(&mut line, p, r.pressure_hpa as u32);
                    p = append(&mut line, p, b"hPa");
                    d.text(0, 4, &line[..p]);

                    let mut line = [0u8; 24];
                    let mut p = append(&mut line, 0, b"H ");
                    p = push_dec(&mut line, p, r.humidity_centi_pct as u32 / 100);
                    p = append(&mut line, p, b"%");
                    d.text(0, 5, &line[..p]);
                }
                Err(_) => d.text(0, 3, b"sensor read failed"),
            }

            d.text(0, 7, b"XIAO MG24 Sense");
            d.flush();
            display = Some(d);
        }

        shown_env += 1;
        if shown_env == 1 {
            if let Ok(r) = reading {
                let mut msg = [0u8; 64];
                let mut pos = push_bytes(&mut msg, 0, b"bsp dev=");
                pos = push_bytes(&mut msg, pos, if devices.oled { b"1" } else { b"0" });
                pos = push_bytes(&mut msg, pos, if devices.rtc { b"1" } else { b"0" });
                pos = push_bytes(&mut msg, pos, if devices.environment { b"1" } else { b"0" });
                pos = push_bytes(&mut msg, pos, b" T=");
                pos = push_dec(&mut msg, pos, r.temperature_centi_c as u32 / 100);
                pos = push_bytes(&mut msg, pos, b"C P=");
                pos = push_dec(&mut msg, pos, r.pressure_hpa as u32);
                pos = push_bytes(&mut msg, pos, b"hPa H=");
                pos = push_dec(&mut msg, pos, r.humidity_centi_pct as u32 / 100);
                if devices_ok {
                    mailbox::pass(&msg[..pos]);
                } else {
                    mailbox::fail(&msg[..pos]);
                }
            }
        }

        led.toggle();
        cortex_m::asm::delay(2_000_000);
    }
}
