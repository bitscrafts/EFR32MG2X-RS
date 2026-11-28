//! Puts the linker scripts this crate owns where `link.x` looks for them.
//!
//! `memory.x` is deliberately **not** one of them: the layout is provided once, by `efr32mg2x-pac`, so that a program
//! cannot end up with two candidate layouts and whichever the linker happens to find first. This crate used to ship a
//! copy of the 1472K one; that copy and the PAC's 1512K one both sat on the search path, which is the ambiguity this
//! removal ends. See the note in `efr32mg2x-hal/build.rs`.
use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    println!("cargo:rustc-link-search={}", out_dir.display());
}
