use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Copy bootloader linker script
    fs::copy("bootloader.x", out_dir.join("bootloader.x"))
        .expect("Failed to copy bootloader.x");

    // Tell cargo to rerun if bootloader.x changes
    println!("cargo:rerun-if-changed=bootloader.x");

    // Tell linker to search in OUT_DIR
    println!("cargo:rustc-link-search={}", out_dir.display());
}
