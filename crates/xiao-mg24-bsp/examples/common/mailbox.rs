//! A tiny report channel for hardware tests, read back over SWD.
//!
//! The problem this solves: a blink, an OLED string or a buzzer tone can only
//! be judged by a human looking at the board. A test that needs eyes cannot be
//! run autonomously, which is exactly the bottleneck on this project.
//!
//! So every example that wants to report something writes a small,
//! magic-tagged record into a fixed RAM location and then behaves normally.
//! The host reads RAM over SWD and parses the record. Nothing about it depends
//! on the clock tree, the baud rate, or the SAMD11's USB serial bridge.
//!
//! Layout (little-endian), `u32` fields:
//!
//! ```text
//! 0x00  magic   = 0x4D42_4F58 ("MBOX"), 0 while a write is in progress
//! 0x04  seq     increments on every write; a stale record is visible
//! 0x08  status  0 = info, 1 = pass, 2 = fail
//! 0x0C  len     message length in bytes (<= 200)
//! 0x10  bytes   message, NOT NUL-terminated
//! ```
//!
//! TORN-READ PROTOCOL: the host can halt the core at any instruction, so a
//! reader can catch the record half-written. The writer therefore clears
//! `magic` first, fills everything, and sets `magic` LAST; the reader discards
//! any record whose magic is not set, and should read twice and accept only a
//! pair with the same `seq`. Without this, a read during a write returns a
//! record like `seq=4185464103 ... "blink seq=000000"` — which is exactly what
//! happened before the protocol was added.
//!
//! Address: `MAILBOX_ADDR`. RAM is 256 KiB at 0x2000_0000 and these examples use
//! a few KiB, so the top 4 KiB block is unused. **Diagnostic convention for
//! examples only** — a real application must place the mailbox with the linker.

#![allow(dead_code)]

/// Fixed record location, 4 KiB below the top of the 256 KiB RAM.
pub const MAILBOX_ADDR: usize = 0x2003_F000;

const MAGIC: u32 = 0x4D42_4F58; // "MBOX"

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum Status {
    Info = 0,
    Pass = 1,
    Fail = 2,
}

/// Write a record to the mailbox using the clear-fill-publish order.
pub fn write(status: Status, msg: &[u8]) {
    let base = MAILBOX_ADDR as *mut u32;
    let len = core::cmp::min(msg.len(), 200);

    // Fresh RAM is not zero, so the first record must not inherit whatever the
    // sequence word happens to contain: only continue a sequence if a valid
    // record is already published.
    let prev_magic = unsafe { core::ptr::read_volatile(base) };
    let prev_seq = unsafe { core::ptr::read_volatile(base.add(1)) };
    let seq = if prev_magic == MAGIC {
        prev_seq.wrapping_add(1)
    } else {
        1
    };

    unsafe {
        // 1. Invalidate first, so a reader mid-write sees "no record".
        core::ptr::write_volatile(base, 0);
        // 2. Fill the payload.
        core::ptr::write_volatile(base.add(1), seq);
        core::ptr::write_volatile(base.add(2), status as u32);
        core::ptr::write_volatile(base.add(3), len as u32);
        let dst = (MAILBOX_ADDR + 16) as *mut u8;
        for (i, b) in msg.iter().take(len).enumerate() {
            core::ptr::write_volatile(dst.add(i), *b);
        }
        // 3. Publish.
        core::ptr::write_volatile(base, MAGIC);
    }
}

pub fn info(msg: &[u8]) {
    write(Status::Info, msg);
}
pub fn pass(msg: &[u8]) {
    write(Status::Pass, msg);
}
pub fn fail(msg: &[u8]) {
    write(Status::Fail, msg);
}

/// Append a byte string to a message buffer, taking the length from the string itself.
///
/// Three examples in this project panicked with "source slice length does not match destination
/// slice length" because they wrote `msg[at..at + 17].copy_from_slice(b"...18 bytes...")` by hand.
/// A hand-counted length is a bug waiting for the day somebody edits the text, and the panic lands in
/// a `panic_halt` loop that looks exactly like a hardware hang. Use this instead.
pub fn push(out: &mut [u8], at: &mut usize, bytes: &[u8]) {
    out[*at..*at + bytes.len()].copy_from_slice(bytes);
    *at += bytes.len();
}
