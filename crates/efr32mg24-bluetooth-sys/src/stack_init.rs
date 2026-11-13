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

// ============================================================================
// The bare-metal adaptation layer
// ============================================================================
//
// **Why this exists, and why the events were missing.**
//
// The vendor ships two ways to run the Bluetooth host: with an RTOS, and without. The vendor blobs in this crate were
// built with an RTOS, so the host delivers its events through two callbacks it is given at initialisation --
// exactly the pair `SL_BT_CONFIG_DEFAULT` fills with `sli_bt_rtos_ll_callback` and `sli_bt_rtos_stack_callback`
// when `SL_CATALOG_KERNEL_PRESENT` is defined, and with NULL when it is not.
//
// The RTOS side of that contract lives in `protocol/bluetooth/src/sl_bt_rtos_adaptation.c`, which is application
// code, not part of any blob. With the callbacks left NULL -- which is what this crate did, because its
// configuration is compiled without a kernel -- the host has no way to tell the application that work is waiting.
// Nothing crashes: the host initialises, answers commands, and never queues a single event. That is the wall the
// examples were stuck behind.
//
// The adaptation is small enough to write out. `bluetooth_thread` in that file is, in essence:
//
//     create the link-layer thread, create the event thread,
//     sli_bt_system_start_bluetooth();
//
// and the link-layer thread is a loop around `sl_bt_priority_handle()` woken by the first callback. Both of those
// functions are in the host blob (verified with `nm`), so a bare-metal build needs only the flags and a pump.

use core::sync::atomic::{AtomicBool, Ordering};

/// Set by the host when the link layer wants servicing. This is `scheduler_callback` in the configuration.
static LINK_LAYER_WAITING: AtomicBool = AtomicBool::new(false);

/// Set by the host when it has work for the application. This is `stack_schedule_callback`.
static STACK_WAITING: AtomicBool = AtomicBool::new(false);

/// How many times each callback fired. Published so a run can say whether the host ever called back at all.
static LINK_LAYER_CALLBACKS: AtomicBool = AtomicBool::new(false);
static STACK_CALLBACKS: AtomicBool = AtomicBool::new(false);

extern "C" fn on_link_layer_wakeup() {
    LINK_LAYER_WAITING.store(true, Ordering::Release);
    LINK_LAYER_CALLBACKS.store(true, Ordering::Release);
}

extern "C" fn on_stack_wakeup() {
    STACK_WAITING.store(true, Ordering::Release);
    STACK_CALLBACKS.store(true, Ordering::Release);
}

extern "C" {
    /// The link-layer servicing function the vendor's own `linklayer_thread` calls.
    fn sl_bt_priority_handle();
    /// The host's own processing: GAP/GATT state and its software timers. The vendor's `sl_bt_step` is
    /// `sl_bt_run()` + `sl_bt_event_pending_len()` + `sl_bt_pop_event()`, and `sl_bt_run` is the part that makes the
    /// host advance; without it a started stack answers synchronous commands and queues nothing.
    fn sl_bt_run();
    /// Bytes in the pending event, or 0 if none is waiting.
    fn sl_bt_event_pending_len() -> u32;
    /// The host's *internal* start. Not the BGAPI `sl_bt_system_start_bluetooth`, which answers
    /// `SL_STATUS_NOT_AVAILABLE` without the on-demand-start component: this is the one the vendor's
    /// `bluetooth_thread` calls when it starts the stack, and it is what generates `system_boot`.
    fn sli_bt_system_start_bluetooth() -> u32;
}

/// Service the link layer now. This is what the vendor's bare-metal build wires to `PendSV`, and what its
/// `linklayer_thread` calls when a kernel is present:
///
/// ```c
/// void PendSV_Handler(void) { sl_bt_priority_handle(); }
/// ```
///
/// Measured on this board: with no `PendSV` handler defined, the very first request from the host lands in
/// `DefaultHandler` and the firmware sits there — `ICSR.VECTACTIVE = 14` with `PC` inside `DefaultHandler_`, no fault
/// and no status, which is the "hang" this project chased for hours.
pub fn service_link_layer() {
    unsafe { sl_bt_priority_handle() };
}

/// Service the host. Call this from the application's main loop, as often as it comes round.
///
/// One call performs one unit of link-layer work if the host asked for it. Events are *not* consumed here: the
/// application pops them with `sl_bt_pop_event`, which is where it wants them anyway.
pub fn pump() {
    if LINK_LAYER_WAITING.swap(false, Ordering::AcqRel) {
        unsafe { sl_bt_priority_handle() };
    }
    // **The host's turn.** Measured: with only the link layer serviced, `sli_bt_system_start_bluetooth` returns 0 and
    // the event queue is still empty (`sl_bt_pop_event` = 45 = `SL_STATUS_NOT_FOUND`), so `system_boot` -- which the
    // start enqueues -- is only delivered once the host runs.
    unsafe { sl_bt_run() };
}

/// Whether an event is waiting to be popped. The application pops it; this only reports.
pub fn event_pending() -> bool {
    unsafe { sl_bt_event_pending_len() > 0 }
}

/// What the host has asked for since boot, for diagnostics: `(link_layer_callbacks, stack_callbacks, link_layer_pending)`.
pub fn adaptation_stats() -> (bool, bool, bool) {
    (
        LINK_LAYER_CALLBACKS.load(Ordering::Acquire),
        STACK_CALLBACKS.load(Ordering::Acquire),
        LINK_LAYER_WAITING.load(Ordering::Acquire),
    )
}

