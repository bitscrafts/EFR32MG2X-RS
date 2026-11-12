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
    fn sli_bt_start_stack() -> u32;
}

/// Initialise and start the Bluetooth host with the defaults a bare-metal build wants.
///
/// Returns the BGAPI status of whichever step failed, so the caller can report it instead of
/// guessing. `0` means both calls accepted.
///
/// # Safety
///
/// Must be called once, before any other BGAPI command, with the radio's clock available. The blobs
/// keep the configuration pointer, so the value must outlive the call — hence a leaked `'static`.
pub unsafe fn start(config: Configuration) -> u32 {
    // The blobs may keep the pointer, so it is given a static home rather than a stack slot. A
    // `Box` would need an allocator this crate does not have; a `static mut` accessed once, under
    // the caller's single-owner contract, is the honest equivalent.
    CONFIGURATION = config;
    let config_ptr = core::ptr::addr_of!(CONFIGURATION);
    let status = sli_bt_init_stack(config_ptr, core::ptr::null_mut(), core::ptr::null_mut());
    if status != 0 {
        return status;
    }
    sli_bt_start_stack()
}

/// The configuration a bare-metal advertiser needs.
///
/// The numbers are the SDK's own defaults where a default exists: 4 KiB of buffer memory, no RTOS
/// flag, four soft timers, no antenna switch and no gain trim.
#[must_use]
pub const fn default_configuration() -> Configuration {
    Configuration {
        config_flags: 0,
        bluetooth: StackConfig {
            max_buffer_memory: 4096,
            linklayer_config: 0,
            linklayer_priorities: core::ptr::null_mut(),
        },
        gattdb: core::ptr::null(),
        scheduler_callback: None,
        stack_schedule_callback: None,
        max_timers: 4,
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
