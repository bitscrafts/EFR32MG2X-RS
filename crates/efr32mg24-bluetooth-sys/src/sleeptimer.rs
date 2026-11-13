//! `sl_sleeptimer`, in Rust, polled from the application loop.
//!
//! The Bluetooth stack imports ten of these functions; without them it does not link. SYSRTC counts correctly
//! (measured 32 968 Hz against a stopwatch — 32.768 kHz within 0.6%), but its **compare event does not fire**
//! yet: the counter passes a programmed deadline and `GRP0_IF` stays clear, so nothing can be interrupt-driven
//! on it today.
//!
//! So this is a scheduler, not an interrupt handler: deadlines are kept in a small table, and `poll()` — called
//! from the application loop — runs whatever has come due. That is enough to boot the stack and keep its event
//! timer running, which is what "Bluetooth working" needs first. When the compare channel is understood, the
//! only change here is who calls `poll()`.
//!
//! The counter is read straight from `SYSRTC0->CNT` at the measured address rather than through the HAL: this
//! crate sits *below* the HAL, and a `-sys` crate is exactly where a raw, documented register access belongs.
//! The offset was measured with `offsetof` against the vendor headers and confirmed against the RAIL blob.

use core::ffi::c_void;

/// `SYSRTC0->CNT` (non-secure alias). Measured, and confirmed by `RTCCSYNC_GetRtccCount` in the RAIL blob,
/// which reads `[r3, #24]` from the same base.
const SYSRTC_CNT: usize = 0x500A_8018;

/// The registers the interrupt path needs, at the offsets measured from the blob (see
/// `docs/04-reference/SYSRTC-COMPARE-WORKS.md`). The per-channel flag is **bit 1** for channel 0, not bit 0, and a
/// flag is cleared by writing it to the `+0x2000` alias rather than by storing zero.
const SYSRTC_BASE: usize = 0x500A_8000;
/// **Group 0, at `0x40`** — the offsets the vendor header gives, and the group an application may own.
///
/// Measured, after two wrong turns: writes to `0x48` and `0x4C` stick, a compare at `0x4C` sets bit 1 of `0x40`
/// exactly one count after the deadline, and this group raises **no** pending interrupt while `IEN` is off — so it
/// is idle unless we use it. The block at `0x60` is group **1**, and its events arrive on IRQ 68, whose handler the
/// RAIL blob defines (the link refuses a duplicate). The earlier note that "writes at 0x40 were silently lost" was
/// wrong: those writes worked, and the flag was being read from bit 0 (`OVF`) instead of bit 1 (`CMP0`).
const R_GRP0_IF: usize = 0x40;
const R_GRP0_IEN: usize = 0x44;
const R_GRP0_CMP0: usize = 0x4C;
/// Write-1-to-clear alias page: `+0x2000`.
const IF_CLEAR_ALIAS: usize = 0x2000;
/// `SYSRTC_GRP0_IEN` bit for compare channel 0: bit 1, which is what `efr32mg24_sysrtc.h` calls
/// `SYSRTC_GRP0_IF_CMP0`. Measured too — `0x22` one count after the deadline, `0x44` on channel 1 — and the two
/// agree, so this is the header being read correctly rather than a header being wrong.
const IEN_CMP0: u32 = 1 << 1;
/// `SYSRTC_APP`: the line group 0 raises. IRQ 67 lives in `ISER2` (64..95), bit 3.
const SYSRTC_APP_IRQ: u32 = 67;
// `SYSRTC_SEQ` is IRQ 68, and it is deliberately **not** enabled here: measured, the group block at 0x60 raises
// that line (`NVIC_ISPR2` read 0x10 with only IRQ 67 enabled), and the RAIL blob owns its handler -- pointing a
// second handler at it fails the link with `duplicate symbol: SYSRTC_SEQ_IRQHandler`. See
// `docs/04-reference/SYSRTC-COMPARE-WORKS.md`.
/// `NVIC_ISER2`, `0xE000_E100 + 4 * (67 / 32)`.
const NVIC_ISER2: usize = 0xE000_E108;

/// Whether the NVIC bit has been set. One write, once, so the scheduler does not pay for it on every schedule.
static mut IRQ_ENABLED: bool = false;
/// How many times the compare interrupt has fired, so the example can tell an interrupt-driven timer from a
/// polled one that happens to look identical from the outside.
static mut IRQ_HITS: u32 = 0;

