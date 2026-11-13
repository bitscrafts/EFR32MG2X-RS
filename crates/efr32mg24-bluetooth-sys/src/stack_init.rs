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
    /// The second and third are the SDK's optional scheduling hooks; both null selects the
    /// interrupt-driven defaults, which is what a bare-metal application wants.
    fn sli_bt_init_stack(
        config: *const Configuration,
        response_callback: *mut c_void,
        event_callback: *mut c_void,
    ) -> u32;

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
    // **RAIL first, and it needs two things this project had to measure.**
    //
    // 1. **HFXO running and selected.** RAIL compares the configuration's `xtalFrequencyHz` against the frequency
    //    the device reports; an HFXO that was never started reports zero, so the comparison fails and RAIL asserts
    //    `RAIL_ASSERT_INVALID_XTAL_FREQUENCY` (code 66) even when the table says 39 MHz. Measured: with the table
    //    alone the assert is 66; with HFXO started first, `RAIL_ConfigChannels` returns `RAIL_STATUS_NO_ERROR`.
    // 2. **A channel configuration**, because the Bluetooth blobs never call `RAIL_ConfigChannels` themselves
    //    (checked: zero imports across all three archives). The configuration is the SDK's own generated table for
    //    this part at 39 MHz, emitted into Rust in `efr32mg24_rail_sys::ble_config`.
    //
    // Both belong here rather than in each example: a caller that forgets either gets a hang with no evidence.
    // SAFETY: called once, before any other BGAPI command; the radio's clocks exist by construction.
    {
        efr32mg24_emlib_sys::emlib_hfxo_start();
        efr32mg24_emlib_sys::emlib_clock_select_hfxo();
        trace(0, 1);

        let mut rail_config: efr32mg24_rail_sys::RAIL_Config_t = core::mem::zeroed();
        let handle = efr32mg24_rail_sys::RAIL_Init(&mut rail_config, None);
        trace(1, handle as usize as u32);
        if handle.is_null() {
            return SL_STATUS_FAIL;
        }
        let channels = efr32mg24_rail_sys::ble_config::config();
        trace(2, channels.xtalFrequencyHz);
        let status = efr32mg24_rail_sys::RAIL_ConfigChannels(handle, channels, None) as u32;
        trace(3, status);
        if status != 0 {
            return status;
        }
    }

    // The blobs may keep the pointer, so it is given a static home rather than a stack slot. A
    // `Box` would need an allocator this crate does not have; a `static mut` accessed once, under
    // the caller's single-owner contract, is the honest equivalent.
    CONFIGURATION = config;
    let config_ptr = core::ptr::addr_of!(CONFIGURATION);
    trace(4, 0xFFFF_FFFF); // about to enter the host's initialisation
    let status = sli_bt_init_stack(config_ptr, core::ptr::null_mut(), core::ptr::null_mut());
    trace(5, status);
    if status != 0 {
        return status;
    }
    // **`sli_bt_start_stack` is not called.** Measured: `sli_bt_init_stack` returns 0 -- the host initialises --
    // and then `sli_bt_start_stack` returns a value that looks like an uninitialised register (0xc2a3de54) and RAIL
    // asserts 66 with a NULL handle, which is the signature of something reconfiguring the radio without a
    // configuration. The host's own event queue is what carries `system_boot`, so the question worth answering
    // first is whether init alone is enough. Leaving the call out is a measurement, not a decision: if `system_boot`
    // never appears this is the line to put back.
    trace(6, 0xFFFF_FFFF);
    trace(7, 0x600D_0000);
    0
}

/// `SL_STATUS_FAIL`, for the one early return above.
const SL_STATUS_FAIL: u32 = 0x0001;

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
