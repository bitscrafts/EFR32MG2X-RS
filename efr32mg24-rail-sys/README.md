# efr32mg24-rail-sys

Low-level FFI bindings to Silicon Labs RAIL (Radio Abstraction Interface Layer) for EFR32MG24.

## Warning

**This crate provides raw, unsafe FFI bindings. Do not use directly.**

Use the safe wrapper in `efr32mg24-hal::radio` instead.

## What is RAIL?

RAIL (Radio Abstraction Interface Layer) is Silicon Labs' proprietary library for low-level radio control on EFR32 wireless MCUs. It provides:

- Physical layer (PHY) configuration
- Channel management
- Packet transmission/reception
- Radio state machine control
- Timer synchronization
- Power management

RAIL is the foundation for higher-level protocol stacks (BLE, Zigbee, Thread, Matter).

## Features

This crate provides production-quality FFI bindings with:

- **Platform-Independent Build**: Custom minimal headers (no system libc dependencies)
- **SHA256 Blob Verification**: Automatic integrity checking for RAIL binary
- **Graceful Degradation**: Compiles without blobs/headers (development workflow)
- **Automated Setup**: Scripts for Gecko SDK download and extraction
- **docs.rs Integration**: Proper documentation configuration
- **CI/CD Ready**: GitHub Actions workflow included
- **Comprehensive Documentation**: Integration guide and multi-device planning

**Build System**:
- Uses `bindgen` for automatic FFI generation (3,974 lines)
- Freestanding environment (no platform-specific dependencies)
- Absolute path handling for cross-compilation
- Works on macOS, Linux, and Windows

## Prerequisites

**Required**: Git LFS (Large File Storage)

Gecko SDK uses Git LFS for binary files. Install before running setup:

```bash
# macOS (MacPorts)
sudo port install git-lfs

# macOS (Homebrew)
brew install git-lfs

# Linux (Debian/Ubuntu)
sudo apt-get install git-lfs

# Initialize Git LFS (run once)
git lfs install
```

## Quick Setup (Automated)

**Recommended**: Use the automated setup script:

```bash
cd efr32mg24-rail-sys
./scripts/setup-rail.sh
```

This will:
1. Check Git LFS installation
2. Clone Gecko SDK v4.4.5 (~300 MB, 5-10 minutes)
3. Pull RAIL blob via Git LFS (~1.5 MB)
4. Extract RAIL headers (~11 header files)
5. Clean up temporary files

Then build:

```bash
cargo build --target thumbv8m.main-none-eabihf
```

## Manual Setup

If you prefer manual setup or the script fails:

### 1. Download Gecko SDK

