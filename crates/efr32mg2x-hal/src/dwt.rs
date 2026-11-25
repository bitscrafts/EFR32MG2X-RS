//! The cycle counter, as a time base that cannot lie — and that says so when it cannot count.
//!
//! Every timing defect in this project was a *declared* interval that nobody measured: a loop that said 400 Hz and ran
//! at 158, an `asm::delay(1_000_000)` that was not a million cycles, a millisecond counter built by dividing
//! microseconds by 1000 and losing the remainder. The DWT cycle counter is the antidote: it counts CPU cycles, so the
//! interval between two samples can be *measured* rather than assumed.
//!
//! ## The counter only runs while a debug session is attached
//!
//! Measured 2026-11-24, twice, with the host's clock as the referee: with the probe detached, two readings 10.8 s apart
//! returned the same value while the SYSRTC advanced 346 091 ticks; under `probe-rs run` the same firmware gave a
//! DWT/SYSRTC ratio of 578.4 against the 579.8 that 19 MHz / 32.768 kHz predicts. `DEMCR.TRCENA` and `CYCCNTENA` read
//! back set while the counter stands still: the writes land, and the counter is simply not clocked once the debug
//! domain powers down. Record: `docs/04-reference/DWT-NEEDS-A-DEBUG-SESSION.md`.
//!
//! That makes a bare reading a trap — it returns a frozen number, which reads as "the interval was zero" and looks like
//! a very fast loop. So the entry point is [`start`], which **proves the counter moves** before handing anything back,
//! and returns [`DwtError::NotRunning`] when it does not. A program that has to measure itself without a probe should
//! use SysTick or a peripheral counter instead; this module is an instrument, not a time source.
//!
//! Only the board knows when this is safe: enabling the counter costs one write and nothing else in this crate touches
//! DWT.

use cortex_m::peripheral::DWT;

/// The counter was enabled and did not advance.
///
/// Not a hardware fault: it is the normal state of this part when no debug session is attached. Read it as "this
/// instrument is unavailable here", not as "the code is broken".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DwtError {
    /// `CYCCNT` did not change over the probe window; the debug domain is almost certainly powered down.
    NotRunning,
}

/// How many reads to spend proving the counter moves before giving up on it.
///
/// One read is a handful of cycles, so this is a few thousand cycles — microseconds at any core clock this part runs
/// at, and far longer than the gap between two `CYCCNT` increments.
const PROBE_READS: u32 = 4096;

/// Enable the trace subsystem and the cycle counter, and prove that it counts. Returns the current count.
///
/// Returns [`DwtError::NotRunning`] when the counter stands still, which is what this part does without a debug
/// session attached. The error must be handled: a frozen counter silently reported as zero is the defect this
/// signature exists to prevent.
///
/// # Safety
///
/// The caller takes ownership of the DWT: nothing else in the program may reconfigure `DEMCR` or `DWT->CTRL`. On a chip
/// with a debugger attached the trace unit may already be in use.
pub unsafe fn start() -> Result<u32, DwtError> {
    let mut core = cortex_m::Peripherals::steal();
    core.DCB.enable_trace();
    // The counter must be cleared before it is enabled: a stale CYCCNT makes the first measured interval look like a
    // wrap.
    core.DWT.cyccnt.write(0);
    core.DWT.enable_cycle_counter();

    let first = cycles();
    for _ in 0..PROBE_READS {
        if cycles() != first {
            return Ok(first);
        }
    }
    Err(DwtError::NotRunning)
}

/// The cycle counter, read with the pointer rather than a stolen peripheral.
///
/// Only meaningful after [`start`] returned `Ok`: with the counter stopped this returns a constant, which is why the
/// entry point is the one that checks.
#[inline]
#[must_use]
pub fn cycles() -> u32 {
    // SAFETY: DWT lives at a fixed address and CYCCNT is a read-only counter here.
    unsafe { (*DWT::PTR).cyccnt.read() }
}

/// Cycles elapsed since `since`, wrapping once.
#[inline]
#[must_use]
pub fn since(since: u32) -> u32 {
    cycles().wrapping_sub(since)
}

/// Convert cycles to microseconds, given the CPU frequency.
#[must_use]
pub fn cycles_to_us(cycles: u32, sysclk_hz: u32) -> u32 {
    if sysclk_hz == 0 {
        return 0;
    }
    ((u64::from(cycles) * 1_000_000) / u64::from(sysclk_hz)) as u32
}
