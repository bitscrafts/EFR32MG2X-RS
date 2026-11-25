//! The EMU's voltage scale, changed and put back, reported rather than assumed.
//!
//! `emu` has been in the HAL for months with no example reaching it. This is the cheapest honest one: ask for VSCALE1,
//! ask for VSCALE2 again, and report what each call returned. Nothing here sleeps the core or changes a clock, so a
//! failure is the call's own return value and not a lost debug connection.
//!
//! ## Why it repeats, and why the old record looked nondeterministic
//!
//! The previous version made the two calls once, wrote a verdict, and then **rewrote the same text as `info` every
//! second** in a loop. A reader over SWD almost always lands on the `info` copy, so the record showed
//! `ok , ok` with no verdict at all -- and the two readings on file, one `ok , ok` and one `ERR , ERR`, cannot be
//! compared because neither carried a status. **An instrument that overwrites its own verdict with a progress message
//! destroys the only thing it was built to say.**
//!
//! Now it runs the round trip five times, counts what came back, and writes one verdict at the end. Five tries turn
//! "sometimes" into a number: `5/5` and `4/5` are different facts, and only one of them is a working driver.
//!
//! What it does *not* prove: that the core actually ran at the lower scale. That needs a current measurement this
//! board cannot give from here, so the record says "accepted", not "measured at 1.0 V".
//!
//! VERIFIED ON HARDWARE (2026-11-24): the record is in this file's header and in the crate README's dated table.

#![no_std]
#![no_main]

use core::fmt::Write;
use efr32mg2x_hal::emu::VoltageScale;
use efr32mg2x_hal::entry;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

/// Round trips to try. Five is enough to see a one-in-three flake and cheap enough to stay in one boot.
const TRIES: u32 = 5;

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
    mailbox::invalidate();
    let board = match efr32mg2x_hal::board::Board::take() {
        Some(b) => b,
        None => loop {
            core::hint::spin_loop();
        },
    };
    let emu = board.emu().expect("the EMU is handed out once");

    let mut down_ok = 0u32;
    let mut up_ok = 0u32;
    for _ in 0..TRIES {
        if emu.set_em01_voltage_scale(VoltageScale::Vscale1).is_ok() {
            down_ok += 1;
        }
        if emu.set_em01_voltage_scale(VoltageScale::Vscale2).is_ok() {
            up_ok += 1;
        }
    }

    let mut b = Buf::new();
    let _ = write!(
        b,
        "emu: VS2->VS1 {down_ok}/{TRIES} , VS1->VS2 {up_ok}/{TRIES}"
    );

    // Every round trip has to land, in both directions: a driver that works four times in five is a driver whose
    // fifth caller runs at a scale nobody verified.
    if down_ok == TRIES && up_ok == TRIES {
        mailbox::pass(b.as_bytes());
    } else {
        mailbox::fail(b.as_bytes());
    }

    loop {
        core::hint::spin_loop();
    }
}
