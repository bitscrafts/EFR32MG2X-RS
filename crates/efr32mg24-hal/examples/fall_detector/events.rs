//! A log of what the classifier saw, kept in RAM so it survives the read delay.
//!
//! The waveform ring holds ten seconds. That is the right size for a recording and the
//! wrong size for a test performed by a person: the bed drops were done and the ring had
//! already moved on by the time the debug probe looked at it, so a real experiment
//! produced no evidence. A short log that is *never* overwritten by time — every event
//! from boot, up to [`LOG_LEN`] — removes the coupling between "when the board is
//! dropped" and "when the host reads".
//!
//! Each entry is what a decision needs: how long the device was in the air, how hard it
//! landed, and how the classifier ruled on it.

/// What happened.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A qualifying free fall ended. The landing has not been judged yet.
    FreeFall = 1,
    /// A hard hit with no free fall before it.
    Impact = 2,
    /// Free fall, landing, quiet: reported as a fall.
    Fall = 3,
}

/// One line of the log.
#[derive(Clone, Copy)]
pub struct Entry {
    /// Milliseconds since boot, so entries can be ordered and separated in time.
    pub uptime_ms: u32,
    /// Free fall that preceded the event, in milliseconds (0 when there was none).
    pub freefall_ms: u16,
    /// Largest magnitude seen, in thousandths of a g.
    pub peak_mg: u16,
    /// What the classifier decided.
    pub kind: u8,
}

/// How many events are kept. 64 covers a test session; the oldest is dropped after that.
pub const LOG_LEN: usize = 64;

/// The log itself, oldest first.
pub static mut LOG: [Entry; LOG_LEN] = [Entry {
    uptime_ms: 0,
    freefall_ms: 0,
    peak_mg: 0,
    kind: 0,
}; LOG_LEN];

/// How many entries have ever been written (may exceed [`LOG_LEN`]).
pub static mut WRITTEN: u32 = 0;

/// Falls since boot. A counter cannot be overwritten by volume: the shaking control
/// filled the log 22 times over and made "did anything happen?" unanswerable, which is
/// the one question a control exists to answer.
pub static mut FALLS: u32 = 0;

/// Impacts since boot, one per episode.
pub static mut IMPACTS: u32 = 0;

/// Free-fall episodes since boot.
pub static mut AIRBORNE: u32 = 0;

/// Append an entry. The caller owns the ordering; this never blocks.
pub fn push(uptime_ms: u32, kind: Kind, freefall_ms: u16, peak_mg: u16) {
    // SAFETY: single-threaded firmware; this module is the only writer and the address is
    // handed to the debug probe read-only.
    unsafe {
        let written = core::ptr::read_volatile(core::ptr::addr_of!(WRITTEN));
        let slot = (written as usize) % LOG_LEN;
        let entry = Entry {
            uptime_ms,
            freefall_ms,
            peak_mg,
            kind: kind as u8,
        };
        core::ptr::write_volatile(core::ptr::addr_of_mut!(LOG[slot]), entry);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(WRITTEN), written.wrapping_add(1));
        let counter = match kind {
            Kind::Fall => core::ptr::addr_of_mut!(FALLS),
            Kind::Impact => core::ptr::addr_of_mut!(IMPACTS),
            Kind::FreeFall => core::ptr::addr_of_mut!(AIRBORNE),
        };
        let value = core::ptr::read_volatile(counter);
        core::ptr::write_volatile(counter, value.wrapping_add(1));
    }
}

/// How many entries have been written (the count, not the index).
#[must_use]
pub fn written() -> u32 {
    // SAFETY: read-only, single-threaded.
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(WRITTEN)) }
}
