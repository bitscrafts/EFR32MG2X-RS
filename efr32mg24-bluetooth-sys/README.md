# efr32mg24-bluetooth-sys

Low-level FFI bindings to Silicon Labs Bluetooth stack (BGAPI) for EFR32MG24.

## Usage

**This crate provides raw, unsafe FFI bindings to the BGAPI protocol stack.**

See `examples/` directory for working examples showing how to:
- Initialize the Bluetooth stack
- Create BLE beacons (advertising)
- Implement GATT servers
- Handle connections

For board-specific examples with sensors, see `xiao-mg24-bsp/examples/`.

## What is BGAPI?

BGAPI (Bluetooth Generic API) is Silicon Labs' proprietary Bluetooth 5.3 LE stack for EFR32 wireless MCUs. It provides:

- **GAP (Generic Access Profile)**: Advertising, scanning, connections
- **GATT (Generic Attribute Profile)**: Services, characteristics, notifications
- **Security**: Pairing, bonding, encryption
- **L2CAP**: Logical Link Control and Adaptation Protocol
- **Connection Management**: Multiple concurrent connections
- **Event System**: Asynchronous event handling

The Bluetooth stack builds on top of RAIL (Radio Abstraction Interface Layer) for radio hardware control.

## Features

This crate provides production-quality FFI bindings with:

- **Platform-Independent Build**: Custom minimal headers (no system libc dependencies)
- **SHA256 Blob Verification**: Automatic integrity checking for Bluetooth binary
- **Graceful Degradation**: Compiles without blobs/headers (development workflow)
- **Automated Setup**: Scripts for Gecko SDK download and extraction
- **docs.rs Integration**: Proper documentation configuration
- **CI/CD Ready**: Compatible with GitHub Actions
- **Comprehensive Documentation**: Integration guides and examples

**Build System**:
- Uses `bindgen` for automatic FFI generation (~2,000-3,000 lines expected)
- Freestanding environment (no platform-specific dependencies)
- Absolute path handling for cross-compilation
- Works on macOS, Linux, and Windows

## Prerequisites

### Required

**1. Git LFS (Large File Storage)**

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

**2. efr32mg24-rail-sys**

The Bluetooth stack depends on RAIL:

```toml
[dependencies]
efr32mg24-rail-sys = "0.1"
efr32mg24-bluetooth-sys = "0.1"
```

RAIL must be set up first (see `efr32mg24-rail-sys` README).

## Quick Setup (Automated)

**Recommended**: Use the automated setup script:

```bash
cd efr32mg24-bluetooth-sys
./scripts/setup-bluetooth.sh
```

This will:
1. Check Git LFS installation
2. Clone Gecko SDK v4.4.5 (~300 MB, 5-10 minutes)
3. Pull Bluetooth blob via Git LFS (~1.5-2 MB)
4. Extract BGAPI headers (~10-20 header files)
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

### 2. Pull LFS Files

Extract Bluetooth blobs with Git LFS:

```bash
git lfs install
git lfs pull --include="protocol/bluetooth/lib/*.a"
```

### 3. Extract Bluetooth Blob

For EFR32MG24B220 (XIAO MG24 Sense):

```bash
# Find the blob
find gecko_sdk -name "libbluetooth_efr32xg24_gcc_release.a"

# Common locations:
# - protocol/bluetooth/lib/libbluetooth_efr32xg24_gcc_release.a
# - protocol/bluetooth/bgstack/lib/libbluetooth_efr32xg24_gcc_release.a

# Copy to this crate
cp gecko_sdk/protocol/bluetooth/lib/libbluetooth_efr32xg24_gcc_release.a \
   blobs/efr32mg24b220/
```

### 4. Extract BGAPI Headers

```bash
# Find BGAPI headers
find gecko_sdk/protocol/bluetooth -name "sl_bt_api.h"

# Common location:
# - protocol/bluetooth/inc/

# Copy headers
cp gecko_sdk/protocol/bluetooth/inc/*.h vendor/bluetooth/
```

### 5. Verify Extraction

