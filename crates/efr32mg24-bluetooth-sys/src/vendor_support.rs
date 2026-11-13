//! The vendor entry points that are neither emlib nor stack, answered in Rust.
//!
//! Three groups, each with the reason it is here rather than vendored as C:
//!
//! 1. **`sli_bt_init_controller_features`** — in the SDK this is a dispatcher over `sl_btctrl_init_*` calls,
//!    every one of them guarded by a catalogue macro. Measured: `sl_btctrl_init_adv`, `_scan`, `_conn` and
//!    `_phy` all exist in the controller blob we link, so this is a faithful Rust transcription of the four
//!    calls a single-protocol peripheral build makes — not an invention.
//!
//! 2. **`sli_power_manager_update_*`** — in the SDK these maintain the power manager's requirements table so the
//!    device may enter EM2. This firmware never sleeps (the polled timer cannot wake us), so the requirements
//!    are counted and not acted on. That is a deliberate, visible limitation, not an oversight: `stats()`
//!    reports the count so a future power manager has something to start from.
//!
//! 3. **`nvm3_*`** — Gecko SDK v4.4.5 ships NVM3's *core* as a prebuilt library, not source (`nvm3.c` is absent
//!    from the package while `nvm3_default.c` is present), so there is nothing to vendor. This implements the
//!    ABI the stack calls against, with **no persistence**: `readData` always reports "key not found", which is
//!    exactly what an empty NVM looks like. Bonding therefore cannot survive a reset yet, and that is stated
//!    here rather than discovered later. Everything the stack does with an empty store works; the moment
//!    persistence matters, this module is the thing to replace.
//!
//! Error codes are Silabs' own, from `ecode.h`: `ECODE_OK = 0`, and the NVM3 base is
//! `ECODE_EMDRV_BASE (0xF0000000) | 0xE000`, with `KEY_NOT_FOUND` at `| 0x0B`.

use core::ffi::c_void;

// The four `sl_btctrl_init_*` declarations that used to live here are gone with the Rust transcription of
// `sli_bt_init_controller_features`: the SDK's own `sl_bt_stack_init.c` provides that function now.

/// How many power-manager requirements the stack has asked for, and how many NVM read misses occurred. Both are
/// diagnostics: the first says what a real power manager would have to honour, the second says how much the
/// stack expects from persistent storage.
static mut EM_REQUIREMENTS: u32 = 0;
static mut NVM_MISSES: u32 = 0;

// `sli_bt_init_controller_features` used to be implemented here, in Rust. It is gone on purpose: the SDK's own
// `sl_bt_stack_init.c` (vendored as `vendor/bt_stack/`) defines it, along with the feature-use and BGAPI class
// tables that the host needs, and two definitions of the same symbol is a link error rather than a choice.

/// Record an energy-mode requirement. The device never sleeps, so nothing is acted on.
#[no_mangle]
pub extern "C" fn sli_power_manager_update_em_requirement(_em: u32, _add: bool) {
    unsafe {
        let p = core::ptr::addr_of_mut!(EM_REQUIREMENTS);
        core::ptr::write_volatile(p, core::ptr::read_volatile(p) + 1);
    }
}

/// Record a clock-preservation requirement. Same reasoning.
#[no_mangle]
pub extern "C" fn sli_power_manager_update_hf_clock_settings_preservation_requirement(_add: bool) {
    unsafe {
        let p = core::ptr::addr_of_mut!(EM_REQUIREMENTS);
        core::ptr::write_volatile(p, core::ptr::read_volatile(p) + 1);
    }
}

/// Subscribe to energy-mode transition events.
///
/// The stack uses this to be told when the device is about to sleep. This firmware never sleeps (the timer is
/// polled), so no transition ever happens and the subscription is recorded without a callback to call. Safe by
/// construction: the callback is never invoked, so nothing downstream can depend on a transition that does not
/// occur. `stats()` reports that a subscription was taken, so a future power manager knows it must honour one.
#[no_mangle]
pub extern "C" fn sl_power_manager_subscribe_em_transition_event(
    _handle: *mut c_void,
    _info: *const c_void,
) {
    unsafe {
        let p = core::ptr::addr_of_mut!(EM_SUBSCRIPTIONS);
        core::ptr::write_volatile(p, core::ptr::read_volatile(p) + 1);
    }
}

static mut EM_SUBSCRIPTIONS: u32 = 0;

// The NVM3 surface that used to live here -- `nvm3_open`, `nvm3_readData`, `nvm3_writeData`, `nvm3_deleteObject`
// and the default-instance globals -- is gone. It was a stand-in that could not work: `sli_bt_start_stack` treats a
// failed `nvm3_open` as fatal (`bl nvm3_open` / `bl ecode2bg` / `bne` in its disassembly), and a success that left
// the handle unfilled made the stack read a null pointer 0x138 bytes in. The vendor's real NVM3 is vendored in
// `vendor/nvm3/` and linked from `libnvm3_CM33_gcc.a`.

/// Power-manager requirements recorded, and NVM3 read misses. The example prints both.
pub fn stats() -> (u32, u32) {
    unsafe {
        (
            core::ptr::read_volatile(core::ptr::addr_of!(EM_REQUIREMENTS)),
            core::ptr::read_volatile(core::ptr::addr_of!(NVM_MISSES)),
        )
    }
}

/// How many energy-mode transition subscriptions were taken.
pub fn subscriptions() -> u32 {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(EM_SUBSCRIPTIONS)) }
}
