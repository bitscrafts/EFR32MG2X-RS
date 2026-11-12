//! Silicon Labs emlib, vendored to the minimum the radio needs.
//!
//! ## Why a crate and not three stubs
//!
//! The RAIL and Bluetooth blobs reference a handful of emlib functions — `CMU_ClockEnable`,
//! `SystemSYSCLKGet`, `SYSTEM_ChipRevisionGet`, `SYSTEM_GetUnique`, `EMU_DCDCSetPFMXTimeoutMaxCtrl`.
//! The first version of this project answered that with hand-written stubs, which is guessing at
//! register writes and calling it integration.
//!
//! The honest answer is the vendor's own source. `em_cmu.c` is 363 KB and needs the full device
//! headers, which is why it looked impossible earlier — but the headers and the CMSIS core are freely
//! available, and three source files plus their includes is a bounded amount of vendor code, not a
//! SDK.
//!
//! ## What is vendored
//!
//! ```text
//! vendor/emlib/src    em_cmu.c, em_emu.c, em_system.c        the three units that were missing
//! vendor/emlib/inc    all 45 emlib headers                   because the three include each other
//! vendor/device       all 87 EFR32MG24 device headers        because em_cmu.c includes the part
//! vendor/cmsis        the CMSIS Core headers                 because the device headers include them
//! ```
//!
//! Source: `github.com/SiliconLabs/simplicity_sdk`, Zlib licence (see `vendor/LICENCE.txt`).
//! The blobs stay proprietary; this is the open part of the same SDK, which is the part that was
//! missing.

#![no_std]

/// The part this build targets. Kept here so a wrong-device build fails in one visible place.
pub const PART: &str = "EFR32MG24B220F1536IM48";

// emlib, reachable from Rust.
//
// The declarations live here rather than in each example so the archive is linked exactly once and
// every consumer gets the same symbols. An empty `extern` block carrying only `#[link]` is dropped by
// the compiler — which is how the first version of this crate produced "undefined symbol" for
// functions that were sitting in the archive the whole time.
#[link(name = "emlib_min", kind = "static")]
extern "C" {
    /// `SystemSYSCLKGet()`: the clock the core is actually running from, in Hz.
    pub fn emlib_sysclk_hz() -> u32;
    /// `CMU_ClockFreqGet(cmuClock_CORE)`: the core branch frequency, in Hz.
    pub fn emlib_core_hz() -> u32;
    /// `SYSTEM_GetUnique()`: the 64-bit device identifier programmed at manufacture.
    pub fn emlib_unique_id() -> u64;
    /// Chip revision, packed as major << 24 | minor << 16 | part number or family.
    pub fn emlib_chip_revision() -> u32;
}