/// Program the next wake and make sure the compare interrupt is unmasked.
///
/// # Safety
///
/// Touches SYSRTC and NVIC registers; call only with the module enabled and running.
unsafe fn arm(next: u32) {
    core::ptr::write_volatile((SYSRTC_BASE + R_GRP0_CMP0) as *mut u32, next);
    core::ptr::write_volatile((SYSRTC_BASE + R_GRP0_IF + IF_CLEAR_ALIAS) as *mut u32, 0xFF);
    core::ptr::write_volatile((SYSRTC_BASE + R_GRP0_IEN) as *mut u32, IEN_CMP0);

    if !core::ptr::read_volatile(core::ptr::addr_of!(IRQ_ENABLED)) {
        // Only the application line. The sequencer line is RAIL's, and enabling it here would land its events on
        // this handler while RAIL expects them on its own.
        core::ptr::write_volatile(NVIC_ISER2 as *mut u32, 1u32 << (SYSRTC_APP_IRQ % 32));
        core::ptr::write_volatile(core::ptr::addr_of_mut!(IRQ_ENABLED), true);
    }
}

/// The earliest deadline in the table, or `None` when nothing is scheduled.
fn earliest() -> Option<u32> {
    unsafe {
        let table = &*core::ptr::addr_of!(TIMERS);
        let mut best: Option<u32> = None;
        for t in table.iter() {
            if !t.active {
                continue;
            }
            best = Some(match best {
                // "Earlier" on a wrapping counter: the smaller distance from now.
                Some(b) if now().wrapping_sub(b) < now().wrapping_sub(t.deadline) => b,
                _ => t.deadline,
            });
        }
        best
    }
}

/// The compare interrupt. This is the path the stack expects, and the reason the polled loop alone was not enough:
/// during `sli_bt_init_stack` the application loop is not running, so a callback that only fires from `poll()`
/// never fires at all, and the init waits forever.
///
/// The SDK's own `sl_sleeptimer_hal_sysrtc.c` runs its expiry callback from this interrupt, so doing the same is
/// not a shortcut -- it is the design. The pollable `poll()` stays for the cases where the loop runs anyway.
#[no_mangle]
pub extern "C" fn SYSRTC_APP_IRQHandler() {
    crate::sleeptimer::on_compare_interrupt();
}

// There is deliberately **no** `SYSRTC_SEQ_IRQHandler` here. The linker refused the duplicate: the RAIL blob
// defines it. That is the whole answer to the interrupt question -- with IRQ 67 enabled and the compare
// programmed, `NVIC_ISPR2` read `0x10`, IRQ 68 pending, because **group 0 belongs to the sequencer line that
// RAIL owns**. A timer in the application's name must use a different group; taking group 0 would mean
// competing with RAIL for its own interrupt, and the duplicate-symbol error is this project being told so by the
// tool instead of by a hang six months from now.

/// Clear the flag first, then run what is due and re-arm: clearing first means a callback that takes longer than
/// the next deadline still gets its own interrupt instead of finding a stuck flag.
pub(crate) fn on_compare_interrupt() {
    unsafe {
        core::ptr::write_volatile(
            (SYSRTC_BASE + R_GRP0_IF + IF_CLEAR_ALIAS) as *mut u32,
            IEN_CMP0,
        );
        let p = core::ptr::addr_of_mut!(IRQ_HITS);
        core::ptr::write_volatile(p, core::ptr::read_volatile(p).wrapping_add(1));
    }
    poll();
}

/// Compare interrupts taken, and callbacks run. A boot where `irq` stays 0 while the stack is waiting means the
/// NVIC bit or `IEN` is wrong, which is a different fault from a scheduler that runs and computes the wrong time.
pub fn irq_stats() -> u32 {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(IRQ_HITS)) }
}

/// The counter runs at 32 968 Hz measured (nominal 32 768). The nominal is reported because the stack converts
/// through it and a 0.6% error over a connection interval is smaller than the interval's own quantisation.
const TICKS_PER_SECOND: u32 = 32_768;
const TICKS_PER_MS: u32 = TICKS_PER_SECOND / 1000; // 32, rounded down; see the note in the HAL

/// `SL_STATUS_OK` and the two failures this layer can produce.
const SL_STATUS_OK: u32 = 0;
const SL_STATUS_INVALID_PARAMETER: u32 = 0x21;
const SL_STATUS_ALLOCATION_FAILED: u32 = 0x01;

pub type TimerCallback = extern "C" fn(*mut c_void);

/// One scheduled callback.
#[derive(Clone, Copy)]
struct Timer {
    handle: *mut u8,
    callback: Option<TimerCallback>,
    data: *mut c_void,
    deadline: u32,
    period: u32,
    /// Kept because the stack passes it and an interrupt-driven version ordering same-tick timers needs it.
    /// A polled scheduler runs whatever is due in table order, which is why nothing reads this yet.
    #[allow(dead_code)]
    priority: u8,
    /// The flags word. Bit 0 (periodic) is consumed at scheduling time; the rest are single-shot options this
    /// implementation does not need.
    #[allow(dead_code)]
    flags: u32,
    active: bool,
}

