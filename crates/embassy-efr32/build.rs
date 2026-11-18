//! Puts `memory.x` where cortex-m-rt's `link.x` looks for it.
//!
//! `link.x` does `INCLUDE memory.x`, and the linker only finds it if this crate adds its own
//! directory to the search path. Without this, every example in this crate fails with
//! "cannot find linker script memory.x" -- a link error that says nothing about the code.
use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    fs::copy("memory.x", out_dir.join("memory.x")).expect("memory.x missing from the crate root");
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed=memory.x");
}
