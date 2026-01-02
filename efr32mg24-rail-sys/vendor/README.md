# Vendor Headers Directory

This directory contains Silicon Labs C header files for RAIL, used by bindgen to generate Rust FFI bindings.

## Important

**These files are NOT included in the repository.**

You must extract them from Gecko SDK yourself.

## How to Obtain

### 1. Download Gecko SDK

```bash
git clone https://github.com/SiliconLabs/gecko_sdk.git
cd gecko_sdk
git checkout v4.4.5  # Or latest stable version
```

### 2. Locate RAIL Headers

```bash
# Main RAIL header location:
# platform/radio/rail_lib/common/

find gecko_sdk/platform/radio/rail_lib -name "*.h"
```

### 3. Copy Headers to This Directory

```bash
# Copy all RAIL headers
cp -r gecko_sdk/platform/radio/rail_lib/common/* \
      efr32mg24-rail-sys/vendor/rail/

# Copy chip-specific headers
cp -r gecko_sdk/platform/radio/rail_lib/chip/efr32/efr32xg2x/* \
      efr32mg24-rail-sys/vendor/rail/chip/
```

## Required Headers

Minimum required files for bindgen:

```
vendor/rail/
├── rail.h                    # Main RAIL API
├── rail_types.h              # RAIL type definitions
├── rail_chip_specific.h      # Chip-specific definitions
├── rail_features.h           # Feature detection
└── chip/
    └── efr32xg24_rail_config.h  # EFR32MG24 configuration
```

## License

Headers are licensed under Zlib license (permissive).

See [NOTICE](../NOTICE) for attribution.

## Directory Structure

```
vendor/
├── README.md (this file)
└── rail/
    ├── rail.h
    ├── rail_types.h
    ├── rail_chip_specific.h
    ├── rail_features.h
    ├── rail_ble.h
    ├── rail_ieee802154.h
    └── chip/
        └── efr32xg24_rail_config.h
```

## Bindgen Configuration

The headers are processed by bindgen in `build.rs`:

```rust
bindgen::Builder::default()
    .header("vendor/rail/rail.h")
    .use_core()
    .ctypes_prefix("cty")
    .allowlist_function("RAIL_.*")
    .generate()
```

## .gitignore

Headers are excluded via `.gitignore`:
```
*.h
```

This keeps the repository size small and respects licensing.

## Verification

To verify headers are correct:

```bash
# Check main header exists
ls vendor/rail/rail.h

# Verify RAIL_Init declaration
grep "RAIL_Init" vendor/rail/rail.h
# Should show: RAIL_Handle_t RAIL_Init(...);
```
