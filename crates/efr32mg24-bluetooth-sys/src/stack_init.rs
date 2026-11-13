//! The vendor's stack entry points, which are not the public commands.
//!
//! ## Why this exists
//!
//! The obvious way to start Bluetooth is `sl_bt_system_start_bluetooth()`. On this SDK configuration
//! it returns **`SL_STATUS_NOT_AVAILABLE` (0x000E)** — measured on the board, reported through the
//! mailbox as `scan: boot failed bgapi14`. The command exists in the blob because the blob is built
//! from a superset, but the *feature* behind it (the "Bluetooth on-demand start" component) is not
//! part of this build, so the stack refuses it at runtime. A return code is the only evidence; the
//! link succeeds.
//!
//! What this build does expect is the pair of internal entry points that the SDK's generated
//! `sl_bt_stack_init()` calls:
//!
//! ```text
//! sli_bt_init_stack(&config, …)   initialise the host with a configuration
//! sli_bt_start_stack()            start it; the stack then emits system_boot
//! ```
//!
//! Both are exported by `libbluetooth_host` (verified with `arm-none-eabi-nm`), and neither is in the
//! headers bindgen saw, because the allowlist matches `sl_bt_*` and these are `sli_bt_*`. The
//! configuration type *is* in the headers (`sl_bt_stack_config.h`), and it is reproduced here.
//!
//! ## The argument that matters
//!
//! `sli_bt_init_stack` reads `config->config_flags` before anything else — the disassembly tests bit
//! 8 (`SL_BT_CONFIG_FLAG_RTOS`) on the first word — so the struct must be laid out exactly as the
//! header declares it. Getting that wrong is not a compile error; it is a boot that fails with a
//! status code that names a different problem.

use core::ffi::c_void;

/// `sl_bt_stack_config_t` — buffer budget and link-layer options.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StackConfig {
    /// Maximum number of bytes for the stack's data buffers.
    pub max_buffer_memory: u32,
    /// Link-layer configuration word; 0 selects the defaults.
    pub linklayer_config: u32,
    /// Optional priority overrides; null uses the built-in ones.
    pub linklayer_priorities: *mut c_void,
}

/// `sl_bt_rf_config_t` — gains and antenna selection.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RfConfig {
    pub tx_gain: i16,
    pub rx_gain: i16,
    pub flags: u8,
    pub antenna: u8,
    pub tx_min_power: i16,
    pub tx_max_power: i16,
}

/// `sl_bt_configuration_t` — what the host is initialised with.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Configuration {
    pub config_flags: u32,
    pub bluetooth: StackConfig,
    /// The GATT database. Null is accepted for advertising-only builds; a connectable product needs
    /// a real database, which is what `docs/04-reference/BLE-GATT-GAP.md` is about.
    pub gattdb: *const c_void,
    pub scheduler_callback: Option<extern "C" fn()>,
    pub stack_schedule_callback: Option<extern "C" fn()>,
    pub max_timers: u8,
    pub rf: RfConfig,
}

/// The one configuration this program will ever use.
static mut CONFIGURATION: Configuration = default_configuration();

extern "C" {
    /// `sli_bt_init_stack` — the host's initialisation, three arguments wide.
    ///
    /// Declared and unused: calling it with NULL feature and class tables was this project's original mistake, and
    /// keeping the declaration records the shape that made the mistake possible.
    #[allow(dead_code)]
    ///
    /// The second and third are the SDK's optional scheduling hooks; both null selects the
    /// interrupt-driven defaults, which is what a bare-metal application wants.
    fn sli_bt_init_stack(
        config: *const Configuration,
        response_callback: *mut c_void,
        event_callback: *mut c_void,
    ) -> u32;

    /// `sl_bt_stack_init` — the SDK's own initialisation entry point, from `sl_bt_stack_init.c`.
    ///
    /// It takes no arguments and supplies the three that `sli_bt_init_stack` needs: the configuration, the
    /// feature-use table and the BGAPI class table. Calling `sli_bt_init_stack` directly with NULLs -- which this
    /// project did, believing the last two were optional scheduling hooks -- initialises the host without
    /// registering any class, so it returns success and then emits nothing at all.
    fn sl_bt_stack_init() -> u32;

    /// `sli_bt_start_stack` — start the host; `system_boot` follows as an event.
    ///
    /// Kept declared and unused on purpose: calling it is what produced `RAIL_ASSERT_INVALID_XTAL_FREQUENCY` with a
    /// NULL handle, so it is the exact experiment to re-run once the handle question below is answered.
    #[allow(dead_code)]
    fn sli_bt_start_stack() -> u32;
}

/// Initialise and start the Bluetooth host with the defaults a bare-metal build wants.
///
/// Where the bring-up got to, and what each step returned.
///
/// Written to a fixed RAM word because this module has no mailbox: a caller that stalls inside `start()` can still
/// be diagnosed by reading `0x2003_F600` with a debugger, which is how the difference between "RAIL refused our
/// configuration" and "the stack reconfigured RAIL itself" was found.
const TRACE: usize = 0x2003_F600;