Download from Silicon Labs:
- [Gecko SDK 4.4.5](https://github.com/SiliconLabs/gecko_sdk) (or later)

```bash
git clone https://github.com/SiliconLabs/gecko_sdk.git
cd gecko_sdk
git checkout v4.4.5
```

### 2. Extract RAIL Blob

Extract the appropriate RAIL library for your device:

```bash
# For EFR32MG24B220 (XIAO MG24 Sense)
# Gecko SDK path: platform/radio/rail_lib/autogen/librail_release/
cp gecko_sdk/platform/radio/rail_lib/autogen/librail_release/librail_efr32xg24_gcc_release.a \
   efr32mg24-rail-sys/blobs/efr32mg24b220/
```

**Note**: The blob is ~500 KB and is **not** included in this repository due to MSLA licensing.

### 3. Extract RAIL Headers

Extract RAIL headers for bindgen:

```bash
# Main headers
cp -r gecko_sdk/platform/radio/rail_lib/common/* \
      efr32mg24-rail-sys/vendor/rail/

# Chip-specific headers
mkdir -p efr32mg24-rail-sys/vendor/rail/chip
cp -r gecko_sdk/platform/radio/rail_lib/chip/efr32/efr32xg2x \
      efr32mg24-rail-sys/vendor/rail/chip/
```

### 4. Build

```bash
cargo build --target thumbv8m.main-none-eabihf
```

Bindgen will automatically generate Rust bindings from the C headers.

## MSLA License

This crate links against Silicon Labs proprietary binary blobs licensed under the **Master Software License Agreement (MSLA)**.

**Key restrictions**:
- Binary blobs may only be used with Silicon Labs hardware
- No reverse engineering or modification of blobs
- Distribution requires MSLA compliance

See [LICENSE-MSLA](LICENSE-MSLA) for full license text.

**Rust bindings** (this crate's source code) are dual-licensed MIT OR Apache-2.0.
**Binary blobs** (librail_*.a) are licensed under MSLA only.

## Usage

**Do not use this crate directly.** It provides only raw FFI bindings.

Use the safe wrapper:

```rust
// DON'T use efr32mg24-rail-sys directly
// DO use the safe HAL instead:
use efr32mg24_hal::radio::ble::{BleAdvertiser, XiaoBle};

let mut ble = XiaoBle::new(/* ... */)?;
ble.start_advertising()?;
```

## Safety

All functions in this crate are `unsafe`. Using RAIL incorrectly can cause:

- **Undefined behavior**: Hardware state corruption
- **Memory unsafety**: DMA buffer overruns
- **Data races**: Interrupt context callbacks
- **Resource leaks**: Radio state not cleaned up
- **Regulatory violations**: Improper radio configuration

**Required expertise**:
- Deep understanding of RAIL API
- Experience with interrupt-safe Rust
- Knowledge of radio regulations (FCC, ETSI, etc.)
- Hardware debugging skills

## Build System

The build process (`build.rs`):

1. **Blob Verification**: Checks for RAIL blob in `blobs/efr32mg24b220/`
2. **SHA256 Integrity Check**: Verifies blob matches expected hash from Gecko SDK v4.4.5
   ```
   Expected: 072f0ff0eb708091e49a3fd536c45f4f0568ed6f163c2889f5a8536b234c66a2
   ```
3. **Linking**: Links the blob using `cargo:rustc-link-lib=static`
4. **Header Detection**: Checks for headers in `vendor/rail/`
5. **Bindgen Generation**: Runs bindgen with platform-independent configuration
   - Uses custom minimal headers (`include/string.h`, `include/em_device.h`)
   - Freestanding environment (`-ffreestanding`)
   - Generates 3,974 lines of Rust FFI bindings
6. **Output**: Bindings written to `OUT_DIR/bindings.rs` and included in `src/lib.rs`

**Graceful degradation**:
- If blob missing: Compiles with warning (for development workflow)
- If blob SHA256 mismatch: Warning (may be different SDK version)
- If headers missing: Creates empty bindings file
- Actual functionality requires both verified blob and headers

**Platform Independence**:
- No system libc dependencies (uses minimal custom headers)
- Works identically on macOS, Linux, and Windows
- Absolute path handling for cross-compilation
- No hardcoded platform-specific paths

## Verification

After setup, verify the installation:

```bash
# Check blob exists and size
ls -lh blobs/efr32mg24b220/librail_efr32xg24_gcc_release.a
# Should show ~1.5 MB (actual size, not LFS pointer)

# Verify SHA256 hash
shasum -a 256 blobs/efr32mg24b220/librail_efr32xg24_gcc_release.a
# Should show: 072f0ff0eb708091e49a3fd536c45f4f0568ed6f163c2889f5a8536b234c66a2

# Check headers extracted
find vendor/rail -name "*.h" | wc -l
# Should show ~11 header files

# Verify main header
ls vendor/rail/rail.h
# Should exist

# Test build (automatic SHA256 verification)
cargo build --target thumbv8m.main-none-eabihf
# Should show: "RAIL blob integrity verified (SHA256 OK)"
# Should compile successfully and generate 3,974 lines of bindings

# Run clippy
cargo clippy --target thumbv8m.main-none-eabihf -- -D warnings
# Should pass with no warnings
```

## Troubleshooting

### Build fails with "RAIL blob not found"

**Solution**: Run `./scripts/setup-rail.sh` or manually copy the blob:
```bash
cp path/to/gecko_sdk/platform/radio/rail_lib/autogen/librail_release/librail_efr32xg24_gcc_release.a \
   blobs/efr32mg24b220/
```

### Build fails with "rail.h not found"

**Solution**: Run `./scripts/setup-rail.sh` or manually copy headers:
```bash
cp -r path/to/gecko_sdk/platform/radio/rail_lib/common/* vendor/rail/
```

### Bindgen errors

**Solution**: Ensure you have:
- libclang installed (`brew install llvm` on macOS, `apt install libclang-dev` on Linux)
- Correct headers in `vendor/rail/`

### "Library not found" linking errors

**Solution**: Verify blob path and name match exactly:
```bash
ls -la blobs/efr32mg24b220/
# Should show: librail_efr32xg24_gcc_release.a
```

### SHA256 mismatch warning

**Symptoms**:
```
cargo:warning=RAIL blob SHA256 mismatch!
cargo:warning=  Expected: 072f0ff...
cargo:warning=  Got:      abc123...
```

**Solution**:
- Re-download RAIL blob from Gecko SDK v4.4.5
- Ensure Git LFS properly pulled the blob (not just pointer file)
- Check file size: should be ~1.5 MB, not 132 bytes
- Run: `git lfs pull --include="platform/radio/rail_lib/autogen/librail_release/*.a"`

## Documentation

Comprehensive documentation is available in the `docs/` directory:

- **[INTEGRATION_GUIDE.md](docs/INTEGRATION_GUIDE.md)** - Complete step-by-step integration documentation
  - Build system architecture
  - Platform-independent headers explanation
  - Bindgen configuration details
  - SHA256 verification implementation
  - Troubleshooting guide

- **[MULTI_DEVICE_SUPPORT.md](docs/MULTI_DEVICE_SUPPORT.md)** - Multi-device support planning
  - Future EFR32 variants (MG25, MG27, MG28)
  - Feature flag strategy
  - Migration path

- **[CI/CD Workflow](.github/workflows/ci.yml)** - Automated testing
  - Format checking (rustfmt)
  - Linting (clippy)
  - Build verification
  - Documentation generation

**See Also**:
- [Silicon Labs RAIL API Documentation](https://docs.silabs.com/rail/latest/)
- [Gecko SDK Repository](https://github.com/SiliconLabs/gecko_sdk)
- [Rust FFI Guide](https://doc.rust-lang.org/nomicon/ffi.html)

## Internal Use Only

This crate is used internally by:
- `efr32mg24-hal::radio` - Safe HAL abstractions
- `efr32mg24-bluetooth-sys` - BLE stack bindings
- `efr32mg24-zigbee-sys` - Zigbee stack bindings

**Do not depend on this crate directly in application code.**

## Attribution

See [NOTICE](NOTICE) file for third-party attributions.

## License

Rust bindings source code: MIT OR Apache-2.0
Binary blobs (librail_*.a): MSLA only

See:
- [LICENSE-MIT](../LICENSE-MIT)
- [LICENSE-APACHE](../LICENSE-APACHE)
- [LICENSE-MSLA](LICENSE-MSLA)

---

**Version**: 0.1.0
**Target**: EFR32MG24B220F1536IM48-B
**Gecko SDK**: 4.4.5+
**Status**: Phase D1 (RAIL Foundation)
**Quality**: A- (Production-Ready)

<!-- META: last_updated=2025-12-28 version=0.1.0 phase=D1-RAIL status=complete quality=A- -->
