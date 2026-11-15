use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // The defmt section the probe reads the format strings from. -Tdefmt.x belongs to the targets that log, and putting it
    // in .cargo/config.toml broke every other crate's examples: they have no defmt build script to provide the script
    // file. Emitted here it reaches this crate's examples and nothing else.
    println!("cargo:rustc-link-arg-examples=-Tdefmt.x");

    // Copy memory.x to OUT_DIR so cortex-m-rt can find it
    fs::copy("memory.x", out_dir.join("memory.x")).expect("Failed to copy memory.x");

    // Copy device.x to OUT_DIR so the linker can find it via INCLUDE directive
    fs::copy("device.x", out_dir.join("device.x")).expect("Failed to copy device.x");

    // Add OUT_DIR to the linker search path
    println!("cargo:rustc-link-search={}", out_dir.display());

    // Re-run if linker scripts change
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=device.x");
}
