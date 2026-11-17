//! Linker fragment for RAM-resident code.
//!
//! The flash controller cannot write or erase while code executes from flash, so the emlib marks the programming
//! routines `__RAMFUNC`, which places them in `.ram`. Without a script of our own that section is linked at its flash
//! address (measured 2026-11-17: `.ram VMA=0x0802e4ac LMA=0x0802e4ac` in a built image), so `MSC_WriteWord` ran from
//! the flash it was programming: NVM3 reported a successful write, nothing was stored, and every Bluetooth bond was
//! silently lost.
//!
//! `memory.x` is the obvious home for this and does not work: cortex-m-rt's `link.x` includes `memory.x` *before* its
//! own `SECTIONS` block, and a `SECTIONS` of ours there is discarded (`undefined symbol: __ramfunc_start`). A second
//! script passed after `link.x` is merged instead.

use std::path::PathBuf;

fn main() {
    let fragment: PathBuf =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("ramfunc.x");
    let dir = fragment
        .parent()
        .expect("ramfunc.x has a parent directory")
        .to_path_buf();
    println!("cargo:rustc-link-search={}", dir.display());
    println!("cargo:rustc-link-arg=-Tramfunc.x");
    println!("cargo:rerun-if-changed=ramfunc.x");
}
