//! Prove that the fault handler records: take a fault on purpose and read what it left.
//!
//! `common/fault_record.rs` exists because four runs of the f20 instrument ended with no record and nobody could say
//! whether the program hung or faulted. An instrument that cannot fail does not count, so this one provokes the fault:
//! it writes one word to a peripheral address that no block answers, and the handler writes `CFSR`, `HFSR`, `BFAR`,
//! `MMFAR` and the stacked `PC` into the mailbox.
//!
//! Expected, if the handler works: `status=FAIL` with a `fault:` line whose `CFSR` has `PRECISERR` (bit 9) set and whose
//! `BFAR` is the address written below. If instead the record stays `magic 0`, the handler did not take, and that is
//! exactly the ambiguity this file exists to remove.

#![no_std]
#![no_main]

#[path = "common/mailbox.rs"]
mod mailbox;

#[path = "common/fault_record.rs"]
mod fault_record;

use cortex_m_rt::entry;
use defmt_rtt as _;
use panic_halt as _;

/// An address inside the peripheral region with no block behind it. A write here is a precise bus error: the core
/// reports the address in `BFAR`, which is the shape a write to an unclocked peripheral also takes on this part.
const NO_BLOCK: usize = 0x5010_0000;

#[entry]
fn main() -> ! {
    mailbox::invalidate();
    mailbox::info(b"fault-record: writing an address no block answers");

    // SAFETY: the write is the measurement. It is expected to fault, and the handler installed by `fault_record`
    // turns that fault into a record instead of a silent stop. Nothing after it runs.
    unsafe {
        core::ptr::write_volatile(NO_BLOCK as *mut u32, 0xDEAD_BEEF);
    }

    // Only reachable if the write did not fault, which would itself be the finding.
    mailbox::fail(b"fault-record: the write did not fault");
    loop {
        cortex_m::asm::wfi();
    }
}
