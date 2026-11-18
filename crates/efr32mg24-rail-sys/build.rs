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

// ---------------------------------------------------------------------------------------------------------------
// Vendored inputs, fetched at build time.
//
// The published crate does not carry the RAIL archive or its headers: they are Silicon Labs' (MSLA), and a
// crates.io package that ships them would be redistributing vendor binaries, on top of the ten-megabyte limit this
// crate would blow. So `cargo install` / `cargo build` pulls them from the project's own repository at a pinned
// commit, verifies the archive's SHA-256, and extracts only this crate's subtrees.
//
// The alternative -- failing and telling the user to unpack the Gecko SDK by hand -- is what the crate did before,
// and it is the reason a clean clone could not build. A build that needs a manual step nobody documents is a build
// that only works for its author.
const VENDOR_COMMIT: &str = "eefde4498a9dc5dd8d55f6023965d566c863f2f5";
const VENDOR_URL: &str = "https://codeload.github.com/bitscrafts/EFR32MG2X-RS/tar.gz/";
const VENDOR_TARBALL_SHA256: &str =
    "ee1a61ee2262a9c91655e2fee35f11e01aef00542cd4cc0705641f1912b05905";

fn ensure_vendor(manifest_dir: &std::path::Path) {
    let have = manifest_dir.join("vendor/rail/rail.h").exists()
        && manifest_dir
            .join("blobs/efr32mg24b220/librail_efr32xg24_gcc_release.a")
            .exists();
    if have {
        return;
    }
    if VENDOR_COMMIT.starts_with("PINNED") {
        println!("cargo:warning=RAIL vendor files are missing and no commit is pinned; building without them");
        return;
    }

    // One download serves every crate on the machine: the cache is keyed by commit, not by crate.
    let cache = env::var("CARGO_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::temp_dir())
        .join("efr32mg2x-vendor")
        .join(VENDOR_COMMIT);
    let tarball = cache.join("repo.tar.gz");
    std::fs::create_dir_all(&cache).expect("cannot create the vendor cache");

    if !tarball.exists() {
        println!(
            "cargo:warning=fetching the RAIL archive and headers from {}{}",
            VENDOR_URL, VENDOR_COMMIT
        );
        let status = std::process::Command::new("curl")
            .args(["-sSL", "-o"])
            .arg(&tarball)
            .arg(format!("{}{}", VENDOR_URL, VENDOR_COMMIT))
            .status()
            .unwrap_or_else(|e| panic!("curl is required to fetch the vendored RAIL files: {e}"));
        assert!(status.success(), "downloading the vendor archive failed");
    }

    let got = sha256_file(&tarball).expect("cannot read the downloaded archive");
    assert_eq!(
        got,
        VENDOR_TARBALL_SHA256,
        "vendor archive checksum mismatch:\n  expected {VENDOR_TARBALL_SHA256}\n  got      {got}\n\
         Delete {tarball:?} and rebuild; do not use an archive that does not match.",
        tarball = tarball
    );

    // Extract only this crate's trees: the archive holds the whole repository.
    let prefix = "EFR32MG2X-RS-"; // the tar root is <repo>-<sha>/
    let status = std::process::Command::new("tar")
        .current_dir(&cache)
        .args(["-xzf"])
        .arg(&tarball)
        .arg("--strip-components=1")
        .arg("-C")
        .arg(&cache)
        .args([
            format!("{prefix}{VENDOR_COMMIT}/crates/efr32mg24-rail-sys/vendor"),
            format!("{prefix}{VENDOR_COMMIT}/crates/efr32mg24-rail-sys/blobs"),
        ])
        .status()
        .unwrap_or_else(|e| panic!("tar is required to unpack the vendor archive: {e}"));
    assert!(status.success(), "extracting the vendor archive failed");

    let src = cache.join("crates/efr32mg24-rail-sys");
    for sub in ["vendor", "blobs"] {
        copy_tree(&src.join(sub), &manifest_dir.join(sub));
    }
    println!("cargo:warning=vendored RAIL files installed from commit {VENDOR_COMMIT}");
}

/// Recursive copy: `fs::copy` needs an existing parent, and the crate has no subtree until this runs.
fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).expect("cannot create the vendor directory");
    for entry in std::fs::read_dir(from).expect("cannot read the vendor source directory") {
        let entry = entry.expect("cannot read a vendor entry");
        let target = to.join(entry.file_name());
        if entry
            .file_type()
            .expect("cannot stat a vendor entry")
            .is_dir()
        {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("cannot copy a vendor file");
        }
    }
}

fn main() {
    // Get the target architecture
    let target = env::var("TARGET").unwrap();

    // Only link blobs for ARM Cortex-M targets
    if target.starts_with("thumbv") {
        // Path to the binary blob directory (use absolute path for cross-compilation)
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        // Nothing below this line can work without the vendor files, so make sure they are here first.
        ensure_vendor(&manifest_dir);
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
            // Same reason, second protocol: rail.h does not pull in the 802.15.4 header either, so
            // RAIL_IEEE802154_Init and RAIL_IEEE802154_Config2p4GHzRadio existed in the blob and in
            // the header and were unreachable from Rust.
            .header("vendor/rail/protocol/ieee802154/rail_ieee802154.h")
            .clang_arg("-Ivendor/rail/protocol/ieee802154")
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
            // **The Radio Configurator's BLE 39 MHz table is vendored but NOT compiled here.** Including it as a
            // separate C unit duplicates symbols that the RAIL blob already contains, because Silabs' `rail_types.h`
            // includes chip-specific `.c` files: the link fails with `duplicate symbol: sli_bootloader_preHook`
            // (rfhal_pti.c) and a pile of `RAILINT_*` names. The data is in `vendor/ble_config/` and the intended
            // route is to emit it as Rust statics -- see docs/04-reference/MODEM-CONFIG.md. It is what fixes
            // `RAIL_ASSERT_INVALID_XTAL_FREQUENCY` (66), which is what stops the Bluetooth stack from starting.
            // **PA auto mode: an application-side file the controller calls into.** `ll_radio_pa.c` in the Bluetooth
            // controller references `RAILCb_PaAutoModeDecision`, and only this plugin defines it. Without it the
            // radio bring-up cannot choose a power amplifier and the stack reports `SL_STATUS_BT_RADIO`.
            .file("vendor/pa_auto_mode/pa_auto_mode.c")
            // The RAIL library already defines `RAIL_ConfigPaAutoEntry`; the plugin's own guard marks its copies weak
            // so the library's strong definitions win and only the missing callback is supplied from here.
            .define("RAIL_PA_AUTO_MODE_WEAK", None)
            // ...and one function in that plugin has no weak guard: `RAIL_ConfigPaAutoEntry` is defined both there and
            // in the RAIL library, which is a duplicate symbol. Renaming it inside this translation unit keeps the
            // library's definition authoritative and leaves the plugin's copy unused, without editing vendor code.
            .define(
                "RAIL_ConfigPaAutoEntry",
                "rail_pa_auto_entry_already_in_library",
            )
            .file("vendor/pa2/pa_conversions_efr32.c")
            .file("vendor/pa2/pa_curves_efr32.c")
            .include("vendor/pa2")
            .include("vendor/pa2/efr32xg24")
            .include("vendor/pa/efr32xg24/config")
            .include("vendor/pa")
            .include("vendor/rail")
            .include("vendor/rail/protocol/ble")
            .include("vendor/rail/chip/efr32xg2x")
            .include("vendor/ble_config")
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
