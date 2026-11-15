//! The EMU's voltage scale, changed and put back, reported rather than assumed.
//!
//! `emu` has been in the HAL for months with no example reaching it. This is the cheapest honest one: ask for VSCALE1,
//! ask for VSCALE2 again, and report what each call returned. Nothing here sleeps the core or changes a clock, so a
//! failure is the call's own return value and not a lost debug connection.
//!
//! What it does *not* prove: the core actually ran at the lower scale. That needs a measurement of current draw, which
//! this board cannot give us from here, so the record says "accepted", not "measured at 1.0 V".

#![no_std]
#![no_main]

use core::fmt::Write;
use efr32mg2x_hal::emu::{Emu, VoltageScale};
use efr32mg2x_hal::entry;
use efr32mg2x_hal::pac;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

struct Buf {
    b: [u8; 120],
    n: usize,
}

impl Buf {
    fn new() -> Self {
        Self { b: [0; 120], n: 0 }
    }
    fn as_bytes(&self) -> &[u8] {
        &self.b[..self.n]
    }
}

impl Write for Buf {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.as_bytes() {
            if self.n < self.b.len() {
                self.b[self.n] = *byte;
                self.n += 1;
            }
        }
        Ok(())
    }
}

#[entry]
fn main() -> ! {
    let dp = match pac::Peripherals::take() {
        Some(d) => d,
        None => loop {
            core::hint::spin_loop();
        },
    };
    let emu = Emu::new(dp.emu_s);

    let down = emu.set_em01_voltage_scale(VoltageScale::Vscale1);
    let back = emu.set_em01_voltage_scale(VoltageScale::Vscale2);

    let mut b = Buf::new();
    let _ = write!(
        b,
        "emu: VS2->VS1 {} , VS1->VS2 {}",
        if down.is_ok() { "ok" } else { "ERR" },
        if back.is_ok() { "ok" } else { "ERR" },
    );

    if down.is_ok() && back.is_ok() {
        mailbox::pass(b.as_bytes());
    } else {
        mailbox::fail(b.as_bytes());
    }

    loop {
        for _ in 0..2_000_000 {
            core::hint::spin_loop();
        }
        mailbox::info(b.as_bytes());
    }
}
