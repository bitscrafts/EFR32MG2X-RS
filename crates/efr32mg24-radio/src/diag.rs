//! Counting wrappers around the RAIL calls the Bluetooth link layer makes, for finding out what it asks the radio to do.
//!
//! Enabled by the `diag` feature, which makes `build.rs` pass `--wrap=<RAIL symbol>` to the linker: every call the
//! controller makes to `RAIL_X` lands in `__wrap_RAIL_X` here, which updates [`DIAG`] and forwards to `__real_RAIL_X`
//! with the same arguments and returns its result unchanged. The wrappers only count and copy arguments; the one
//! exception is the scheduled-transmit wrapper, which also reads `RAIL_GetTime` to record how far ahead of now the
//! link layer schedules.
//!
//! [`DIAG`] is a `#[no_mangle]` static in `.bss`, so its address comes from `nm` on the image that was flashed. It is
//! not at a fixed address on purpose: the top of RAM used to be shared with the stack.
//!
//! Self-tests (a zero from an instrument counts only if its self-test is non-zero in the same run):
//!
//! - `get_time` must grow by hundreds per second, because the scheduler reads the clock on every pass. A count of zero
//!   means the wrap did not take effect, not that the radio is idle.
//! - `config_events` must be at least 1: it is called once at boot, so it proves the link only.
//!
//! Layout, in `u32` words, as read with `mdw`:
//!
//! ```text
//!  0 magic 0xD1A60001      9 sched_last_status      18 sched_lead_max (as i32)
//!  1 get_time              10 sched_nonzero_status  19 sched_last_when
//!  2 config_events         11 sched_last_channel    20 sched_last_delta (when - previous when)
//!  3 start_tx              12 sched_last_mode       21 idle
//!  4 start_tx_last_status  13 sched_last_options    22 idle_last_mode
//!  5 start_tx_nonzero      14 sched_last_txduringrx 23 write_fifo
//!  6 start_tx_last_channel 15 sched_hist_inrange    24 write_last_len
//!  7 start_tx_last_options 16 (reserved)            25 write_captured_len
//!  8 sched                 17 sched_lead_min (i32)  26..34 first 32 bytes of the first FIFO write after a transmit
//! ```

use core::sync::atomic::{AtomicU32, Ordering::Relaxed};

const WORDS: usize = 36;

/// The counters, as a flat array so a single `mdw` reads them all.
#[no_mangle]
pub static DIAG: [AtomicU32; WORDS] = [const { AtomicU32::new(0) }; WORDS];

const MAGIC: usize = 0;
const GET_TIME: usize = 1;
const CONFIG_EVENTS: usize = 2;
const START_TX: usize = 3;
const START_TX_STATUS: usize = 4;
const START_TX_NONZERO: usize = 5;
const START_TX_CHANNEL: usize = 6;
const START_TX_OPTIONS: usize = 7;
const SCHED: usize = 8;
const SCHED_STATUS: usize = 9;
const SCHED_NONZERO: usize = 10;
const SCHED_CHANNEL: usize = 11;
const SCHED_MODE: usize = 12;
const SCHED_OPTIONS: usize = 13;
const SCHED_TXDURINGRX: usize = 14;
const SCHED_LEAD_MIN: usize = 17;
const SCHED_LEAD_MAX: usize = 18;
const SCHED_WHEN: usize = 19;
const SCHED_DELTA: usize = 20;
const IDLE: usize = 21;
const IDLE_MODE: usize = 22;
const WRITE: usize = 23;
const WRITE_LEN: usize = 24;
const WRITE_CAPTURED_LEN: usize = 25;
const WRITE_BYTES: usize = 26;
/// Calls to `RAIL_ScheduleRx`. A scanner that hears nothing either never gets here or gets here and nothing arrives.
const RX_SCHED: usize = 15;
/// The status of the last `RAIL_ScheduleRx`.
const RX_SCHED_STATUS: usize = 16;
/// Calls to `RAIL_ConfigRxOptions`.
const RX_CONFIG_OPTS: usize = 27;

/// Set when a transmit started and the next FIFO write has not been captured yet.
static ARMED: AtomicU32 = AtomicU32::new(0);

/// Mark the table as live and seed the lead range. Call once before the stack starts.
pub fn init() {
    DIAG[SCHED_LEAD_MIN].store(i32::MAX as u32, Relaxed);
    DIAG[SCHED_LEAD_MAX].store(i32::MIN as u32, Relaxed);
    DIAG[MAGIC].store(0xD1A6_0001, Relaxed);
}

