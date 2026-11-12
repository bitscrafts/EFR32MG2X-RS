//! Compile the CMSIS helpers the Silicon Labs blobs assume the application provides.
//!
//! The two radio blobs (`libbluetooth_*` and `librail_*`) were built against a CMSIS runtime and
//! reference `CORE_*` helpers that the C headers normally provide as `static inline`. Linking C
//! objects against them means those helpers have to exist somewhere as real symbols, and the
//! correct somewhere is one place for both — not a copy per blob, which is how the two definitions
//! drift and one of them ends up masking interrupts differently from the other.

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    println!("cargo:rerun-if-changed=src/cmsis_stubs.c");

    // These helpers use ARM assembly (MRS/MSR/CPSID/WFE), so they only exist for the Cortex-M
    // targets this workspace builds for. A host build of the workspace must not try.
    if !target.starts_with("thumbv") {
        return;
    }

    let mut build = cc::Build::new();
    build
        .file("src/cmsis_stubs.c")
        .flag("-mthumb")
        .flag("-Os")
        .flag("-ffunction-sections")
        .flag("-fdata-sections");

    // No `-mcpu`/`-march` here on purpose: `cc` already derives the CPU from the Rust target, and
    // adding `-mcpu=cortex-m33` next to its `-march=armv8-m.main+fp` makes the compiler warn about
    // a conflict on every single build — noise that hides real warnings.
    build.compile("cmsis_stubs");
}
