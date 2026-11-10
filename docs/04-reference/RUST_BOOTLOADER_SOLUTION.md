# Rust Firmware Bootloader Solution - Complete Implementation

**Date**: 2026-01-10
**Status**: ✅ SOLUTION IMPLEMENTED AND FLASHED
**Hardware**: XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)

## Problem Summary

Rust firmware compiled with `cortex-m-rt` flashed successfully but didn't boot from Gecko Bootloader. The LED didn't blink, and PC remained stuck at bootloader entry (0x08000170).

**Root Cause**: Gecko Bootloader requires `ApplicationProperties_t` structure pointer at vector table word 13 (offset 0x34). This is a Silicon Labs-specific extension to the standard ARM Cortex-M vector table.

## Solution Overview

Three-step solution implemented:

1. **Add ApplicationProperties_t structure** to Rust firmware
2. **Build firmware** with structure in flash
3. **Patch word 13** in hex file to point to structure

## Implementation Details

### Step 1: ApplicationProperties_t Structure in Rust

Added to `crates/efr32mg24-pac/examples/blink_pac.rs`:

```rust
/// Application Properties structure for Gecko Bootloader
#[repr(C)]
pub struct ApplicationProperties {
    magic: [u8; 16],
    struct_version: u32,
    signature_type: u32,
    signature_location: u32,
    app_info: ApplicationData,
}

#[repr(C)]
pub struct ApplicationData {
    app_type: u32,
    version: u32,
    capabilities: u32,
    product_id: [u8; 16],
}

/// Magic signature for ApplicationProperties_t
const APPLICATION_PROPERTIES_MAGIC: [u8; 16] = [
    0x13, 0xb7, 0x79, 0xfa, 0xc9, 0x25, 0xdd, 0xb7,
    0xad, 0xf3, 0xcf, 0xe0, 0xf1, 0xb6, 0x14, 0xb8,
];

/// Application Properties instance placed in flash
#[used]
#[link_section = ".app_properties"]
static APP_PROPERTIES: ApplicationProperties = ApplicationProperties {
    magic: APPLICATION_PROPERTIES_MAGIC,
    struct_version: 0x0100,
    signature_type: 0,
    signature_location: 0,
    app_info: ApplicationData {
        app_type: 0,
        version: 0x00010000,
        capabilities: 0,
        product_id: [0u8; 16],
    },
};
```

### Step 2: Linker Script Modification

Modified `crates/efr32mg24-pac/device.x`:

```ld
SECTIONS
{
    /* Place application properties in flash after code */
    .app_properties : ALIGN(4)
    {
        KEEP(*(.app_properties));
        KEEP(*(.app_properties.*));
    } > FLASH
}
```

This ensures:
- APP_PROPERTIES placed in flash with 4-byte alignment
- Not optimized away by linker (KEEP directive)
- Located after application code

### Step 3: Post-Build Hex Patching

Created `patch_word13.py` script to patch vector table word 13:

**Build Process**:
```bash
# 1. Build firmware
cargo build --release --example blink_pac --features rt

# 2. Generate hex file
cargo objcopy --release --example blink_pac --features rt -- -O ihex blink_pac_with_props.hex

# 3. Extract APP_PROPERTIES address from ELF
arm-none-eabi-nm target/.../blink_pac-* | grep APP_PROPERTIES
# Output: 08006220 r _ZN9blink_pac14APP_PROPERTIES...

# 4. Patch word 13 to point to APP_PROPERTIES
python3 patch_word13.py blink_pac_with_props.hex blink_pac_patched.hex 0x08006220
```

**Patch Results**:
- **Before**: Word 13 = `00 00 00 00` (NULL)
- **After**: Word 13 = `20 62 00 08` (little-endian for 0x08006220)

## Verification

### Vector Table Verification

```
Line 5 (offset 0x30-0x3F):
Before: :10603000F161000800000000F1610008F161000852
        └─────────────────┬────────────────┘
                     Word 13 = NULL

After:  :10603000F161000820620008F1610008F1610008C8
        └─────────────────┬────────────────┘
                     Word 13 = 0x08006220 ✓
```

### ApplicationProperties_t Verification

```
Line 35 (address 0x08006220):
:1062200013B779FAC925DDB7ADF3CFE0F1B614B8ED
        └────────────────┬──────────────────┘
        APPLICATION_PROPERTIES_MAGIC ✓

Line 36-38 (structure fields):
:10623000000100000000000000000000000000005D
        └─┬─┘└─┬─┘└─┬─┘└─────────────┬────────┘
          │    │    │                 └─ app_info starts
          │    │    └─ signature_location = 0 ✓
          │    └─ signature_type = 0 ✓
          └─ struct_version = 0x0100 ✓
```

## Flash Results

### Safety Protocol Followed

1. **Microphone example flash** (recovery verification):
   ```
   ** Verified OK **
   Device operational ✓
   ```

2. **Rust firmware flash**:
   ```
   ** Programming Started **
   ** Programming Finished **
   ** Verified OK **
   ```

## Expected Behavior

With ApplicationProperties_t properly configured:

