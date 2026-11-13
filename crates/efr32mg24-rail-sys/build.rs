use sha2::{Digest, Sha256};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/// Expected SHA256 hash of librail_efr32xg24_gcc_release.a from Gecko SDK v4.4.5
const EXPECTED_BLOB_SHA256: &str =
    "072f0ff0eb708091e49a3fd536c45f4f0568ed6f163c2889f5a8536b234c66a2";

/// Compute SHA256 hash of a file
fn sha256_file(path: &std::path::Path) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn main() {
    // Get the target architecture
    let target = env::var("TARGET").unwrap();

    // Only link blobs for ARM Cortex-M targets
    if target.starts_with("thumbv") {
        // Path to the binary blob directory (use absolute path for cross-compilation)
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let blob_dir = manifest_dir.join("blobs/efr32mg24b220");

        // Check if blob exists (won't exist initially, that's OK)
        let blob_path = blob_dir.join("librail_efr32xg24_gcc_release.a");

        if blob_path.exists() {
            // Verify blob integrity with SHA256
            match sha256_file(&blob_path) {
                Ok(hash) => {
                    if hash != EXPECTED_BLOB_SHA256 {
                        println!("cargo:warning=RAIL blob SHA256 mismatch!");
                        println!("cargo:warning=  Expected: {}", EXPECTED_BLOB_SHA256);
                        println!("cargo:warning=  Got:      {}", hash);
                        println!("cargo:warning=  The blob may be corrupted or from a different Gecko SDK version.");
                        println!("cargo:warning=  Expected version: Gecko SDK v4.4.5");
                    }
                }
                Err(e) => {
                    println!("cargo:warning=Could not verify RAIL blob SHA256: {}", e);
                }
            }

            // Link the RAIL binary blob
            println!("cargo:rustc-link-search=native={}", blob_dir.display());
            // whole-archive: the interrupt handlers live in blob objects that nothing references by name
            // once device.x provides weak aliases, and an archive member is only pulled in for an
            // undefined symbol. Without this the radio interrupt lands in DefaultHandler and the
            // transmitter stalls inside the handler, measured as pc in DefaultHandler_.
            println!("cargo:rustc-link-lib=static:+whole-archive=rail_efr32xg24_gcc_release");
            println!("cargo:blob-dir={}", blob_dir.display());
            println!(
                "cargo:warning=Linking RAIL blob from {}",
                blob_path.display()
            );
        } else {
            println!(
                "cargo:warning=RAIL blob not found at {}. This is expected during development.",
                blob_path.display()
            );
            println!("cargo:warning=To use RAIL, extract librail_efr32xg24_gcc_release.a from Gecko SDK to {}", blob_path.display());
        }
    }

    // Path to RAIL headers (won't exist initially)
    let header_path = PathBuf::from("vendor/rail/rail.h");

    if header_path.exists() {
        // Clang arguments for bindgen
        // Platform-independent: use our minimal headers instead of system libc
        let clang_args = vec![
            // Add our minimal include directory (provides string.h, etc.)
            "-Iinclude",
            // Add vendor/rail directory for RAIL headers
            "-Ivendor/rail",
            // Add chip-specific headers directory
            "-Ivendor/rail/chip/efr32xg2x",
            // Freestanding environment (no system libc)
            "-ffreestanding",
            // Suppress warnings about system headers
            "-Wno-everything",
        ];

        // Generate Rust bindings from C headers using bindgen
        let bindings = bindgen::Builder::default()
            .header(header_path.to_str().unwrap())
            // rail.h does not pull in the protocol headers, so the BLE configuration API
            // (`RAIL_BLE_ConfigPhy1Mbps`, `RAIL_BLE_ConfigChannelRadioParams`) never appeared in the
            // bindings — the functions existed in the header and in the blob, and were unreachable
            // from Rust. Adding the protocol header is what makes them callable.
            .header("vendor/rail/protocol/ble/rail_ble.h")
            .clang_arg("-Ivendor/rail/protocol/ble")
            // Add clang arguments
            .clang_args(&clang_args)
            // Use core instead of std (for no_std compatibility)
            .use_core()
            // Use cty for C types
            .ctypes_prefix("cty")
            // Don't generate layout tests (they fail on cross-compilation)
            .layout_tests(false)
            // Allowlist RAIL functions and types
            .allowlist_function("RAIL_.*")
            .allowlist_type("RAIL_.*")
            .allowlist_var("RAIL_.*")
            // Generate bindings
            .generate()
            .expect("Unable to generate RAIL bindings");

        // Write bindings to OUT_DIR
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");

        println!(
            "cargo:warning=Generated RAIL bindings from {}",
            header_path.display()
        );
    } else {
        println!(
            "cargo:warning=RAIL headers not found at {}. Bindings will not be generated.",
            header_path.display()
        );
        println!(
            "cargo:warning=To generate bindings, extract rail.h from Gecko SDK to {}",
            header_path.display()
        );

        // Create empty bindings file so the crate still compiles
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        std::fs::write(
            out_path.join("bindings.rs"),
            "// RAIL bindings not generated (headers not found)\n// This is a placeholder to allow compilation\n"
        ).expect("Couldn't write placeholder bindings");
    }

    // The PA conversion plugin (Zlib, Gecko SDK v4.4.5, the RAIL 2.x variant of it).
    //
    // Measured before this existed: `RAIL_SetTxPowerDbm(handle, 100)` returned 0 and
    // `RAIL_GetTxPowerDbm` read back -32768 -- RAIL's "invalid power" -- while every transmission still
    // completed (`RAIL_EVENT_TX_PACKET_SENT` fired). The dBm-to-raw mapping is a WEAK function the customer
    // is documented to override; rail.h says so at RAIL_ConvertDbmToRaw. Without the override the PA gets
    // no valid level, drives nothing, and the modem reports success for a packet nobody can hear.
    //
    // This also supplies `sl_rail_util_pa_get_tx_power_config_2p4ghz`, one of the five symbols that kept
    // the vendor Bluetooth stack from linking.
    if target.starts_with("thumbv") {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let emlib_vendor = manifest_dir.join("../efr32mg24-emlib-sys/vendor");
        cc::Build::new()
            .file("vendor/pa2/pa_conversions_efr32.c")
            .file("vendor/pa2/pa_curves_efr32.c")
            .include("vendor/pa2")
            .include("vendor/pa2/efr32xg24")
            .include("vendor/pa/efr32xg24/config")
            .include("vendor/pa")
            .include("vendor/rail")
            .include("vendor/rail/protocol/ble")
            .include("vendor/rail/chip/efr32xg2x")
            // The device header refuses to compile without the exact part number; emlib-sys defines the
            // same two for its own units.
            .define("EFR32MG24B220F1536IM48", None)
            .define("EFR32MG24", None)
            .include(emlib_vendor.join("emlib/inc"))
            .include(emlib_vendor.join("device"))
            .include(emlib_vendor.join("common/inc"))
            .include(emlib_vendor.join("common"))
            .include(emlib_vendor.join("cmsis"))
            // Same reasons as emlib: vendor C written for gnu11, and the device unit carries SAU setup that
            // needs the CMSE extension even though we never run it.
            .flag_if_supported("-mcmse")
            .flag_if_supported("-ffunction-sections")
            .flag_if_supported("-fdata-sections")
            .flag_if_supported("-Wno-everything")
            .compile("rail_pa");
        // Exposed so a dependent can place this archive AFTER its own blobs: the PA curves are referenced
        // by the Bluetooth blobs, and an archive only satisfies symbols that are already undefined.
        println!("cargo:lib-dir={}", std::env::var("OUT_DIR").unwrap());
        println!("cargo:rerun-if-changed=vendor/pa2/");
    }

    // Tell Cargo to re-run this build script if these files change
    println!("cargo:rerun-if-changed=blobs/");
    println!("cargo:rerun-if-changed=vendor/");
}
