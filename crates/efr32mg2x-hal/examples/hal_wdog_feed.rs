//! The watchdog, fed on purpose, and asked whether it took the lock.
//!
//! `wdog` had no example. The one thing a watchdog example must not do is prove itself by resetting the board: a reset
//! wipes the evidence and looks identical to a crash. So this configures a long period, confirms `is_locked()` is false,
//! feeds it a fixed number of times, and then keeps feeding it forever — the demonstration is that the board is still
//! here, saying so.
//!
//! `debug_run` is set on purpose: with a probe attached, a halt should not turn into a reset of the whole board.
//!
//! KNOWN DEFECT (2026-11-15, measured, not fixed): this example does not run. Writing `WDOG.CFG` is a precise data access
//! error, at `0x4b00400c` through the secure alias and at `0x5b00400c` through the non-secure one -- so it is not the
//! alias. Reads at both windows succeed (`IPVERSION=1`, `ENABLE=1`), and the unlock key this HAL writes (`0xABE8`) is the
//! one in the vendor's own `efr32mg24_wdog.h`, so neither the address nor the key explains it. Until that is settled this
//! example carries no hardware date.

#![no_std]
#![no_main]

use core::fmt::Write;
use efr32mg2x_hal::entry;
use efr32mg2x_hal::wdog::{TimeoutPeriod, Wdog, WdogConfig};
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
    mailbox::invalidate();
    let board = match efr32mg2x_hal::board::Board::take() {
        Some(b) => b,
        None => loop {
            core::hint::spin_loop();
        },
    };

    let config = WdogConfig {
        period: TimeoutPeriod::Cycles8k,
        em1_run: false,
        em2_run: false,
        em3_run: false,
        debug_run: true,
        reset_disable: false,
    };

    let mut wdog = match board.wdog(config) {
        Ok(w) => w,
        Err(e) => {
            let mut b = Buf::new();
            let _ = write!(b, "wdog: construction failed ({:?})", e);
            mailbox::fail(b.as_bytes());
            loop {
                core::hint::spin_loop();
            }
        }
    };

    let locked_before = wdog.is_locked();
    for _ in 0..8 {
        wdog.feed();
    }
    let locked_after = wdog.is_locked();

    let mut b = Buf::new();
    let _ = write!(
        b,
        "wdog: fed 8x, locked {} -> {}",
        locked_before as u8, locked_after as u8
    );

    if !locked_after {
        mailbox::pass(b.as_bytes());
    } else {
        mailbox::fail(b.as_bytes());
    }

    // Keep feeding: the point of the record is that the board is still answering.
    loop {
        for _ in 0..200_000 {
            core::hint::spin_loop();
        }
        wdog.feed();
        mailbox::info(b.as_bytes());
    }
}
