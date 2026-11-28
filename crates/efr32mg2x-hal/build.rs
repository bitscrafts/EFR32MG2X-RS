use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // The defmt section the probe reads the format strings from. -Tdefmt.x belongs to the targets that log, and putting it
    // in .cargo/config.toml broke every other crate's examples: they have no defmt build script to provide the script
    // file. Emitted here it reaches this crate's examples and nothing else.
    println!("cargo:rustc-link-arg-examples=-Tdefmt.x");

    // `memory.x` is **not** emitted here, and the omission is the point: until 2026-11-27 this crate copied its own
    // 1472K layout into the link search path while `efr32mg2x-pac` copied a 1512K one, and the linker took the first
    // match -- so the reservation for NVM3 was silently ignored in every image, and a product whose code grew past
    // 0x08176000 would have written its bonds on top of itself. One crate in the graph provides the layout: the PAC.
    // A crate that wants a different one passes its own `-Tmemory.x`, which is a visible decision rather than a race.
    // Copy device.x to OUT_DIR so the linker can find it via INCLUDE directive
    fs::copy("device.x", out_dir.join("device.x")).expect("Failed to copy device.x");

    // Add OUT_DIR to the linker search path
    println!("cargo:rustc-link-search={}", out_dir.display());

    // Re-run if the linker script changes
    println!("cargo:rerun-if-changed=device.x");
}
