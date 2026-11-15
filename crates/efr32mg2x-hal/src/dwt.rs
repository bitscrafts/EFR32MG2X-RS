//! The cycle counter, as a time base that cannot lie.
//!
//! Every timing defect in this project was a *declared* interval that nobody measured: a loop
//! that said 400 Hz and ran at 158, an `asm::delay(1_000_000)` that was not a million cycles, a
//! millisecond counter built by dividing microseconds by 1000 and losing the remainder. The DWT
//! cycle counter is the antidote: it counts CPU cycles, so the interval between two samples can be
//! *measured* rather than assumed.
//!
//! It is a core peripheral (Debug Watchpoint and Trace), not part of the PAC, and it has to be
//! switched on — `TRCENA` in `DEMCR`, then `CYCCNTENA` in `DWT->CTRL`. Both writes are ignored
//! silently if the wrong order is used, and the counter reads as zero, which looks like a
//! measurement of a very fast loop.
//!
//! Only the board knows when this is safe: enabling the counter costs one write and nothing else
//! in this crate touches DWT.

use cortex_m::peripheral::DWT;

/// Enable the trace subsystem and the cycle counter. Returns the current count.
///
/// # Safety
///
/// The caller takes ownership of the DWT: nothing else in the program may reconfigure `DEMCR` or
/// `DWT->CTRL`. On a chip with a debugger attached the trace unit may already be in use.
pub unsafe fn start() -> u32 {
    let mut core = cortex_m::Peripherals::steal();
    core.DCB.enable_trace();
    // The counter must be cleared before it is enabled: a stale CYCCNT makes the first measured
    // interval look like a wrap.
    core.DWT.cyccnt.write(0);
    core.DWT.enable_cycle_counter();
    cycles()
}

/// The cycle counter, read with the pointer rather than a stolen peripheral.
///
/// `#[inline]` on purpose: this is called between two samples, and the call overhead would land
/// inside the interval it is measuring.
#[inline]
#[must_use]
pub fn cycles() -> u32 {
    // SAFETY: DWT lives at a fixed address and CYCCNT is a read-only counter here. If the counter
    // was never started this returns 0, which a caller can detect by comparing two reads.
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
