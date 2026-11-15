use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Put `memory.x` and `device.x` in our output directory and ensure it's on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // Copy memory.x to output directory
    let memory_x = include_bytes!("memory.x");
    let mut f = File::create(out.join("memory.x")).unwrap();
    f.write_all(memory_x).unwrap();

    // Copy device.x to output directory
    let device_x = include_bytes!("device.x");
    let mut f = File::create(out.join("device.x")).unwrap();
    f.write_all(device_x).unwrap();

    // Add the output directory to the linker search path
    println!("cargo:rustc-link-search={}", out.display());

    // Rebuild if memory.x or device.x changes
    // The .defmt section the probe reads the format strings from, for this crate's examples only: in
    // .cargo/config.toml it would break every other crate's examples, which have no defmt to supply the script.
    println!("cargo:rustc-link-arg-examples=-Tdefmt.x");
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=device.x");
}
