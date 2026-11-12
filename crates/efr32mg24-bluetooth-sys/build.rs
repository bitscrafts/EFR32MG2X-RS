use sha2::{Digest, Sha256};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/// Expected SHA256 hashes for Bluetooth blobs from Gecko SDK v4.4.5
/// Note: Gecko SDK v4.4.5+ uses split Bluetooth stack architecture
const EXPECTED_CONTROLLER_SHA256: &str =
    "4440d8ed63081df122702f5a00ef141134727d9251947de093deb7df8e179167";
const EXPECTED_HOST_SHA256: &str =
    "27dd331af5cae646aad8e772ee7d0934f5bfc712f8fb9eb374630a946c912dc7";
const EXPECTED_COMMON_SHA256: &str =
    "01f45ff9639a24b02c57d3ceaedf592f31f29873ca788f22b24f5b2c930b682e";

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

        // Gecko SDK v4.4.5+ uses split Bluetooth stack:
        //   1. libbluetooth_controller (link layer)
        //   2. libbluetooth_host (host stack)
        //   3. libbgcommon (common components)

        let controller_path = blob_dir.join("libbluetooth_controller_efr32xg24_gcc_release.a");
        let host_path = blob_dir.join("libbluetooth_host_efr32xg24_gcc_release.a");
        let common_path = blob_dir.join("libbgcommon_efr32xg24_gcc_release.a");

        let all_blobs_exist =
            controller_path.exists() && host_path.exists() && common_path.exists();

        if all_blobs_exist {
            // Verify blob integrity with SHA256
            match sha256_file(&controller_path) {
                Ok(hash) => {
                    if hash != EXPECTED_CONTROLLER_SHA256 {
                        println!("cargo:warning=Bluetooth controller blob SHA256 mismatch!");
                        println!("cargo:warning=  Expected: {}", EXPECTED_CONTROLLER_SHA256);
                        println!("cargo:warning=  Got:      {}", hash);
                    }
                }
                Err(e) => {
                    println!(
                        "cargo:warning=Could not verify controller blob SHA256: {}",
                        e
                    );
                }
            }

            match sha256_file(&host_path) {
                Ok(hash) => {
                    if hash != EXPECTED_HOST_SHA256 {
                        println!("cargo:warning=Bluetooth host blob SHA256 mismatch!");
                        println!("cargo:warning=  Expected: {}", EXPECTED_HOST_SHA256);
                        println!("cargo:warning=  Got:      {}", hash);
                    }
                }
                Err(e) => {
                    println!("cargo:warning=Could not verify host blob SHA256: {}", e);
                }
            }

            match sha256_file(&common_path) {
                Ok(hash) => {
                    if hash != EXPECTED_COMMON_SHA256 {
                        println!("cargo:warning=Bluetooth common blob SHA256 mismatch!");
                        println!("cargo:warning=  Expected: {}", EXPECTED_COMMON_SHA256);
                        println!("cargo:warning=  Got:      {}", hash);
                    }
                }
                Err(e) => {
                    println!("cargo:warning=Could not verify common blob SHA256: {}", e);
                }
            }

            // Link all three Bluetooth binary blobs
            println!("cargo:rustc-link-search=native={}", blob_dir.display());
            println!("cargo:rustc-link-lib=static=bluetooth_controller_efr32xg24_gcc_release");
            println!("cargo:rustc-link-lib=static=bluetooth_host_efr32xg24_gcc_release");
            println!("cargo:rustc-link-lib=static=bgcommon_efr32xg24_gcc_release");
        } else {
            println!(
                "cargo:warning=Bluetooth blobs not found. This is expected during development."
            );
            println!("cargo:warning=Required blobs:");
            println!("cargo:warning=  - {}", controller_path.display());
            println!("cargo:warning=  - {}", host_path.display());
            println!("cargo:warning=  - {}", common_path.display());
            println!("cargo:warning=To extract blobs, run: ./scripts/setup-bluetooth.sh");
        }
    }

    // Path to Bluetooth BGAPI headers (won't exist initially)
    let header_path = PathBuf::from("vendor/bluetooth/sl_bt_api.h");

    if header_path.exists() {
        // Clang arguments for bindgen
        // Platform-independent: use our minimal headers instead of system libc
        let clang_args = vec![
            // Add our minimal include directory (provides string.h, etc.)
            "-Iinclude",
            // Add vendor/bluetooth directory for BGAPI headers
            "-Ivendor/bluetooth",
            // Freestanding environment (no system libc)
            "-ffreestanding",
            // Suppress warnings about system headers
            "-Wno-everything",
        ];

        // Generate Rust bindings from C headers using bindgen
        let bindings = bindgen::Builder::default()
            .header(header_path.to_str().unwrap())
            // Add clang arguments
            .clang_args(&clang_args)
            // Use core instead of std (for no_std compatibility)
            .use_core()
            // Use cty for C types
            .ctypes_prefix("cty")
            // NOTE: these headers declare an *anonymous* union inside `sl_bt_msg_t`, and bindgen
            // emulates anonymous unions with `__BindgenUnionField<T>` no matter which rust target
            // is requested -- measured: adding `.rust_target(Stable_1_71)` left the output
            // byte-identical. The emulation is not a bug to work around; it is what makes the
            // access explicit, and `src/blocking/events.rs` reads every payload through
            // `as_ref()`, which is `unsafe` and says so.
            // Don't generate layout tests (they fail on cross-compilation)
            .layout_tests(false)
            // Allowlist Bluetooth BGAPI functions and types
            .allowlist_function("sl_bt_.*")
            .allowlist_type("sl_bt_.*")
            .allowlist_var("sl_bt_.*")
            // Also allow SL_STATUS types
            .allowlist_type("SL_STATUS")
            .allowlist_var("SL_STATUS_.*")
            // Generate bindings
            .generate()
            .expect("Unable to generate Bluetooth bindings");

        // Write bindings to OUT_DIR
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    } else {
        println!(
            "cargo:warning=Bluetooth headers not found at {}. Bindings will not be generated.",
            header_path.display()
        );
        println!(
            "cargo:warning=To generate bindings, extract sl_bt_api.h from Gecko SDK to {}",
            header_path.display()
        );

        // Create empty bindings file so the crate still compiles
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        std::fs::write(
            out_path.join("bindings.rs"),
            "// Bluetooth bindings not generated (headers not found)\n// This is a placeholder to allow compilation\n"
        ).expect("Couldn't write placeholder bindings");
    }

    // Tell Cargo to re-run this build script if these files change
    // The stubs are part of this crate's compiled output, and cargo does not watch them by
    // default: without this line an edit to cmsis_stubs.c is silently not compiled, the link
    // fails on symbols that are visibly present in the source, and the build looks stale.
    println!("cargo:rerun-if-changed=blobs/");
    println!("cargo:rerun-if-changed=vendor/");
}
