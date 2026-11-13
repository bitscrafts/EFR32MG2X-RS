//! Build the three emlib units the radio blobs link against.
//!
//! Keep this list short on purpose: every file added here is vendor C that has to compile for a
//! Cortex-M33 with the full device headers. Three is what the radio needs; adding a fourth because it
//! is "probably useful" turns this crate into an SDK nobody can audit.

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    for f in [
        "vendor/emlib/src/em_cmu.c",
        "vendor/emlib/src/em_emu.c",
        "vendor/emlib/src/em_system.c",
        "vendor/device_src/system_efr32mg24.c",
        "src/shim.c",
    ] {
        println!("cargo:rerun-if-changed={f}");
    }
    for d in [
        "vendor/emlib/inc",
        "vendor/device",
        "vendor/cmsis",
        "vendor/common",
    ] {
        println!("cargo:rerun-if-changed={d}");
    }

    // The device headers contain ARM intrinsics and register maps; a host build must not try.
    if !target.starts_with("thumbv") {
        return;
    }

    let mut build = cc::Build::new();
    build
        .file("vendor/emlib/src/em_cmu.c")
        .file("vendor/emlib/src/em_emu.c")
        .file("vendor/emlib/src/em_system.c")
        // SystemSYSCLKGet lives here, not in emlib: em_cmu.c calls it and the device unit owns it.
        .file("vendor/device_src/system_efr32mg24.c")
        // Our wrappers: the only code here that is this project's, and it exists to keep emlib enums
        // out of the Rust side.
        .file("src/shim.c")
        // The part define is what makes em_device.h select efr32mg24b220f1536im48.h. Without it the
        // device header errors out — which is the good failure: it cannot silently target another chip.
        .define("EFR32MG24B220F1536IM48", None)
        .define("EFR32MG24", None)
        .include("vendor/emlib/inc")
        .include("vendor/device")
        .include("vendor/cmsis")
        .include("vendor/common")
        .flag("-mthumb")
        .flag("-Os")
        .flag("-ffunction-sections")
        .flag("-fdata-sections")
        // emlib is vendor C from a project that compiles with gnu11; the warnings are not ours to fix
        // and a wall of them hides the one that matters.
        // The device's system unit also carries the SAU setup used by the vendor's own startup code.
        // We do not use that startup (cortex-m-rt owns reset), so those functions are dead and get
        // removed by --gc-sections — but the file will not preprocess without the CMSE extension.
        .flag_if_supported("-mcmse")
        .flag_if_supported("-Wno-everything");

    build.compile("emlib_min");
    // Where the archive landed, so a crate that needs the vendor library AFTER its own blob can say so.
    // The link order matters and cannot be expressed by dependency order alone: see
    // crates/efr32mg24-bluetooth-sys/build.rs.
    println!("cargo:lib-dir={}", std::env::var("OUT_DIR").unwrap());
}
