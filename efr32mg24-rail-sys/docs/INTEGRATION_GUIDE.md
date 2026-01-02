# RAIL Integration Guide - Complete Documentation

## Overview

This document provides comprehensive documentation of every step in integrating Silicon Labs RAIL (Radio Abstraction Interface Layer) into the EFR32MG24 Rust HAL project.

**Purpose**: Low-level FFI bindings to RAIL proprietary binary library
**Target**: Production-quality -sys crate for safe HAL wrapper
**Status**: Complete and verified

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Project Structure](#project-structure)
3. [Build System Architecture](#build-system-architecture)
4. [Platform-Independent Headers](#platform-independent-headers)
5. [Bindgen Integration](#bindgen-integration)
6. [Blob Linking](#blob-linking)
7. [SHA256 Verification](#sha256-verification)
8. [Safety Documentation](#safety-documentation)
9. [CI/CD Integration](#cicd-integration)
10. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required Tools

1. **Rust Toolchain** (stable)
   ```bash
   rustup target add thumbv8m.main-none-eabihf
   ```

2. **Clang/LLVM** (for bindgen)
   ```bash
   # macOS (MacPorts)
   sudo port install llvm-10

   # macOS (Homebrew)
   brew install llvm

   # Linux
   sudo apt-get install libclang-dev
   ```

3. **Git LFS** (for Gecko SDK)
   ```bash
   # macOS (MacPorts)
   sudo port install git-lfs

   # macOS (Homebrew)
   brew install git-lfs

   # Linux
   sudo apt-get install git-lfs

   # Initialize
   git lfs install
   ```

### Required Files

1. **RAIL Binary Blob**: `librail_efr32xg24_gcc_release.a` (1.5 MB)
   - Source: Gecko SDK v4.4.5
   - Path: `platform/radio/rail_lib/autogen/librail_release/`

2. **RAIL Headers**: `rail.h` and dependencies (11 files)
   - Source: Gecko SDK v4.4.5
   - Path: `platform/radio/rail_lib/`

---

## Project Structure

```
efr32mg24-rail-sys/
├── Cargo.toml                    # Package metadata and dependencies
├── build.rs                      # Build script (bindgen + linking)
├── src/
│   └── lib.rs                    # FFI bindings entry point
├── blobs/
│   └── efr32mg24b220/
│       └── librail_efr32xg24_gcc_release.a  # RAIL binary (1.5 MB)
├── vendor/
│   └── rail/
│       ├── rail.h                # Main RAIL API header
│       ├── rail_types.h          # Type definitions
│       ├── rail_features.h       # Feature detection
│       └── chip/
│           └── efr32xg2x/
│               └── rail_chip_specific.h  # Chip-specific defs
├── include/
│   ├── string.h                  # Minimal libc stubs
│   └── em_device.h               # Device-specific macros
├── scripts/
│   ├── setup-rail.sh             # Automated Gecko SDK setup
│   └── download-rail-direct.sh   # Direct download (no Git LFS)
├── docs/
│   ├── MULTI_DEVICE_SUPPORT.md   # Multi-device planning
│   └── INTEGRATION_GUIDE.md      # This file
├── .github/
│   └── workflows/
│       └── ci.yml                # GitHub Actions CI/CD
└── README.md                     # User documentation
```

---

## Build System Architecture

### Step 1: Cargo.toml Configuration

**File**: `Cargo.toml`

```toml
[package]
name = "efr32mg24-rail-sys"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[dependencies]
cty = "0.2"  # C type definitions for no_std

[build-dependencies]
bindgen = "0.69"  # Generate Rust FFI from C headers
sha2 = "0.10"     # Blob integrity verification

[package.metadata.docs.rs]
default-target = "thumbv8m.main-none-eabihf"
targets = []
rustdoc-args = ["--cfg", "docsrs"]
```

**Key Decisions**:
- `cty` for no_std C types (instead of std::os::raw)
- `bindgen` for automatic FFI generation
- `sha2` for production-quality blob verification
- docs.rs metadata for better documentation

### Step 2: Build Script Structure

**File**: `build.rs`

```rust
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use sha2::{Sha256, Digest};

fn main() {
    // Step 1: Get target architecture
    let target = env::var("TARGET").unwrap();

    // Step 2: Only link blobs for ARM Cortex-M
    if target.starts_with("thumbv") {
        link_rail_blob();
    }

    // Step 3: Generate bindings if headers exist
    if headers_available() {
        generate_bindings();
    } else {
        create_placeholder_bindings();
    }

    // Step 4: Re-run triggers
    println!("cargo:rerun-if-changed=blobs/");
    println!("cargo:rerun-if-changed=vendor/");
}
```

**Architecture Principles**:
1. **Target-specific linking** - Only link for ARM targets
2. **Graceful degradation** - Compile without blobs/headers (dev workflow)
3. **Automated bindgen** - No manual FFI code
4. **Rebuild optimization** - Only re-run when files change

---

## Platform-Independent Headers

### Challenge

RAIL headers include system headers (`string.h`, `em_device.h`) which:
- Vary across platforms (macOS vs Linux vs Windows)
- Don't exist in freestanding ARM environment
- Cause bindgen failures with cross-compilation

### Solution: Minimal Header Stubs

**Step 1**: Create `include/string.h`

```c
// Minimal string.h for RAIL bindings generation
#ifndef _STRING_H_
#define _STRING_H_

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Memory functions used by RAIL
void *memcpy(void *dest, const void *src, size_t n);
void *memset(void *s, int c, size_t n);
void *memmove(void *dest, const void *src, size_t n);
int memcmp(const void *s1, const void *s2, size_t n);

#ifdef __cplusplus
}
#endif

#endif
```

**Why This Works**:
- Provides only declarations (no implementation)
- Compatible across all platforms
- Bindgen only needs type signatures

**Step 2**: Create `include/em_device.h`

```c
// IMPORTANT: This stub is configured for EFR32MG24 (Series 2 Config 4).
// For other EFR32 variants, adjust _SILICON_LABS_32B_SERIES_2_CONFIG:
//   - EFR32MG24: CONFIG = 4
//   - EFR32MG25: CONFIG = 5
//   - EFR32MG27: CONFIG = 7
//   - EFR32MG28: CONFIG = 8

#ifndef EM_DEVICE_H
#define EM_DEVICE_H

#include <stdint.h>
#include <stdbool.h>

// EFR32MG24 Series 2 Config 4
#define _SILICON_LABS_32B_SERIES_2_CONFIG 4
#define _SILICON_LABS_32B_SERIES_2_CONFIG_4 1

// Cortex-M33 core definitions
#define __CM33_REV                0x0001U
#define __FPU_PRESENT             1U
#define __DSP_PRESENT             1U

#endif
```

**Device-Specific Configuration**:
- Hardcoded for EFR32MG24 initially
- Documented for future multi-device support
- Matches RAIL library expectations

---

## Bindgen Integration

### Step 1: Clang Arguments

**File**: `build.rs` (continued)

```rust
let clang_args = vec![
    // Custom include directory (string.h, em_device.h)
    "-Iinclude",

    // RAIL headers directory
    "-Ivendor/rail",

    // Chip-specific headers
    "-Ivendor/rail/chip/efr32xg2x",

    // Freestanding environment (no system libc)
    "-ffreestanding",

    // Suppress warnings
    "-Wno-everything",
];
```

**Critical Points**:
1. `-Iinclude` MUST come first (overrides system headers)
2. `-ffreestanding` prevents libc assumptions
3. `-Wno-everything` suppresses bindgen noise

### Step 2: Bindgen Configuration

```rust
let bindings = bindgen::Builder::default()
    .header("vendor/rail/rail.h")
    .clang_args(&clang_args)

    // no_std compatibility
    .use_core()
    .ctypes_prefix("cty")

    // Skip layout tests (fail on cross-compilation)
    .layout_tests(false)

    // Allowlist RAIL APIs only
    .allowlist_function("RAIL_.*")
    .allowlist_type("RAIL_.*")
    .allowlist_var("RAIL_.*")

    .generate()
    .expect("Unable to generate RAIL bindings");
```

**Configuration Rationale**:
- `.use_core()` - no_std support
- `.ctypes_prefix("cty")` - Consistent C types
- `.layout_tests(false)` - Avoid cross-compilation issues
- Allowlist pattern - Only expose RAIL APIs (not string.h, etc.)

### Step 3: Generated Output

**Result**: `target/.../out/bindings.rs` (3,974 lines)

```rust
// Sample generated code:
pub const RAIL_DMA_INVALID: u32 = 255;
pub type RAIL_Handle_t = *mut RAIL_Handle;

extern "C" {
    pub fn RAIL_Init(
        railCfg: *mut RAIL_Config_t,
        cb: RAIL_Callback_t,
    ) -> RAIL_Handle_t;

    pub fn RAIL_GetVersion(
        version: *mut RAIL_Version_t,
        verbose: bool,
    );
}
```

**Quality Check**:
- All functions have `extern "C"` linkage
- Pointers properly typed
- Constants correctly defined
- No unsafe blocks (implicitly unsafe FFI)

---

## Blob Linking

### Step 1: Blob Path Resolution

**File**: `build.rs` (linking section)

```rust
// Use absolute path for cross-compilation
let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
let blob_dir = manifest_dir.join("blobs/efr32mg24b220");
let blob_path = blob_dir.join("librail_efr32xg24_gcc_release.a");
```

**Why Absolute Paths**:
- Relative paths fail in cross-compilation
- `CARGO_MANIFEST_DIR` is always correct
- Works in nested workspace builds

### Step 2: Link Instructions

```rust
if blob_path.exists() {
    println!("cargo:rustc-link-search=native={}", blob_dir.display());
    println!("cargo:rustc-link-lib=static=rail_efr32xg24_gcc_release");
}
```

**Cargo Directives**:
- `rustc-link-search` - Where to find the library
- `rustc-link-lib=static` - Link statically (embedded)
- Name without `lib` prefix or `.a` suffix

### Step 3: Graceful Degradation

```rust
if blob_path.exists() {
    // Link blob
} else {
    println!("cargo:warning=RAIL blob not found.");
    println!("cargo:warning=This is expected during development.");
    // Still compile - create empty bindings
}
```

**Development Workflow**:
- Crate compiles without blobs
- Allows incremental setup
- Prevents build failures during setup

---

## SHA256 Verification

### Step 1: Expected Hash Storage

**File**: `build.rs` (top-level constant)

```rust
/// Expected SHA256 hash of librail_efr32xg24_gcc_release.a from Gecko SDK v4.4.5
const EXPECTED_BLOB_SHA256: &str =
    "072f0ff0eb708091e49a3fd536c45f4f0568ed6f163c2889f5a8536b234c66a2";
```

**Hash Source**: Downloaded from official Gecko SDK v4.4.5

### Step 2: Verification Function

```rust
fn sha256_file(path: &Path) -> Result<String, std::io::Error> {
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
```

**Implementation Details**:
- 8 KB buffer for performance
- Streaming hash (doesn't load entire file)
- Standard SHA256 from `sha2` crate

### Step 3: Verification Logic

```rust
match sha256_file(&blob_path) {
    Ok(hash) => {
        if hash != EXPECTED_BLOB_SHA256 {
            println!("cargo:warning=RAIL blob SHA256 mismatch!");
            println!("cargo:warning=  Expected: {}", EXPECTED_BLOB_SHA256);
            println!("cargo:warning=  Got:      {}", hash);
            println!("cargo:warning=  The blob may be corrupted or from a different Gecko SDK version.");
        } else {
            println!("cargo:warning=RAIL blob integrity verified (SHA256 OK)");
        }
    }
    Err(e) => {
        println!("cargo:warning=Could not verify RAIL blob SHA256: {}", e);
    }
}
```

**Error Handling**:
- Mismatch → Warning (not error, for flexibility)
- Shows expected vs actual hash
- Suggests possible causes
- Success → Confirmation message

---

## Safety Documentation

### Step 1: Crate-Level Documentation

**File**: `src/lib.rs`

```rust
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! Low-level FFI bindings to Silicon Labs RAIL.
//!
//! # Safety
//!
//! This crate provides raw, unsafe FFI bindings. All functions
//! are `unsafe` and require careful handling of:
//!
//! - Memory safety: RAIL operates on hardware registers and DMA buffers
//! - Thread safety: RAIL callbacks execute in interrupt context
//! - Lifetime management: RAIL handles must outlive resources
//! - Initialization order: RAIL requires specific init sequences
//!
//! **Do not use this crate directly.** Use `efr32mg24-hal::radio` instead.
```

**Safety Categories**:
1. **Memory safety** - DMA, hardware registers
2. **Thread safety** - Interrupt context
3. **Lifetime** - Handle management
4. **Initialization** - Sequencing requirements

### Step 2: MSLA Licensing

```rust
//! # MSLA License
//!
//! This crate links against Silicon Labs proprietary binary blobs
//! licensed under the Master Software License Agreement (MSLA).
//!
//! **Key restrictions**:
//! - Binary blobs may only be used with Silicon Labs hardware
//! - No reverse engineering or modification of blobs
//! - Distribution requires MSLA compliance
```

**Legal Clarity**:
- Explicit license notice
- Key restrictions highlighted
- Points to LICENSE-MSLA file

### Step 3: Usage Warning

```rust
//! # Usage
//!
//! ```no_run
//! use efr32mg24_rail_sys as rail;
//!
//! // SAFETY: This is highly unsafe and provided only as an example.
//! unsafe {
//!     // let handle = rail::RAIL_Init(&config, &callback);
//! }
//! ```
//!
//! **Do not use this API directly. Use `efr32mg24-hal::radio` instead.**
```

**User Guidance**:
- Shows minimal example
- Emphasizes unsafe nature
- Directs to safe wrapper

---

## CI/CD Integration

### GitHub Actions Workflow

**File**: `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    paths:
      - 'efr32mg24-rail-sys/**'
  pull_request:

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: thumbv8m.main-none-eabihf
          components: rustfmt, clippy

      - name: Check format
        run: cargo fmt -- --check

      - name: Clippy
        run: cargo clippy --target thumbv8m.main-none-eabihf -- -D warnings

      - name: Check build (graceful degradation)
        run: cargo check --target thumbv8m.main-none-eabihf

      - name: Check documentation
        run: cargo doc --target thumbv8m.main-none-eabihf --no-deps
```

**CI Goals**:
1. Format verification (rustfmt)
2. Lint checks (clippy)
3. Graceful degradation test (build without blobs)
4. Documentation generation

---

## Troubleshooting

### Issue 1: "string.h not found"

**Symptoms**:
```
vendor/rail/rail.h:35:10: fatal error: 'string.h' file not found
```

**Solution**:
Ensure `include/string.h` exists and `-Iinclude` is in clang_args.

### Issue 2: "Unsupported platform!"

**Symptoms**:
```
rail_chip_specific.h:150:2: error: "Unsupported platform!"
```

**Solution**:
Check `include/em_device.h` has correct `_SILICON_LABS_32B_SERIES_2_CONFIG` value.

### Issue 3: "Library not found"

**Symptoms**:
```
error: could not find native static library `rail_efr32xg24_gcc_release`
```

**Solution**:
Use absolute paths in `build.rs` via `CARGO_MANIFEST_DIR`.

### Issue 4: "SHA256 mismatch"

**Symptoms**:
```
cargo:warning=RAIL blob SHA256 mismatch!
```

**Solution**:
Re-download RAIL blob from Gecko SDK v4.4.5 or verify expected hash.

---

## Verification Checklist

After completing integration, verify:

- [ ] Crate compiles: `cargo build --target thumbv8m.main-none-eabihf`
- [ ] No warnings (except build.rs informational)
- [ ] Clippy passes: `cargo clippy --target thumbv8m.main-none-eabihf -- -D warnings`
- [ ] Format passes: `cargo fmt -- --check`
- [ ] Docs build: `cargo doc --no-deps`
- [ ] SHA256 verified: Build output shows "SHA256 OK"
- [ ] Bindings generated: 3,974 lines in `bindings.rs`
- [ ] No unsafe in lib.rs (FFI is implicitly unsafe)

---

## Next Steps

1. **Create safe HAL wrapper** (`efr32mg24-hal::radio`)
2. **Test on hardware** (XIAO MG24 Sense)
3. **Add protocol support** (BLE, Zigbee, Thread)
4. **Multi-device support** (MG25, MG27, MG28)
5. **Publish to crates.io**

---

## References

- [Silicon Labs RAIL API Documentation](https://docs.silabs.com/rail/latest/)
- [Gecko SDK Repository](https://github.com/SiliconLabs/gecko_sdk)
- [Rust FFI Guide](https://doc.rust-lang.org/nomicon/ffi.html)
- [bindgen User Guide](https://rust-lang.github.io/rust-bindgen/)

---

**Document Version**: 1.0.0
**Last Updated**: 2025-12-28
**Phase**: D1-RAIL (Radio Foundation)
**Status**: Complete

<!-- META: last_updated=2025-12-28 version=1.0.0 phase=D1-RAIL status=complete author=orchestrator -->
