//! A fault handler that records what happened instead of hanging.
//!
//! `cortex-m-rt`'s default `HardFault` is an empty loop, which is indistinguishable from a slow initialisation: the
//! board simply stops answering. This one stores the exception frame's program counter together with `CFSR` and
//! `BFAR`, so the fault names its own instruction and the reason.
//!
//! It lives in the library, not in an example, for the same reason the `PendSV` handler does: an example that
//! forgets it fails silently, and the failure looks like the radio rather than like the bug.
//!
//! The record is a `#[no_mangle]` static rather than a fixed address. Measurement, not preference: the first version
//! of this instrumentation wrote to `0x2003_F6xx`, which sits inside the SWD mailbox's own page, and the mailbox's
//! next record silently overwrote it — the values read back were the mailbox's, not the fault's.
//!
//! Reading it, from the host, with the address taken from the ELF:
//!
//! ```text
//! arm-none-eabi-nm <elf> | grep FAULT_RECORD
//! openocd ... -c "mdw <address> 20"
//! ```
//!
//! `FAULT_RECORD[0]` is `0xFA17_0000` when a fault was recorded, `[1]` is the stack pointer that was in use, `[2]`
//! is `CFSR`, `[3]` is `BFAR`, and `[4..20]` are the sixteen words at that stack pointer — the exception frame is
//! the run whose `xPSR` has bit 0 set, with `pc` two words above it.

/// The last fault, or zeroes if none has happened since reset.
///
/// Volatile writes from the handler and `#[no_mangle]` so the address is stable across builds of anything that links
/// this crate.
#[no_mangle]
pub static mut FAULT_RECORD: [u32; 20] = [0; 20];

/// `0xFA17_0000`: tells a recorded fault from a slot that was never written.
pub const FAULT_MAGIC: u32 = 0xFA17_0000;

/// The recorded fault, as `(stack pointer, cfsr, bfar)`.
///
/// `None` if nothing has faulted since reset.
pub fn last_fault() -> Option<(u32, u32, u32)> {
    // SAFETY: reads the record the handler writes; nothing else writes it.
    let record = unsafe { &*core::ptr::addr_of!(FAULT_RECORD) };
    if record[0] == FAULT_MAGIC {
        Some((record[1], record[2], record[3]))
    } else {
        None
    }
}

/// Clear the record, so a run cannot report a fault from the run before it.
pub fn clear_fault() {
    for word in unsafe { &mut *core::ptr::addr_of_mut!(FAULT_RECORD) } {
        // SAFETY: the record is ours; volatile so the write is not optimised away.
        unsafe { core::ptr::write_volatile(word, 0) };
    }
}

/// Record the fault and stop. Never returns — there is nothing safe to return to.
///
/// `cortex-m-rt` hands the exception frame over as an argument, which is better than reconstructing it from `msp`:
/// the handler's own prologue moves the stack, so the address read inside the body is not the frame's.
#[cortex_m_rt::exception]
unsafe fn HardFault(frame: &cortex_m_rt::ExceptionFrame) -> ! {
    let base = core::ptr::addr_of_mut!(FAULT_RECORD) as *mut u32;
    core::ptr::write_volatile(base, FAULT_MAGIC);
    core::ptr::write_volatile(
        base.add(1),
        cortex_m::register::msp::read() as *const u32 as u32,
    );
    // CFSR and BFAR, from the System Control Block.
    core::ptr::write_volatile(
        base.add(2),
        core::ptr::read_volatile(0xE000_ED28 as *const u32),
    );
    core::ptr::write_volatile(
        base.add(3),
        core::ptr::read_volatile(0xE000_ED38 as *const u32),
    );
    // The frame: r0, r1, r2, r3, r12, lr, pc, xpsr.
    let words = frame as *const cortex_m_rt::ExceptionFrame as *const u32;
    for i in 0..8 {
        core::ptr::write_volatile(base.add(4 + i), core::ptr::read_volatile(words.add(i)));
    }
    loop {
        core::hint::spin_loop();
    }
}