fn trace(slot: u32, value: u32) {
    unsafe {
        core::ptr::write_volatile((TRACE + (slot as usize) * 4) as *mut u32, value);
    }
}

/// Initialise the radio and the Bluetooth host, and report the status of whichever step failed.
///
/// # Safety
///
/// Must be called once, before any other BGAPI command, with the radio's clocks available. The blobs keep the
/// configuration pointer, so the value is given a static home and must remain the only one in use.
pub unsafe fn start(config: Configuration) -> u32 {
    // **The stack owns RAIL, and that order is forced by RAIL's own contract.**
    //
    // `rail.h` says of `RAIL_Init`: "Call this function only once per protocol. If called again, it will do nothing
    // and return NULL." The Bluetooth host calls it during its initialisation. This module used to call it first,
    // which meant the host got NULL back, stored NULL, and later passed NULL to `RAIL_SetTune` -- and RAIL asserted
    // `RAIL_ASSERT_INVALID_XTAL_FREQUENCY` with a NULL handle, which read as "the crystal frequency is wrong" while
    // the real fault was that two owners existed.
    //
    // So: the host initialises the radio, and the application configures the channels afterwards, through the
    // generic handle `RAIL_EFR32_HANDLE` (`0xFFFFFFFF`) that this single-instance part uses. The frequency itself
    // still has to be known to RAIL before it configures the radio, which is why the HFXO is started first --
    // without it the device reports zero hertz and the assertion is genuine.
    {
        efr32mg24_emlib_sys::emlib_hfxo_start();
        efr32mg24_emlib_sys::emlib_clock_select_hfxo();
        trace(0, 1);
    }

    // The blobs may keep the pointer, so it is given a static home rather than a stack slot. A
    // `Box` would need an allocator this crate does not have; a `static mut` accessed once, under
    // the caller's single-owner contract, is the honest equivalent.
    // The configuration the SDK's wrapper uses is `bt_config` inside `sl_bt_stack_init.c`; the value passed here is
    // kept for callers that want to see it, but the host reads its own. That is why `SL_BT_CONFIG_DEFAULT` in the
    // vendored file is the thing to change for buffer sizes and connection counts.
    CONFIGURATION = config;
    trace(4, 0xFFFF_FFFF); // about to enter the host's initialisation
    let status = sl_bt_stack_init();
    trace(5, status);
    if status != 0 {
        return status;
    }

    // Channels, on the generic handle, now that the host has created the RAIL instance.
    // SAFETY: called once, after the host's initialisation and before it uses the radio.
    unsafe {
        let channels = efr32mg24_rail_sys::ble_config::config();
        trace(2, channels.xtalFrequencyHz);
        let chan_status = efr32mg24_rail_sys::RAIL_ConfigChannels(
            0xFFFF_FFFF as efr32mg24_rail_sys::RAIL_Handle_t,
            channels,
            None,
        ) as u32;
        trace(3, chan_status);
        if chan_status != 0 {
            return chan_status;
        }
    }
    // **`sli_bt_start_stack` is not called.** Measured: `sli_bt_init_stack` returns 0 -- the host initialises --
    // and then `sli_bt_start_stack` returns a value that looks like an uninitialised register (0xc2a3de54) and RAIL
    // asserts 66 with a NULL handle, which is the signature of something reconfiguring the radio without a
    // configuration. The host's own event queue is what carries `system_boot`, so the question worth answering
    // first is whether init alone is enough. Leaving the call out is a measurement, not a decision: if `system_boot`
    // never appears this is the line to put back.
    trace(6, 0xFFFF_FFFF);
    // **Started, because the event queue needs it.** Measured with the SDK's own initialiser in place: init returns
    // 0, a BGAPI command (`sl_bt_system_get_version`) also returns 0 -- the host runs -- but `sl_bt_pop_event`
    // answers 45 (`SL_STATUS_NOT_FOUND`) and `system_boot` never arrives. The earlier attempt to call this produced
    // a RAIL assert, and that attempt was made when the initialiser had no feature or BGAPI class tables; with them
    // present the call is worth repeating, and the trace records what it returns either way.
    let started = sli_bt_start_stack();
    trace(7, started);
    started
}

/// The configuration a bare-metal advertiser needs.
///
/// The numbers are the SDK's own defaults where a default exists: no RTOS flag, no antenna switch and no
/// gain trim. Buffer memory and soft timers were raised from 4 KiB / 4 after `sli_bt_init_stack` answered
/// `SL_STATUS_NO_MORE_RESOURCE` (0x001A) with the smaller budget -- measured, not guessed.
#[must_use]
pub const fn default_configuration() -> Configuration {
    Configuration {
        config_flags: 0,
        bluetooth: StackConfig {
            max_buffer_memory: 8192,
            linklayer_config: 0,
            linklayer_priorities: core::ptr::null_mut(),
        },
        gattdb: core::ptr::null(),
        scheduler_callback: None,
        stack_schedule_callback: None,
        max_timers: 8,
        rf: RfConfig {
            tx_gain: 0,
            rx_gain: 0,
            flags: 0,
            antenna: 0,
            tx_min_power: -260,
            tx_max_power: 200,
        },
    }
}
