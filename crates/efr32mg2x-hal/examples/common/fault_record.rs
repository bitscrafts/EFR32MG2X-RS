//! Record a fault where an instrument can read it, instead of hanging silently.
//!
//! Four runs of the f20 transmit-timeout instrument on 2026-11-28 ended with `magic 0`: no record at all. Each one was
//! read as "the program hung", and one of them really did hang inside a wait -- but nothing in the firmware could tell
//! a hang from a fault, because `cortex-m-rt`'s default `HardFault` handler is an infinite loop and says nothing. The
//! project has already met this twice on the bench: `BURTC_NS + 0x08` (`cfsr=0x8200 bfar=0x50064008`) and
//! `ACMP0_S + 0x04` (`cfsr=0x8200 bfar=0x49008004`) both took a HardFault on the first boot of their drivers, and both
//! were found by hand.
//!
//! This installs the handler an instrument needs: it writes the fault status words and the stacked PC into the same
//! mailbox `common/mailbox.rs` already fills, so the next silent run says *which* it was and *where*.
//!
//! It must not use the allocator, `Record`'s helpers that could panic, or anything that needs a working stack far
//! beyond what the fault left: a plain 128-byte buffer and byte writes.

use cortex_m_rt::{exception, ExceptionFrame};

use crate::mailbox;
use efr32mg2x_hal::record::Record;

/// The System Control Block words a fault leaves behind.
#[allow(dead_code)]
mod scb {
    pub const CFSR: *const u32 = 0xE000_ED28 as *const u32;
    pub const HFSR: *const u32 = 0xE000_ED2C as *const u32;
    pub const MMFAR: *const u32 = 0xE000_ED34 as *const u32;
    pub const BFAR: *const u32 = 0xE000_ED38 as *const u32;
}

// Nothing else is needed to install it: `#[exception]` defines `HardFault`, and linking this module is what replaces
// the default handler. The example that wants it declares the module, which is also how it stops being dead code.

/// Write the fault's own words into the mailbox. Never returns.
///
/// `CFSR` names the class: `PRECISERR` (bit 9) means the address is in `BFAR`, which for this part is how "a write to a
/// peripheral nobody enabled" reads. `HFSR.FORCED` (bit 30) means a fault escalated, which is what a fault taken inside
/// a fault handler looks like.
#[exception]
unsafe fn HardFault(frame: &ExceptionFrame) -> ! {
    // SAFETY: these are the architecturally fixed SCB addresses; reading them is what a fault handler is for, and the
    // block cannot be disabled while the core is running.
    let cfsr = unsafe { core::ptr::read_volatile(scb::CFSR) };
    let hfsr = unsafe { core::ptr::read_volatile(scb::HFSR) };
    let mmfar = unsafe { core::ptr::read_volatile(scb::MMFAR) };
    let bfar = unsafe { core::ptr::read_volatile(scb::BFAR) };

    let mut l = Record::<128>::new();
    l.s(b"fault: cfsr=");
    l.h32(cfsr);
    l.s(b" hfsr=");
    l.h32(hfsr);
    l.s(b" bfar=");
    l.h32(bfar);
    l.s(b" mmfar=");
    l.h32(mmfar);
    l.s(b" pc=");
    l.h32(frame.pc());
    mailbox::fail(l.bytes());

    loop {
        cortex_m::asm::wfi();
    }
}
