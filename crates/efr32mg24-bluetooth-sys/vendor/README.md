# Vendor Headers Directory

This directory contains Silicon Labs Bluetooth BGAPI header files extracted from Gecko SDK.

## Important

**These files are NOT included in the repository** due to MSLA licensing restrictions.

You must extract them from Gecko SDK yourself.

## How to Obtain

### Automated Setup (Recommended)

Run the setup script from the crate root:

```bash
cd efr32mg24-bluetooth-sys
bash scripts/setup-bluetooth.sh
```

### Manual Extraction

#### 1. Download Gecko SDK

```bash
git clone https://github.com/SiliconLabs/gecko_sdk.git
cd gecko_sdk
git checkout v4.4.5
```

#### 2. Locate Bluetooth Headers

The BGAPI headers are typically in:
- `protocol/bluetooth/inc/`
- `protocol/bluetooth/bgapi/inc/`
- `protocol/bluetooth/include/`

Key header file: `sl_bt_api.h`

```bash
find gecko_sdk/protocol/bluetooth -name "sl_bt_api.h"
```

#### 3. Copy to This Directory

```bash
# Copy all Bluetooth headers
cp gecko_sdk/protocol/bluetooth/inc/*.h \
   efr32mg24-bluetooth-sys/vendor/bluetooth/
```

## Required Headers

At minimum, you need:

- `sl_bt_api.h` - Main BGAPI header
- `sl_bt_types.h` - Bluetooth type definitions
- `sl_status.h` - Status code definitions (if separate)

## Directory Structure

```
vendor/
├── README.md (this file)
└── bluetooth/
    ├── sl_bt_api.h        (NOT in git, extract from SDK)
    ├── sl_bt_types.h      (NOT in git, extract from SDK)
    └── *.h                (other Bluetooth headers)
```

## Usage

These headers are used by `build.rs` to generate Rust FFI bindings via `bindgen`:

```rust
// build.rs
bindgen::Builder::default()
    .header("vendor/bluetooth/sl_bt_api.h")
    .clang_args(&["-Ivendor/bluetooth"])
    .generate()
```

## License

Headers are part of Gecko SDK and subject to Silicon Labs MSLA.

See [LICENSE-MSLA](../LICENSE-MSLA) for restrictions.

## .gitignore

Header files are excluded via `.gitignore`:
```
*.h
*.hpp
```

This prevents accidental commits of MSLA-licensed files.

## Verification

To verify headers are correct:

```bash
# Check main header exists
ls -lh vendor/bluetooth/sl_bt_api.h

# Search for key BGAPI function declarations
grep "sl_bt_advertiser_start" vendor/bluetooth/sl_bt_api.h
grep "sl_bt_gatt_server" vendor/bluetooth/sl_bt_api.h
```

## Next Steps

After extracting headers:

1. Run `cargo build --target thumbv8m.main-none-eabihf`
2. Bindgen will auto-generate Rust bindings
3. Check `target/*/build/efr32mg24-bluetooth-sys-*/out/bindings.rs`