/// One line for the mailbox: the headline counts, without needing a debugger.
///
/// Returns the number of bytes written to `out`.
pub fn summary(out: &mut [u8]) -> usize {
    let mut at = 0;
    for (tag, idx) in [
        (&b"gt="[..], GET_TIME),
        (&b" ce="[..], CONFIG_EVENTS),
        (&b" tx="[..], START_TX),
        (&b" st="[..], SCHED),
        (&b" id="[..], IDLE),
        (&b" wr="[..], WRITE),
    ] {
        for b in tag {
            if at < out.len() {
                out[at] = *b;
                at += 1;
            }
        }
        at = push_dec(out, at, DIAG[idx].load(Relaxed));
    }
    at
}

fn push_dec(out: &mut [u8], mut at: usize, mut value: u32) -> usize {
    let mut digits = [0u8; 10];
    let mut n = 0;
    loop {
        digits[n] = b'0' + (value % 10) as u8;
        value /= 10;
        n += 1;
        if value == 0 {
            break;
        }
    }
    for i in (0..n).rev() {
        if at < out.len() {
            out[at] = digits[i];
            at += 1;
        }
    }
    at
}

extern "C" {
    fn __real_RAIL_GetTime() -> u32;
    fn __real_RAIL_ConfigEvents(handle: *mut core::ffi::c_void, mask: u64, events: u64) -> u8;
    fn __real_RAIL_StartTx(
        handle: *mut core::ffi::c_void,
        channel: u16,
        options: u32,
        info: *const core::ffi::c_void,
    ) -> u8;
    fn __real_RAIL_StartScheduledTx(
        handle: *mut core::ffi::c_void,
        channel: u16,
        options: u32,
        config: *const ScheduleTxConfig,
        info: *const core::ffi::c_void,
    ) -> u8;
    fn __real_RAIL_Idle(handle: *mut core::ffi::c_void, mode: u8, wait: bool);
    /// The schedule-RX config and the scheduler info are forwarded untouched: this wrapper counts and forwards, it
    /// does not interpret. Declaring them opaque is therefore honest rather than lazy -- there is no field here that
    /// this code claims to understand.
    fn __real_RAIL_ScheduleRx(
        handle: *mut core::ffi::c_void,
        channel: u16,
        cfg: *const core::ffi::c_void,
        scheduler_info: *const core::ffi::c_void,
    ) -> u8;
    /// `RAIL_RxOptions_t` is a 32-bit option mask in `rail.h`.
    fn __real_RAIL_ConfigRxOptions(handle: *mut core::ffi::c_void, mask: u32, options: u32) -> u8;
    fn __real_RAIL_WriteTxFifo(
        handle: *mut core::ffi::c_void,
        data: *const u8,
        len: u16,
        reset: bool,
    ) -> u16;
}

/// `RAIL_ScheduleTxConfig_t`: `when` (u32), `mode` (u8 enum), `txDuringRx` (u8 enum).
#[repr(C)]
pub struct ScheduleTxConfig {
    when: u32,
    mode: u8,
    tx_during_rx: u8,
}

#[no_mangle]
pub unsafe extern "C" fn __wrap_RAIL_GetTime() -> u32 {
    DIAG[GET_TIME].fetch_add(1, Relaxed);
    __real_RAIL_GetTime()
}

#[no_mangle]
pub unsafe extern "C" fn __wrap_RAIL_ConfigEvents(
    handle: *mut core::ffi::c_void,
    mask: u64,
    events: u64,
) -> u8 {
    DIAG[CONFIG_EVENTS].fetch_add(1, Relaxed);
    __real_RAIL_ConfigEvents(handle, mask, events)
}

#[no_mangle]
pub unsafe extern "C" fn __wrap_RAIL_StartTx(
    handle: *mut core::ffi::c_void,
    channel: u16,
    options: u32,
    info: *const core::ffi::c_void,
) -> u8 {
    DIAG[START_TX].fetch_add(1, Relaxed);
    DIAG[START_TX_CHANNEL].store(u32::from(channel), Relaxed);
    DIAG[START_TX_OPTIONS].store(options, Relaxed);
    ARMED.store(1, Relaxed);
    let status = __real_RAIL_StartTx(handle, channel, options, info);
    DIAG[START_TX_STATUS].store(u32::from(status), Relaxed);
    if status != 0 {
        DIAG[START_TX_NONZERO].store(u32::from(status), Relaxed);
    }
    status
}