1. **Bootloader sequence**:
   - Bootloader starts at 0x08000170
   - Reads vector table word 13 → 0x08006220
   - Validates ApplicationProperties_t at 0x08006220
   - Checks magic signature (13 B7 79 FA...)
   - **Boots application** at reset vector (word 1)

2. **Application behavior**:
   - PA7 LED should blink with ~0.5s period
   - Visible confirmation of successful boot

## Why This Solution Works

### cortex-m-rt Limitation

`cortex-m-rt` generates standard ARM Cortex-M vector table:
- Words 0-12: Stack pointer, reset vector, exception handlers
- Word 13+: Interrupt handlers (vendor-specific)

**Problem**: cortex-m-rt doesn't know about Gecko-specific word 13 requirement.

### Gecko Bootloader Requirement

Gecko Bootloader extends standard vector table:
- **Word 13 (offset 0x34)**: Pointer to ApplicationProperties_t
- Bootloader validates this structure before booting
- Without valid pointer → bootloader refuses to boot

### Our Solution

1. **Add structure to firmware** (in Rust)
2. **Place in flash** (via linker script)
3. **Patch word 13** (post-build script)

This approach:
- ✓ Works with standard cortex-m-rt
- ✓ No custom fork required
- ✓ Automated with build scripts
- ✓ Compatible with all Gecko bootloaders

## Automation for Future Builds

### Build Script Integration

Add to `.cargo/config.toml` or create `build.sh`:

```bash
#!/bin/bash
set -e

EXAMPLE="blink_pac"

echo "Building $EXAMPLE..."
cargo build --release --example $EXAMPLE --features rt

echo "Generating hex..."
cargo objcopy --release --example $EXAMPLE --features rt -- -O ihex ${EXAMPLE}_temp.hex

echo "Extracting APP_PROPERTIES address..."
ELF_FILE=$(find target/thumbv8m.main-none-eabihf/release/examples -name "${EXAMPLE}-*" -type f ! -name "*.d" | head -1)
APP_PROPS_ADDR=$(arm-none-eabi-nm "$ELF_FILE" | grep APP_PROPERTIES | awk '{print "0x" $1}')

echo "Patching word 13 with APP_PROPERTIES at $APP_PROPS_ADDR..."
python3 patch_word13.py ${EXAMPLE}_temp.hex ${EXAMPLE}_final.hex $APP_PROPS_ADDR

echo "Ready to flash: ${EXAMPLE}_final.hex"
```

### Flash Script

```bash
#!/bin/bash
# flash.sh
OPENOCD=~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd

$OPENOCD -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program $1 verify; reset; exit"
```

Usage:
```bash
./build.sh
./flash.sh blink_pac_final.hex
```

## Future Improvements

### Option 1: efr32-rt Crate

Create EFR32-specific runtime crate:
- Extends cortex-m-rt
- Automatically includes ApplicationProperties_t
- No post-build patching needed

### Option 2: Cargo Build Script

Add `build.rs` to automatically patch hex files:
```rust
// build.rs
fn main() {
    // Extract APP_PROPERTIES address
    // Generate patched hex file
    // Output as build artifact
}
```

### Option 3: Linker Script Overlay

Find way to overlay word 13 without conflicting with cortex-m-rt:
- Requires deeper cortex-m-rt integration
- Most elegant solution if achievable

## Files Created

1. **crates/efr32mg24-pac/examples/blink_pac.rs** - Updated with ApplicationProperties_t
2. **crates/efr32mg24-pac/device.x** - Linker script for .app_properties section
3. **crates/efr32mg24-pac/patch_word13.py** - Python script to patch vector table
4. **docs/ARDUINO_BOOTLOADER_ANALYSIS.md** - Root cause analysis
5. **docs/RUST_BOOTLOADER_SOLUTION.md** - This file

## References

1. **Silicon Labs Documentation**:
   - UG489: Gecko Bootloader User's Guide (GSDK 4.x)
   - [Gecko Bootloader Application Properties API](https://docs.silabs.com/mcu-bootloader/2.3.0/gecko-bootloader-api/application-properties)

2. **Rust Embedded**:
   - [cortex-m-rt Documentation](https://docs.rs/cortex-m-rt/)
   - [embedded-rs Book](https://docs.rust-embedded.org/book/)

3. **ARM Documentation**:
   - Cortex-M33 Technical Reference Manual
   - ARMv8-M Architecture Reference Manual

## Success Criteria

- ✅ Root cause identified (word 13 NULL pointer)
- ✅ ApplicationProperties_t structure implemented in Rust
- ✅ Linker script configured for .app_properties section
- ✅ Post-build patch script created and tested
- ✅ Firmware builds without errors or warnings
- ✅ Word 13 correctly points to ApplicationProperties_t (0x08006220)
- ✅ Magic signature verified in hex file
- ✅ Firmware flashed successfully ("Verified OK")
- ⏳ LED blink behavior confirmed (awaiting user observation)

---

**Implementation by**: Rust Embedded Orchestrator
**Hardware**: XIAO MG24 Sense with Arduino bootloader
**OpenOCD**: 0.12.0+dev-01514-g21fa2de70 (Arduino fork)
**Rust**: 1.83.0
**Target**: thumbv8m.main-none-eabihf
