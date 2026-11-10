//! PAC-only read of the Expansion Base RTC (PCF8563 at 0x51).
//!
//! Reads registers 0x02..0x08 (seconds, minutes, hours, day, weekday, month,
//! year) as BCD, decodes them and reports the date and time through the
//! mailbox. Also reports the VL bit (bit 7 of seconds): when it is set the
//! oscillator stopped at some point and the time is not trustworthy — a fact
//! worth reporting instead of silently printing a wrong clock.
//!
//! Verification without eyes: two reads a few seconds apart must show the
//! seconds field advancing, which this example does by reading twice and
//! reporting both.
//!
//! The bus is bit-banged because the on-chip I2C0 master will not complete
//! reads on this die; see `common/i2c_bitbang.rs` for the evidence.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/i2c_bitbang.rs"]
mod i2c_bitbang;
#[path = "common/mailbox.rs"]
mod mailbox;

const RTC_ADDR: u8 = 0x51;

fn push_hex2(buf: &mut [u8], mut pos: usize, value: u8) -> usize {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    if pos < buf.len() {
        buf[pos] = HEX[(value >> 4) as usize];
        pos += 1;
    }
    if pos < buf.len() {
        buf[pos] = HEX[(value & 0xF) as usize];
        pos += 1;
    }
    pos
}

fn push_bytes(buf: &mut [u8], mut pos: usize, s: &[u8]) -> usize {
    for b in s {
        if pos < buf.len() {
            buf[pos] = *b;
            pos += 1;
        }
    }
    pos
}

fn bcd_to_dec(v: u8) -> u8 {
    ((v >> 4) & 0x0F) * 10 + (v & 0x0F)
}

struct Clock {
    sec: u8,
    min: u8,
    hour: u8,
    day: u8,
    month: u8,
    year: u8,
    vl: bool,
}

fn read_clock(bus: &i2c_bitbang::Bus<'_>) -> Option<Clock> {
    let mut raw = [0u8; 7]; // 0x02..0x08
    if !bus.read_regs(RTC_ADDR, 0x02, &mut raw) {
        return None;
    }
    Some(Clock {
        sec: bcd_to_dec(raw[0] & 0x7F),
        min: bcd_to_dec(raw[1] & 0x7F),
        hour: bcd_to_dec(raw[2] & 0x3F),
        day: bcd_to_dec(raw[3] & 0x3F),
        month: bcd_to_dec(raw[5] & 0x1F),
        year: bcd_to_dec(raw[6]),
        vl: raw[0] & 0x80 != 0,
    })
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());
    let bus = i2c_bitbang::Bus::new(&dp.gpio_s);

    if !bus.probe(RTC_ADDR) {
        mailbox::fail(b"rtc ack=0 (no device at 0x51)");
        idle(&dp);
    }

    let first = read_clock(&bus);
    // A real gap on the bus, so the seconds field has to move.
    cortex_m::asm::delay(12_000_000);
    let second = read_clock(&bus);

    match (first, second) {
        (Some(a), Some(b)) => {
            let mut buf = [0u8; 96];
            let mut pos = push_bytes(&mut buf, 0, b"rtc 20");
            pos = push_hex2(&mut buf, pos, a.year);
            pos = push_bytes(&mut buf, pos, b"-");
            pos = push_hex2(&mut buf, pos, a.month);
            pos = push_bytes(&mut buf, pos, b"-");
            pos = push_hex2(&mut buf, pos, a.day);
            pos = push_bytes(&mut buf, pos, b" ");
            pos = push_hex2(&mut buf, pos, a.hour);
            pos = push_bytes(&mut buf, pos, b":");
            pos = push_hex2(&mut buf, pos, a.min);
            pos = push_bytes(&mut buf, pos, b":");
            pos = push_hex2(&mut buf, pos, a.sec);
            pos = push_bytes(&mut buf, pos, b" then ");
            pos = push_hex2(&mut buf, pos, b.hour);
            pos = push_bytes(&mut buf, pos, b":");
            pos = push_hex2(&mut buf, pos, b.min);
            pos = push_bytes(&mut buf, pos, b":");
            pos = push_hex2(&mut buf, pos, b.sec);
            pos = push_bytes(&mut buf, pos, if a.vl { b" VL=1" } else { b" VL=0" });

            // The clock is trustworthy when VL is clear and the two samples
            // differ; a stopped oscillator shows the same second twice.
            let advanced = (a.sec, a.min) != (b.sec, b.min);
            if advanced && !a.vl {
                mailbox::pass(&buf[..pos]);
            } else {
                mailbox::fail(&buf[..pos]);
            }
        }
        _ => mailbox::fail(b"rtc read failed"),
    }

    idle(&dp);
}

fn idle(dp: &pac::Peripherals) -> ! {
    dp.gpio_s.porta_model().modify(|_r, w| w.mode7().pushpull());
    loop {
        for _ in 0..2_000_000 {
            cortex_m::asm::nop();
        }
        dp.gpio_s
            .porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 7)) });
    }
}
