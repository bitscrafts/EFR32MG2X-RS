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
            println!("cargo:rustc-link-lib=static=rail_efr32xg24_gcc_release");
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

    // Tell Cargo to re-run this build script if these files change
    println!("cargo:rerun-if-changed=blobs/");
    println!("cargo:rerun-if-changed=vendor/");
}
