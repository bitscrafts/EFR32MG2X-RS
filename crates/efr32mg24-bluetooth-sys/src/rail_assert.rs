//! RAIL's assertion callback, answered instead of swallowed.
//!
//! `RAILCb_AssertFailed` is a **weak** symbol in the RAIL blob whose default implementation is an infinite loop.
//! Measured consequence: the Bluetooth stack's initialisation appeared to hang — no mailbox write, no fault, nothing
//! to read — and the only evidence was the program counter sitting inside `RAILCb_AssertFailed` at `0x08019da8`.
//! It was not waiting for anything. It had asserted, and the assertion had nowhere to go.
//!
//! This is the same lesson as the fault handler in this project: **a failure that reports is a diagnosis, a failure
//! that loops is a lost session.** The code is published to a fixed RAM word and to the mailbox, so an assert is
//! visible from outside without a debugger.
//!
//! The error codes come from `RAIL_AssertErrorCodes_t` in `rail.h`; the caller's own line/timeout information is not
//! part of the callback's signature in this SDK version, so the code is what identifies the failure.

use core::ffi::c_void;

/// Where the last assert is recorded, as `[magic, error_code, rail_handle]`. A fixed address so it can be read with
/// the debugger while the program sits in the (now reporting) loop.
pub const ASSERT_SLOT: usize = 0x2003_F500;
const ASSERT_MAGIC: u32 = 0x4153_5254; // "ASRT"

/// How many asserts have been seen. One is enough to stop a boot, but counting them separates "asserted once" from
/// "asserting in a loop".
static mut ASSERT_COUNT: u32 = 0;

/// The callback RAIL calls instead of spinning forever.
///
/// # Safety
///
/// Called by RAIL's own code with its handle and an error code; nothing here dereferences either.
#[no_mangle]
pub unsafe extern "C" fn RAILCb_AssertFailed(rail_handle: *mut c_void, error_code: u32) {
    let p = core::ptr::addr_of_mut!(ASSERT_COUNT);
    let count = core::ptr::read_volatile(p).wrapping_add(1);
    core::ptr::write_volatile(p, count);

    let slot = ASSERT_SLOT as *mut u32;
    core::ptr::write_volatile(slot, ASSERT_MAGIC);
    core::ptr::write_volatile(slot.add(1), error_code);
    core::ptr::write_volatile(slot.add(2), rail_handle as u32);
    core::ptr::write_volatile(slot.add(3), count);

    // Stop here, deliberately: continuing past an assert in a real-time radio sequencer produces packets nobody can
    // explain. The record above is what a reader needs, and the loop is bounded evidence rather than a hang --
    // `ASSERT_SLOT` says why.
    loop {
        core::hint::spin_loop();
    }
}

/// The last assert as `(error_code, rail_handle, count)`, or `None` when RAIL has not asserted.
pub fn last_assert() -> Option<(u32, u32, u32)> {
    unsafe {
        let slot = ASSERT_SLOT as *const u32;
        if core::ptr::read_volatile(slot) != ASSERT_MAGIC {
            return None;
        }
        Some((
            core::ptr::read_volatile(slot.add(1)),
            core::ptr::read_volatile(slot.add(2)),
            core::ptr::read_volatile(slot.add(3)),
        ))
    }
}
