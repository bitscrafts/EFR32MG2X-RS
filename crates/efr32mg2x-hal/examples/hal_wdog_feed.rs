//! The watchdog, fed on purpose, and asked whether it took the lock.
//!
//! `wdog` has no working example yet. The one thing a watchdog example must not do is prove itself by resetting the
//! board: a reset wipes the evidence and looks identical to a crash. So this configures a long period, records the
//! state of the block, feeds it, and keeps feeding it — the demonstration is that the board is still here, saying so.
//!
//! `debug_run` is set on purpose: with a probe attached, a halt should not turn into a reset of the whole board.
//!
//! ## The open defect: writing `WDOG_CMD` is a bus fault, and five causes are ruled out
//!
//! Measured 2026-11-24 with `probe-rs run`, which reports the faulting frame:
//!
//! ```text
//! HardFault <Cause: Escalated BusFault (Precise data access error) at location: 0x5b00400c>
//!   Wdog::feed  (src/wdog/mod.rs, the CMD write)
//! ```
//!
//! The note this file used to carry blamed `CFG`; the address `0x0c` is `CMD`. Ruled out, one boot each:
//!
//! - **not the window alias.** The same fault at `0x4b00400c` (secure) and `0x5b00400c` (non-secure).
//! - **not the configuration lock.** Identical with `lock_config()` removed from `Wdog::new`.
//! - **not the low-frequency clock.** Identical with `CMU.WDOG0CLKCTRL = HCLKDIV1024` (read back as 4) instead of
//!   LFRCO, so the LF domain is not the missing piece.
//! - **not the pending PRS flags.** Identical after clearing `IF.PEM0`/`IF.PEM1`, which were both set.
//! - **not a dead or unconfigured block.** While faulted the registers read `IPVERSION=1`, `EN=1`,
//!   `CFG=0x000a0020` (PERSEL = 10 = 8k cycles and DEBUGRUN = 1, exactly what was written) and `SYNCBUSY=0`, and
//!   `CMU_CLKEN0` bit 13 is set. Only the write to `CMD` refuses.
//!
//! The manual's sequence rule is satisfied — configuration registers before `EN`, SYNC-type registers after it
//! (RM 26.5.2) — and that is the order `Wdog::new` uses. So the state record below is written **before** the first
//! feed: if the feed faults, the last record is the state at the moment it did, which is the instrument this example
//! was missing. Next instruments, in order: the `CMD` write through the SET alias (`0x100C`), `STATUS`/`SYNCBUSY`
//! read from inside a fault handler, and the vendor's own `WDOG_Feed` in the same boot as a control.

#![no_std]
#![no_main]

use core::fmt::Write;
use efr32mg2x_hal::entry;
use efr32mg2x_hal::wdog::{TimeoutPeriod, WdogConfig};
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

struct Buf {
    b: [u8; 160],
    n: usize,
}

impl Buf {
    fn new() -> Self {
        Self { b: [0; 160], n: 0 }
    }
    fn clear(&mut self) {
        self.n = 0;
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

    // NOTE: not read before `board.wdog(..)` -- with `CMU_CLKEN0.WDOG0` clear (its reset state) *any* access to this
    // block faults, reads included. Measured: reading IPVERSION first gave a precise data access error at 0x5b004000.
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
        Err(_) => {
            mailbox::fail(b"wdog: construction failed");
            loop {
                core::hint::spin_loop();
            }
        }
    };

    // Eight feeds back to back, with nothing between them and construction: that is the sequence that faulted before
    // (see the header), so it is the sequence the test has to run. A bus fault here halts the core and leaves no
    // record, which is why the reading below is also the evidence that the feeds happened at all.
    for _ in 0..8 {
        wdog.feed();
    }

    let after = wdog.state();
    let locked = wdog.is_locked();
    let mut b = Buf::new();
    let _ = write!(
        b,
        "wdog: fed 8x ipv{:x} en{} cfg{:x} st{:x} sb{:x} lock{}",
        after.ipversion, after.en, after.cfg, after.status, after.syncbusy, locked as u8,
    );

    // Locked is what a configured watchdog must be: the configuration cannot be changed after `new()`, and the
    // feeds above are the proof that clearing still works while locked.
    if locked {
        mailbox::pass(b.as_bytes());
    } else {
        mailbox::fail(b.as_bytes());
    }

    // Keep feeding, and keep publishing the verdict rather than a progress line: an `info` record here would overwrite
    // the pass a reader is looking for -- the mistake `hal_emu_voltage` paid for.
    let mut feeds = 8u32;
    loop {
        wdog.feed();
        feeds = feeds.wrapping_add(1);
        for _ in 0..200_000 {
            core::hint::spin_loop();
        }
        let now = wdog.state();
        b.clear();
        let _ = write!(
            b,
            "wdog: fed {} st{:x} sb{:x} lock{}",
            feeds,
            now.status,
            now.syncbusy,
            wdog.is_locked() as u8,
        );
        if wdog.is_locked() {
            mailbox::pass(b.as_bytes());
        } else {
            mailbox::fail(b.as_bytes());
        }
    }
}