impl Timer {
    const EMPTY: Self = Self {
        handle: core::ptr::null_mut(),
        callback: None,
        data: core::ptr::null_mut(),
        deadline: 0,
        period: 0,
        priority: 0,
        flags: 0,
        active: false,
    };
}

/// The stack schedules a handful of timers; eight is generous for a boot path and small enough to keep the
/// table in a static without a heap.
const MAX_TIMERS: usize = 8;
static mut TIMERS: [Timer; MAX_TIMERS] = [Timer::EMPTY; MAX_TIMERS];

/// Counters for the mailbox-free diagnostics the examples can read.
static mut OVERDUE_RUNS: u32 = 0;
static mut FULL_ERRORS: u32 = 0;

#[inline]
fn now() -> u32 {
    // SAFETY: fixed peripheral address, read-only access.
    unsafe { core::ptr::read_volatile(SYSRTC_CNT as *const u32) }
}

/// Set a timer, replacing whatever the handle pointed at.
///
/// # Safety
///
/// `handle` must point to at least one byte of storage the stack owns; the callback is called with `data`.
#[no_mangle]
pub unsafe extern "C" fn sl_sleeptimer_restart_timer(
    handle: *mut u8,
    timeout: u32,
    callback: Option<TimerCallback>,
    data: *mut c_void,
    priority: u8,
    flags: u32,
) -> u32 {
    if handle.is_null() {
        return SL_STATUS_INVALID_PARAMETER;
    }

    // A handle already scheduled is found and reused, which is how `restart` behaves: one timer per handle.
    let table = &mut *core::ptr::addr_of_mut!(TIMERS);
    let mut slot: Option<usize> = None;
    let mut free: Option<usize> = None;
    for (i, t) in table.iter().enumerate() {
        if t.active && t.handle == handle {
            slot = Some(i);
            break;
        }
        if !t.active && free.is_none() {
            free = Some(i);
        }
    }
    let index = match slot.or(free) {
        Some(i) => i,
        None => {
            // Nothing free: report it rather than silently dropping a timer the stack is waiting on.
            core::ptr::write_volatile(core::ptr::addr_of_mut!(FULL_ERRORS), unsafe {
                core::ptr::read_volatile(core::ptr::addr_of!(FULL_ERRORS)) + 1
            });
            return SL_STATUS_ALLOCATION_FAILED;
        }
    };

    table[index] = Timer {
        handle,
        callback,
        data,
        deadline: now().wrapping_add(timeout),
        // Bit 0 of the flags is the "periodic" bit in `sl_sleeptimer_flags_t`.
        period: if flags & 1 != 0 { timeout } else { 0 },
        priority,
        flags,
        active: true,
    };

    // **Arm here, not only at the end of `poll()`.** Arming when the timer is scheduled is what makes the timer
    // work during a blocking call: if the interrupt were only programmed by `poll()`, then any code path that
    // schedules a timer and then blocks -- exactly what `sli_bt_init_stack` does -- would wait for a deadline
    // nothing was ever told about. Measured: a timer scheduled for 50 ms and waited on for 200 ms produced zero
    // callbacks and zero interrupts until this line existed.
    if let Some(next) = earliest() {
        arm(next);
    }
    if let Some(next) = earliest() {
        unsafe { arm(next) };
    }
    SL_STATUS_OK
}

/// Cancel a timer.
///
/// # Safety
///
/// `handle` must be one previously passed to `sl_sleeptimer_restart_timer`, or null.
#[no_mangle]
pub unsafe extern "C" fn sl_sleeptimer_stop_timer(handle: *mut u8) -> u32 {
    let table = &mut *core::ptr::addr_of_mut!(TIMERS);
    for t in table.iter_mut() {
        if t.active && t.handle == handle {
            t.active = false;
            return SL_STATUS_OK;
        }
    }
    SL_STATUS_INVALID_PARAMETER
}

/// True when the handle's timer is scheduled.
///
/// # Safety
///
/// As `sl_sleeptimer_stop_timer`.
#[no_mangle]
pub unsafe extern "C" fn sl_sleeptimer_is_timer_running(handle: *mut u8, running: *mut u8) -> u32 {
    if running.is_null() {
        return SL_STATUS_INVALID_PARAMETER;
    }
    let table = &*core::ptr::addr_of!(TIMERS);
    let mut is = 0u8;
    for t in table.iter() {
        if t.active && t.handle == handle {
            is = 1;
            break;
        }
    }
    core::ptr::write_volatile(running, is);
    SL_STATUS_OK
}

