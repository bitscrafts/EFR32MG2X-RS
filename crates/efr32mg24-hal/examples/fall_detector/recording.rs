//! The recording ring: the last 15 seconds of acceleration, kept in RAM.
//!
//! The classifier and the dataset need the same thing at the same time. A build that
//! only classifies cannot be trained on (there is nothing to read afterwards), and a
//! build that only records cannot be watched while the board is being dropped. So the
//! application records every sample it classifies: the run that demonstrates the
//! detector is also the run that produces the training data.
//!
//! 4000 samples at 400 Hz is 10 seconds and 24 KB of the 252 KB now available. The
//! host reads it over SWD from the address announced in the mailbox at boot.

/// Samples kept: 4000 × 400 Hz = 10 s.
pub const CAPTURE_LEN: usize = 4000;

/// Word-aligned so the AHB/debug reads see plain 32-bit words.
#[repr(align(32))]
pub struct Samples(pub [[i16; 3]; CAPTURE_LEN]);

/// The ring itself.
pub static mut CAPTURE: Samples = Samples([[0; 3]; CAPTURE_LEN]);

/// How many samples have been written since boot — the host needs it to know where
/// the newest sample is (`TOTAL % CAPTURE_LEN`).
pub static mut TOTAL: u32 = 0;

/// Address of the ring, for the boot report.
#[must_use]
pub fn base_address() -> u32 {
    unsafe { core::ptr::addr_of!(CAPTURE.0) as u32 }
}

/// Address of the sample counter.
#[must_use]
pub fn total_address() -> u32 {
    unsafe { core::ptr::addr_of!(TOTAL) as u32 }
}

/// Append one sample.
pub fn push(accel_mg: [i32; 3], index: usize) {
    // Milligrams fit comfortably in i16 at ±8 g (±8000 mg); the clamp is there so a
    // wild sample cannot wrap into a plausible-looking value.
    let clamp = |v: i32| v.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
    unsafe {
        CAPTURE.0[index] = [clamp(accel_mg[0]), clamp(accel_mg[1]), clamp(accel_mg[2])];
        TOTAL = TOTAL.wrapping_add(1);
    }
}

/// Samples written so far.
#[must_use]
pub fn total() -> u32 {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(TOTAL)) }
}