#[no_mangle]
pub unsafe extern "C" fn __wrap_RAIL_StartScheduledTx(
    handle: *mut core::ffi::c_void,
    channel: u16,
    options: u32,
    config: *const ScheduleTxConfig,
    info: *const core::ffi::c_void,
) -> u8 {
    DIAG[SCHED].fetch_add(1, Relaxed);
    DIAG[SCHED_CHANNEL].store(u32::from(channel), Relaxed);
    DIAG[SCHED_OPTIONS].store(options, Relaxed);
    if !config.is_null() {
        let when = (*config).when;
        DIAG[SCHED_MODE].store(u32::from((*config).mode), Relaxed);
        DIAG[SCHED_TXDURINGRX].store(u32::from((*config).tx_during_rx), Relaxed);
        let delta = when.wrapping_sub(DIAG[SCHED_WHEN].load(Relaxed));
        DIAG[SCHED_DELTA].store(delta, Relaxed);
        DIAG[SCHED_WHEN].store(when, Relaxed);
        let lead = when.wrapping_sub(__real_RAIL_GetTime()) as i32;
        if lead < DIAG[SCHED_LEAD_MIN].load(Relaxed) as i32 {
            DIAG[SCHED_LEAD_MIN].store(lead as u32, Relaxed);
        }
        if lead > DIAG[SCHED_LEAD_MAX].load(Relaxed) as i32 {
            DIAG[SCHED_LEAD_MAX].store(lead as u32, Relaxed);
        }
    }
    ARMED.store(1, Relaxed);
    let status = __real_RAIL_StartScheduledTx(handle, channel, options, config, info);
    DIAG[SCHED_STATUS].store(u32::from(status), Relaxed);
    if status != 0 {
        DIAG[SCHED_NONZERO].store(u32::from(status), Relaxed);
    }
    status
}

#[no_mangle]
pub unsafe extern "C" fn __wrap_RAIL_Idle(handle: *mut core::ffi::c_void, mode: u8, wait: bool) {
    DIAG[IDLE].fetch_add(1, Relaxed);
    DIAG[IDLE_MODE].store(u32::from(mode), Relaxed);
    __real_RAIL_Idle(handle, mode, wait)
}

#[no_mangle]
pub unsafe extern "C" fn __wrap_RAIL_WriteTxFifo(
    handle: *mut core::ffi::c_void,
    data: *const u8,
    len: u16,
    reset: bool,
) -> u16 {
    DIAG[WRITE].fetch_add(1, Relaxed);
    DIAG[WRITE_LEN].store(u32::from(len), Relaxed);
    // The first write after a transmit started: keep its first 32 bytes, once per transmit.
    if !data.is_null() && ARMED.swap(0, Relaxed) == 1 {
        DIAG[WRITE_CAPTURED_LEN].store(u32::from(len), Relaxed);
        for w in 0..8usize {
            let mut word = 0u32;
            for b in 0..4usize {
                let i = w * 4 + b;
                if i < usize::from(len) {
                    word |= u32::from(core::ptr::read(data.add(i))) << (8 * b);
                }
            }
            DIAG[WRITE_BYTES + w].store(word, Relaxed);
        }
    }
    __real_RAIL_WriteTxFifo(handle, data, len, reset)
}

/// Count the link layer's receive scheduling.
///
/// A scanner that reports nothing has two possible causes and they call for opposite work: the link layer never asks
/// the radio to receive, or it asks and no packet ever arrives. This counter is the difference. It reads 0 in a
/// firmware whose scanner "started successfully", which is what the board did before the scanner component was added
/// to the catalogue.
#[no_mangle]
pub unsafe extern "C" fn __wrap_RAIL_ScheduleRx(
    handle: *mut core::ffi::c_void,
    channel: u16,
    cfg: *const core::ffi::c_void,
    scheduler_info: *const core::ffi::c_void,
) -> u8 {
    DIAG[RX_SCHED].fetch_add(1, Relaxed);
    let status = __real_RAIL_ScheduleRx(handle, channel, cfg, scheduler_info);
    DIAG[RX_SCHED_STATUS].store(u32::from(status), Relaxed);
    status
}

/// Count the receive-option changes, which is how the link layer turns filtering and address matching on.
#[no_mangle]
pub unsafe extern "C" fn __wrap_RAIL_ConfigRxOptions(
    handle: *mut core::ffi::c_void,
    mask: u32,
    options: u32,
) -> u8 {
    DIAG[RX_CONFIG_OPTS].fetch_add(1, Relaxed);
    __real_RAIL_ConfigRxOptions(handle, mask, options)
}
