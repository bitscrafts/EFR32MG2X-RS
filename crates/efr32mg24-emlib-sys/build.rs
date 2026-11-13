//! Build the emlib units the radio blobs link against.
//!
//! **The version matters, and this file now says which one.** The blobs are from Gecko SDK v4.4.5 and call
//! `CMU_ClockEnable`, `CMU_HFXOCTuneSet`, `EMU_DCDCSetPFMXTimeoutMaxCtrl` and `GPIO_PinModeSet`; the newer
//! simplicity_sdk generation this crate first used has `CMU_HFXOCTuneDeltaSet` and
//! `EMU_DCDCSetPFMXModePeakCurrent` instead, and no em_gpio at all. Measured by linking the stack: nine
//! undefined symbols that were pure version skew.
//!
//! So `vendor445/` holds the v4.4.5 sources and headers, next to the original `vendor/` tree, which stays
//! untouched. Two SDK generations side by side is not untidiness — it is the record of which one the blobs
//! expect.
//!
//! Keep the source list short on purpose: every file here is vendor C that must compile for a Cortex-M33 with
//! the full device headers, and adding a file because it is "probably useful" turns this into an SDK nobody
//! can audit.

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();

    let sources = [
        "vendor445/src/em_cmu.c",
        "vendor445/src/em_emu.c",
        "vendor445/src/em_system.c",
        // GPIO_PinModeSet: the stack configures its GPIOs itself, and the v4.4.5 emlib owns that function.
        "vendor445/src/em_gpio.c",
        // SystemSYSCLKGet and friends: em_cmu.c calls into the device unit.
        "vendor445/device/system_efr32mg24.c",
        "src/shim.c",
    ];
    for f in sources {
        println!("cargo:rerun-if-changed={f}");
    }
    for d in [
        "vendor445/inc",
        "vendor445/device",
        "vendor445/cmsis",
        "vendor/common",
    ] {
        println!("cargo:rerun-if-changed={d}");
    }

    // The device headers contain ARM intrinsics and register maps; a host build must not try.
    if !target.starts_with("thumbv") {
        return;
    }

    let mut build = cc::Build::new();
    for f in sources {
        build.file(f);
    }
    build
        // The part define is what makes em_device.h select the b220 header. Without it the device header
        // errors out — which is the good failure: it cannot silently target another chip.
        .define("EFR32MG24B220F1536IM48", None)
        .define("EFR32MG24", None)
        // The v4.4.5 CMSIS core headers use `__RESTRICT`, which their own cmsis_gcc.h normally defines. The
        // device header chain in this package does not pull that file in, so it is defined here rather than
        // editing vendor headers.
        .define("__RESTRICT", "restrict")
        .include("vendor445/inc")
        .include("vendor445/device")
        .include("vendor445/cmsis")
        .include("vendor/common")
        .flag("-mthumb")
        .flag("-Os")
        .flag("-ffunction-sections")
        .flag("-fdata-sections")
        // The device's system unit carries the SAU setup; we do not use that startup (cortex-m-rt owns reset),
        // so those functions are dead and removed by --gc-sections, but the file will not preprocess without
        // the CMSE extension.
        .flag_if_supported("-mcmse")
        .flag_if_supported("-Wno-everything");

    build.compile("emlib_min");
    // Where the archive landed, so a crate that needs the vendor library AFTER its own blob can say so. The
    // link order cannot be expressed by dependency order alone: see crates/efr32mg24-bluetooth-sys/build.rs.
    println!("cargo:lib-dir={}", std::env::var("OUT_DIR").unwrap());
}