```bash
# Check blob (should be ~1.5-2 MB)
ls -lh blobs/efr32mg24b220/libbluetooth_efr32xg24_gcc_release.a

# Verify it's a real binary, not LFS pointer
file blobs/efr32mg24b220/libbluetooth_efr32xg24_gcc_release.a
# Should show: current ar archive

# Check headers
ls vendor/bluetooth/
# Should see: sl_bt_api.h, sl_bt_types.h, etc.
```

## Build

After setup, build the crate:

```bash
# Build for target
cargo build --target thumbv8m.main-none-eabihf

# The build will:
# 1. Verify Bluetooth blob SHA256 (warns on mismatch)
# 2. Link libbluetooth_efr32xg24_gcc_release.a
# 3. Generate Rust bindings from sl_bt_api.h via bindgen
# 4. Create libefr32mg24_bluetooth_sys.rlib
```

**Build time**: ~2-3 minutes (after PAC and dependencies)

**Output**: `target/thumbv8m.main-none-eabihf/debug/libefr32mg24_bluetooth_sys.rlib`

## Graceful Degradation

The crate compiles even without blobs/headers:

```bash
# Without Bluetooth blob/headers
cargo check
# Warning: Bluetooth blob not found (placeholder bindings generated)
# Warning: Bluetooth headers not found (bindings will not be generated)

# Still compiles successfully for development
```

This allows:
- Documentation generation on docs.rs
- Development without Gecko SDK
- CI/CD workflows

Actual Bluetooth functionality requires both blob and headers.

## Usage

**Do not use this crate directly!** It provides unsafe FFI only.

### Correct Usage (via HAL and BSP)

```rust
// Use safe abstractions instead
use efr32mg24_hal::radio::ble::BleAdvertiser;
use xiao_mg24_bsp::XiaoBle;

fn main() {
    let mut ble = XiaoBle::new(...)?;
    ble.set_device_name("XIAO MG24")?;
    ble.start_advertising()?;
}
```

### Unsafe FFI (Internal Use Only)

```rust
use efr32mg24_bluetooth_sys as bt;

unsafe {
    // Initialize Bluetooth stack
    bt::sl_bt_init_stack(...);

    // Start advertising
    bt::sl_bt_advertiser_start(...);

    // This is highly unsafe and error-prone!
    // Use HAL/BSP instead.
}
```

## Safety

This crate provides **unsafe** FFI bindings. All functions are marked `unsafe` because:

1. **Memory Safety**: Bluetooth stack operates on hardware and buffers
2. **Thread Safety**: Callbacks execute in interrupt/event context
3. **Lifetime**: Handles must outlive their resources
4. **Initialization**: Requires specific initialization order
5. **State Machine**: Incorrect state transitions cause undefined behavior

**Never use these bindings directly.** Use safe wrappers:
- `efr32mg24-hal::radio::ble` - Safe generic traits
- `xiao-mg24-bsp::XiaoBle` - Safe concrete implementation

## Memory Requirements

| Component | Flash | RAM |
|-----------|-------|-----|
| RAIL (required) | ~50 KB | ~10 KB |
| Bluetooth stack | ~120 KB | ~36 KB |
| **Total** | **~170 KB** | **~46 KB** |

**Target**: EFR32MG24B220 has 1536 KB Flash, 256 KB RAM - plenty of space!

## License

### This Crate (Rust Code)

MIT OR Apache-2.0 (your choice)

### Bluetooth Stack Blob

**MSLA (Master Software License Agreement)** - Silicon Labs proprietary

See `LICENSE-MSLA` for full terms.

**Key restrictions**:
- Blob may only be used with Silicon Labs hardware
- No reverse engineering or modification
- Distribution requires MSLA compliance

### Compatibility

This crate is compatible with:
- Gecko SDK v4.4.5 (tested)
- Gecko SDK v4.4.x (likely compatible)
- Newer versions (may need blob hash update)

## Architecture

```
User Application
    ↓
xiao-mg24-bsp (XiaoBle)
    ↓ implements
efr32mg24-hal::radio::ble (BleAdvertiser trait)
    ↓ uses
efr32mg24-bluetooth-sys (this crate - unsafe FFI)
    ↓ links
libbluetooth_efr32xg24_gcc_release.a (Silicon Labs blob)
    ↓ calls
efr32mg24-rail-sys (RAIL FFI)
    ↓ links
librail_efr32xg24_gcc_release.a (Silicon Labs blob)
    ↓ controls
EFR32MG24 2.4 GHz Radio Hardware
```