// The RAIL configuration structure that used to live here is gone with the application's own `RAIL_Init`: the
// controller builds its own `RAIL_Config_t` in `.bss.railConfig`, with its own event handler, and calls
// `RAIL_Init` itself. Two callers of a function documented to work "only once per protocol" is the defect that
// produced a NULL handle and a fault at `[NULL + 0x138]`.

/// The one host configuration this program will ever use.
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
    // Four measurements, in the order the vendor's own initialiser puts them, with a trace word per step so that a
    // failure names itself instead of hiding behind a hang.
    //
    //   0  HFXO running        1  RAIL instance         2  table's crystal       3  channel config status
    //   4  entering the host   5  host initialisation   6  entering the start    7  start status
    //
    // **Why the application initialises RAIL at all.** `RAIL_ChannelConfig_t.xtalFrequencyHz` is the only place RAIL
    // learns the crystal, and only `RAIL_ConfigChannels` accepts it. Nothing in the Bluetooth blobs calls that
    // function (verified: zero imports across all four), so somebody else has to -- and the vendor's own answer is
    // `sl_rail_util_init.c`, which does `RAIL_Init` then `RAIL_ConfigData` then `RAIL_ConfigChannels` from the
    // application, before the stack is started.
    {
        efr32mg24_emlib_sys::emlib_hfxo_start();
        efr32mg24_emlib_sys::emlib_clock_select_hfxo();
        trace(0, 1);
    }

    // **The radio is the controller's, entirely.**
    //
    // Measured, after three wrong attempts of my own: the Bluetooth controller fills its own `RAIL_Config_t` in
    // `.bss.railConfig` (including `eventsCallback = ll_radioRailEventHandler`) and calls `RAIL_Init` itself inside
    // `ll_radioInit`. `RAIL_Init` is documented to "return NULL if called again", so an application that calls it
    // first makes the controller store a NULL handle -- and the very next `RAIL_ConfigEvents` faults reading
    // `[NULL + 0x138]`, which is exactly the fault this module kept producing.
    //
    // So the application does not call `RAIL_Init` and does not configure channels. (An earlier version of this
    // comment claimed "the crystal frequency travels inside the chip-specific configuration they carry"; that was
    // not measured, and the controller's PHY functions pass library-internal configs instead -- see the review in
    // docs/04-reference/BLE-OPUS-REVIEW.md. That claim is removed rather than repeated.)
    // `sl_rail_util_pa_init()` was tried here and is not the missing piece: it pulls `RAILCb_PaAutoModeDecision`,
    // which the PA component expects the application to define, and the radio bring-up fails identically without it.
    // Left out rather than half-wired.

    trace(2, 0);
    trace(3, 0);

    // The host. `sl_bt_stack_init` is the vendor's wrapper and supplies the feature-use and BGAPI class tables that
    // `sli_bt_init_stack` requires; calling the three-argument form with NULLs initialises a host that registers
    // nothing and never emits an event.
    CONFIGURATION = config;
    trace(4, 0xFFFF_FFFF);
    let status = sl_bt_stack_init();
    trace(5, status);
    if status != 0 {
        return status;
    }

    // **The real start.** `sli_bt_start_stack` is the wrong function (it returned `0x0483`, a value no header
    // defines, and produced no event). The vendor's own `bluetooth_thread` calls `sli_bt_system_start_bluetooth()`
    // to start the host, and the BGAPI command `sl_bt_system_start_bluetooth` is a different thing entirely: it
    // exists only in on-demand-start builds and answers `SL_STATUS_NOT_AVAILABLE` here, which is correct.
    trace(6, 0xFFFF_FFFF);
    // **Three questions answered by one flash, because the low byte cannot tell them apart.**
    //
    // `bg_radio_init` computes `BTLE_LL_Init() & 0xFF` and replaces non-zero with the fixed `0x040C`, so the status
    // that reaches us has lost the detail. Both candidates inside the controller end in `0x07`:
    //   * `ll_radioInit` returns **7** when `RAIL_SetTxFifo` is not 512;
    //   * `ll_randomInit` -> `bg_random_get_data` -> a refused `psa_generate_random` becomes **0x0407**
    //     (`SL_STATUS_BT_CRYPTO`), whose low byte is also 7.
    //
    // So: count the RNG requests the host made during this call, and read the two globals the controller fills --
    // `ll_radio + 0x28` is its RAIL handle, `ll_radio_pa + 20` is `0xFFFF` only if the PA stage succeeded.
    let rng_before = crate::psa::stats().2;
    let started = sli_bt_system_start_bluetooth();
    trace(7, started);
    unsafe {
        extern "C" {
            static ll_radio: [u32; 16];
            static ll_radio_pa: [u16; 12];
        }
        trace(1, crate::psa::stats().2.wrapping_sub(rng_before));
        trace(
            2,
            core::ptr::read_volatile(core::ptr::addr_of!(ll_radio[10])),
        );
        trace(
            3,
            core::ptr::read_volatile(core::ptr::addr_of!(ll_radio_pa[10])) as u32,
        );
    }
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
        scheduler_callback: Some(on_link_layer_wakeup),
        stack_schedule_callback: Some(on_stack_wakeup),
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
