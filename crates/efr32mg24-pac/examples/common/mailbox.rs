//! A tiny report channel for hardware tests, read back over SWD.
//!
//! The problem this solves: a blink, an OLED string or a buzzer tone can only
//! be judged by a human looking at the board. A test that needs eyes cannot be
//! run autonomously, which is exactly the bottleneck on this project.
//!
//! So every example that wants to report something writes a small,
//! magic-tagged record into a fixed RAM location and then behaves normally.
//! The host reads RAM with `probe-rs read` (read-only, no flashing) and parses
//! the record. Nothing about it depends on the clock tree, the baud rate, or
//! the SAMD11's USB serial bridge being healthy.
//!
//! Layout (little-endian), all `u32` fields word-aligned:
//!
//! ```text
//! 0x00  magic   = 0x4D42_4F58 ("MBOX")
//! 0x04  seq     increments on every write, so a stale record is visible
//! 0x08  status  0 = info, 1 = pass, 2 = fail
//! 0x0C  len     message length in bytes (<= 200)
//! 0x10  bytes   message, NOT NUL-terminated
//! ```
//!
//! Address: `MAILBOX_ADDR`. RAM is 256 KiB at 0x2000_0000, and the examples
//! here use a few KiB of it, so the top 4 KiB block is unused. **This is a
//! diagnostic convention for examples only** — a real application must place
//! the mailbox with the linker, not by assumption.

#![allow(dead_code)]

/// Fixed record location. Chosen 4 KiB below the top of the 256 KiB RAM.
pub const MAILBOX_ADDR: usize = 0x2003_F000;

const MAGIC: u32 = 0x4D42_4F58; // "MBOX"

/// Status values understood by the host-side reader.
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum Status {
    Info = 0,
    Pass = 1,
    Fail = 2,
}

/// Write a record to the mailbox. Safe to call repeatedly; `seq` increases so
/// the reader can tell a fresh record from a leftover one in uninitialised RAM.
pub fn write(status: Status, msg: &[u8]) {
    let base = MAILBOX_ADDR as *mut u32;
    let len = core::cmp::min(msg.len(), 200);

    // Read the previous sequence number before overwriting anything.
    let prev_seq = unsafe { core::ptr::read_volatile(base.add(1)) };
    let seq = if prev_seq == MAGIC {
        1
    } else {
        prev_seq.wrapping_add(1)
    };

    unsafe {
        core::ptr::write_volatile(base.add(0), MAGIC);
        core::ptr::write_volatile(base.add(1), seq);
        core::ptr::write_volatile(base.add(2), status as u32);
        core::ptr::write_volatile(base.add(3), len as u32);

        let dst = (MAILBOX_ADDR + 16) as *mut u8;
        for (i, b) in msg.iter().take(len).enumerate() {
            core::ptr::write_volatile(dst.add(i), *b);
        }
    }
}

/// Convenience wrappers.
pub fn info(msg: &[u8]) {
    write(Status::Info, msg);
}
pub fn pass(msg: &[u8]) {
    write(Status::Pass, msg);
}
pub fn fail(msg: &[u8]) {
    write(Status::Fail, msg);
}