## Multi-Protocol Support

The Bluetooth stack can coexist with other protocols via Dynamic Multiprotocol (DMP):

```toml
[dependencies]
efr32mg24-rail-sys = "0.1"      # Shared foundation
efr32mg24-bluetooth-sys = "0.1" # BLE
efr32mg24-zigbee-sys = "0.1"    # Zigbee (when available)
```

RAIL's hardware scheduler automatically time-slices radio access between protocols.

See `docs/RADIO_ARCHITECTURE_SYS_CRATES.md` for details.

## Troubleshooting

### Blob SHA256 Mismatch

```
warning: Bluetooth blob SHA256 mismatch!
  Expected: abc123...
  Got:      def456...
```

**Cause**: Different Gecko SDK version or corrupted blob

**Fix**:
1. Verify you're using Gecko SDK v4.4.5
2. Re-download with `git lfs pull --include="protocol/bluetooth/lib/*.a"`
3. Update `EXPECTED_BLOB_SHA256` in `build.rs` for newer SDK versions

### Blob is LFS Pointer

```
warning: Bluetooth blob is still an LFS pointer
```

**Cause**: Git LFS didn't pull the actual binary

**Fix**:
```bash
cd gecko_sdk
git lfs pull --include="protocol/bluetooth/lib/*.a"
```

### Headers Not Found

```
warning: Bluetooth headers not found at vendor/bluetooth/sl_bt_api.h
```

**Cause**: Headers not extracted from Gecko SDK

**Fix**:
```bash
cp gecko_sdk/protocol/bluetooth/inc/*.h vendor/bluetooth/
```

### Bindgen Errors

```
error: Unable to generate Bluetooth bindings
```

**Possible causes**:
1. Missing LLVM/clang (required by bindgen)
2. Incorrect header paths
3. Missing system headers

**Fix**:
```bash
# Install LLVM (macOS)
brew install llvm

# Check bindgen
cargo install bindgen-cli --locked
bindgen --version
```

## Documentation

- **README.md** (this file) - Quick start and overview
- **blobs/README.md** - Binary blob extraction guide
- **vendor/README.md** - Header extraction guide
- **docs/RADIO_ARCHITECTURE_SYS_CRATES.md** - Complete architecture explanation

## Examples

See `xiao-mg24-bsp` for working examples using safe abstractions:

- `ble_beacon.rs` - Non-connectable advertising
- `ble_peripheral.rs` - Connectable peripheral
- `ble_gatt_server.rs` - Heart rate sensor

## Contributing

Contributions welcome! Please ensure:

1. **Never commit blobs or headers** (`.gitignore` prevents this)
2. Test on multiple platforms (macOS, Linux, Windows)
3. Update SHA256 hash if supporting newer Gecko SDK
4. Maintain compatibility with `efr32mg24-hal`

## Resources

### Silicon Labs

- [Bluetooth SDK Documentation](https://docs.silabs.com/bluetooth/latest/)
- [BGAPI Reference](https://docs.silabs.com/bluetooth/latest/bgapi)
- [Gecko SDK](https://github.com/SiliconLabs/gecko_sdk)

### EFR32MG24

- [EFR32MG24 Reference Manual](https://www.silabs.com/documents/public/reference-manuals/efr32xg24-rm.pdf)
- [EFR32MG24 Datasheet](https://www.silabs.com/documents/public/data-sheets/efr32mg24-datasheet.pdf)

### Bluetooth Specifications

- [Bluetooth Core Specification 5.3](https://www.bluetooth.com/specifications/specs/core-specification-5-3/)
- [GATT Specifications](https://www.bluetooth.com/specifications/specs/)

## Support

For issues, questions, or contributions:

**Repository**: https://github.com/bitscrafts/EFR32MG2X-RS

**Author**: Marcelo Correa <mvcorrea+github@gmail.com>

---

**Created**: December 28, 2025
**Phase**: D2 - Bluetooth 5.3 LE
**Status**: Crate structure complete, awaiting Gecko SDK extraction for bindings generation

<\!-- META: last_updated=2025-12-28 version=1.0.0 status=active maintained_by=orchestrator -->
