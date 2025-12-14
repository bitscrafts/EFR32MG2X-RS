use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Copy memory.x to OUT_DIR so the linker can find it via INCLUDE directive
    fs::copy("memory.x", out_dir.join("memory.x")).expect("Failed to copy memory.x");

    // Copy device.x to OUT_DIR so the linker can find it via INCLUDE directive
    fs::copy("device.x", out_dir.join("device.x")).expect("Failed to copy device.x");

    // Add OUT_DIR to the linker search path
    // This allows link.x to find memory.x and device.x via INCLUDE directives
    // DO NOT add -Tmemory.x flag here - cortex-m-rt handles that
    println!("cargo:rustc-link-search={}", out_dir.display());

    // Re-run if linker scripts change
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=device.x");
}
