//! The recording ring: the last ten seconds of acceleration, kept in RAM.
//!
//! The classifier and the dataset need the same thing at the same time. A build that
//! only classifies cannot be trained on (there is nothing to read afterwards), and a
//! build that only records cannot be watched while the board is being dropped. So the
//! application records every sample it classifies: the run that demonstrates the
//! detector is also the run that produces the training data.
//!
//! 4000 samples is 24 KB of the 252 KB now available. The host reads it over SWD from
//! the address announced in the mailbox at boot.
//!
//! **Every sample stores the interval that preceded it.** The loop cannot promise a rate
//! — the display flush alone blocks for tens of milliseconds on a bit-banged bus — so a
//! recording without its own timing cannot be analysed honestly. An earlier round of
//! drops was analysed as if it had run at 400 Hz when the loop had actually run at 158,
//! which made every duration in the analysis 2.5 times too short.

/// Samples kept.
pub const CAPTURE_LEN: usize = 4000;

/// Word-aligned so the AHB/debug reads see plain 32-bit words.
#[repr(align(32))]
pub struct Samples(pub [[i16; 3]; CAPTURE_LEN]);

/// The ring itself.
pub static mut CAPTURE: Samples = Samples([[0; 3]; CAPTURE_LEN]);

/// Interval before each stored sample, in 250 µs units (capped at 255 = 63.75 ms).
pub static mut DTS: [u8; CAPTURE_LEN] = [0; CAPTURE_LEN];

/// How many samples have been written since boot — the host needs it to know where
/// the newest sample is (`TOTAL % CAPTURE_LEN`).
pub static mut TOTAL: u32 = 0;

/// Address of the ring, for the boot report.
#[must_use]
pub fn base_address() -> u32 {
    unsafe { core::ptr::addr_of!(CAPTURE.0) as u32 }
}

/// Address of the per-sample intervals.
#[must_use]
pub fn dts_address() -> u32 {
    unsafe { core::ptr::addr_of!(DTS) as u32 }
}

/// Address of the sample counter.
#[must_use]
pub fn total_address() -> u32 {
    unsafe { core::ptr::addr_of!(TOTAL) as u32 }
}

/// Append one sample, with the interval that preceded it.
pub fn push_timed(accel_mg: [i32; 3], index: usize, dt_us: u32) {
    // Milligrams fit comfortably in i16 at ±8 g (±8000 mg); the clamp is there so a
    // wild sample cannot wrap into a plausible-looking value.
    let clamp = |v: i32| v.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
    let units = (dt_us / 250).min(255) as u8;
    unsafe {
        CAPTURE.0[index] = [clamp(accel_mg[0]), clamp(accel_mg[1]), clamp(accel_mg[2])];
        DTS[index] = units;
        TOTAL = TOTAL.wrapping_add(1);
    }
}

/// Samples written so far.
#[must_use]
pub fn total() -> u32 {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(TOTAL)) }
}