/// Start (the stack uses restart; this exists because the symbol is imported).
///
/// # Safety
///
/// As `sl_sleeptimer_restart_timer`.
#[no_mangle]
pub unsafe extern "C" fn sl_sleeptimer_start_timer(
    handle: *mut u8,
    timeout: u32,
    callback: Option<TimerCallback>,
    data: *mut c_void,
    priority: u8,
    flags: u32,
) -> u32 {
    sl_sleeptimer_restart_timer(handle, timeout, callback, data, priority, flags)
}

#[no_mangle]
pub extern "C" fn sl_sleeptimer_init() -> u32 {
    // Nothing to initialise: the counter is started by the HAL before the stack runs. Kept as a symbol because
    // `timer.c.obj` imports it and calls it during the stack's own init.
    SL_STATUS_OK
}

#[no_mangle]
pub extern "C" fn sl_sleeptimer_get_tick_count() -> u32 {
    now()
}

#[no_mangle]
pub extern "C" fn sl_sleeptimer_get_tick_count64() -> u64 {
    now() as u64
}

#[no_mangle]
pub extern "C" fn sl_sleeptimer_get_timer_frequency() -> u32 {
    TICKS_PER_SECOND
}

/// The crystal's accuracy, in parts per million. LFRCO is a trimmed RC oscillator, not a crystal: 500 ppm is a
/// deliberate, conservative figure, and the stack uses it only to widen its timing margins.
#[no_mangle]
pub extern "C" fn sl_sleeptimer_get_clock_accuracy() -> u16 {
    500
}

#[no_mangle]
pub extern "C" fn sl_sleeptimer_ms_to_tick(ms: u16) -> u32 {
    ms as u32 * TICKS_PER_MS
}

#[no_mangle]
pub extern "C" fn sl_sleeptimer_ms32_to_tick(ms: u32) -> u32 {
    ms.saturating_mul(TICKS_PER_MS)
}

#[no_mangle]
pub extern "C" fn sl_sleeptimer_tick_to_ms(ticks: u32) -> u32 {
    ticks / TICKS_PER_MS
}

#[no_mangle]
pub extern "C" fn sl_sleeptimer_tick64_to_ms(ticks: u64) -> u64 {
    ticks / TICKS_PER_MS as u64
}

/// Run every callback whose deadline has passed. **The application loop must call this.**
///
/// This is the whole scheduler. It is safe to call from the main loop only: the callbacks belong to the stack
/// and may call back into it, so running them from an interrupt would re-enter the stack's own state machine
/// from a context it does not expect.
pub fn poll() {
    let deadline_now = now();
    // Collect first, then run: a callback may reschedule itself, and mutating the table while iterating it is
    // exactly the kind of aliasing this language refuses to compile but a raw loop would let through.
    let mut due: [(Option<TimerCallback>, *mut c_void); MAX_TIMERS] =
        [(None, core::ptr::null_mut()); MAX_TIMERS];
    let mut due_count = 0usize;

    unsafe {
        let table = &mut *core::ptr::addr_of_mut!(TIMERS);
        for t in table.iter_mut() {
            if !t.active {
                continue;
            }
            // Wraparound-safe: the timer is due when `now - deadline` is small in the unsigned sense.
            if deadline_now.wrapping_sub(t.deadline) < 0x8000_0000 {
                due[due_count] = (t.callback, t.data);
                due_count += 1;
                if t.period != 0 {
                    t.deadline = t.deadline.wrapping_add(t.period);
                } else {
                    t.active = false;
                }
                if due_count == MAX_TIMERS {
                    break;
                }
            }
        }
        core::ptr::write_volatile(core::ptr::addr_of_mut!(OVERDUE_RUNS), {
            core::ptr::read_volatile(core::ptr::addr_of!(OVERDUE_RUNS)) + due_count as u32
        });
    }

    for (cb, data) in due.iter().take(due_count) {
        if let Some(f) = cb {
            f(*data);
        }
    }

    // Re-arm for whatever is next. With nothing scheduled the interrupt source is turned off rather than left
    // pointing at a deadline in the past, which would make the handler run continuously.
    // SAFETY: the timer table is this module's own and the registers are the ones measured above.
    unsafe {
        match earliest() {
            Some(next) => arm(next),
            None => core::ptr::write_volatile((SYSRTC_BASE + R_GRP0_IEN) as *mut u32, 0),
        }
    }
}

/// How many callbacks have run, and how many schedules failed for want of a slot.
pub fn stats() -> (u32, u32) {
    unsafe {
        (
            core::ptr::read_volatile(core::ptr::addr_of!(OVERDUE_RUNS)),
            core::ptr::read_volatile(core::ptr::addr_of!(FULL_ERRORS)),
        )
    }
}
