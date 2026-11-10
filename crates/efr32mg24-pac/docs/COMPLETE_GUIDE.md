# Complete Guide: Rust on EFR32MG24 with Gecko Bootloader

**Hardware**: Seeed Studio XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
**Achievement**: First confirmed bare-metal Rust boot on EFR32MG24
**Date**: January 10, 2026

---

## Table of Contents

1. [Silicon Labs Engineer Guidance](#silicon-labs-engineer-guidance)
2. [ApplicationProperties Complete Specification](#applicationproperties-complete-specification)
3. [Tools and Verification](#tools-and-verification)
4. [Complete Working Example](#complete-working-example)
5. [Build and Flash Workflow](#build-and-flash-workflow)
6. [Troubleshooting](#troubleshooting)

---

## Silicon Labs Engineer Guidance

### Official Forum Response

A Silicon Labs engineer provided critical insights on Gecko Bootloader integration:

#### Key Requirements

> "the bootloader requires an application to have a **valid** Applications Properties structure or it will not jump to the application's RESET vector after device reset"

#### Word 13 Mechanism

> "In order to simplify finding the Application Properties, we use an unused exception vector (13) to point to the application properties"

This means:
- Vector table word 13 (offset 0x34) must point to ApplicationProperties
- Bootloader reads word 13 to find the structure
- If invalid or missing, bootloader won't jump to application

#### Official Source Code

Two critical files in Gecko SDK:

1. **[application_properties.h](https://github.com/SiliconLabs/gecko_sdk/blob/gsdk_4.5/platform/bootloader/api/application_properties.h)**
   - Complete structure definition
   - Magic value constants
   - Field documentation

2. **[application_properties.c](https://github.com/SiliconLabs/gecko_sdk/blob/gsdk_4.5/platform/bootloader/app_properties/app_properties.c)**
   - Example initialization
   - Compiler attributes
   - Section placement

#### Validation Tool

Use Simplicity Commander to analyze firmware:

```bash
$ commander util appinfo firmware.hex
Found application properties in image.
Application properties info:
Application properties location : 0x08006228
Signature location              : Not set (0xffffffff)
App type                        : Unknown application type 00000022
App version                     : 0x00000001
```

This shows exactly what the bootloader sees!

#### Vector Table Discovery

The engineer provided a Python script to find vector tables:

```bash
$ python3 start-page.py firmware.hex 0x08006228
vector table located at 0x08006000 pointing to application properties at 0x08006228
```

This verifies Word 13 points to ApplicationProperties correctly.

---

## ApplicationProperties Complete Specification

### Official Structure (from Silicon Labs)

```c
typedef struct {
  uint8_t magic[16];                     // MUST be first field!
  uint32_t structVersion;                // Version of this struct
  uint32_t signatureType;                // Type of signature (0 = none)
  uint32_t signatureLocation;            // Signature pointer
  ApplicationData_t app;                 // Application metadata
  ApplicationCertificate_t *cert;        // Certificate pointer
  uint8_t *longTokenSectionAddress;      // Token section pointer
  const uint8_t decryptKey[16];          // Decryption key
} ApplicationProperties_t;

typedef struct ApplicationData {
  uint32_t type;                         // Application type
  uint32_t version;                      // Application version
  uint32_t capabilities;                 // Capability flags
  uint8_t productId[16];                 // Product identifier
} ApplicationData_t;
```

**Total Size**: 80 bytes (official structure)

### Arduino Extended Format

Arduino adds an **8-byte header** before the official structure:

```
Offset  Contents
0x00    [02 01 02 02 02 03 02 04]  ← 8-byte header
0x08    [13 B7 79 FA ... ]         ← Magic signature (Word 13 points HERE!)
0x18    struct_version, etc.
...     rest of structure
```

**Total Size with Header**: 88 bytes

**Critical**: Word 13 must point to the **magic** (offset 0x08), NOT the header (offset 0x00)!

### Rust Implementation

```rust
#[repr(C)]
pub struct ApplicationProperties {
    header: [u8; 8],                // Arduino-specific header
    magic: [u8; 16],                // Magic signature
    struct_version: u32,
    signature_type: u32,
    signature_location: u32,
    app_info: ApplicationData,
    cert: u32,                       // Pointer (NULL = 0)
    long_token_section_address: u32, // Pointer (NULL = 0)
    decrypt_key: [u8; 16],           // All zeros
}

#[repr(C)]
pub struct ApplicationData {
    app_type: u32,
    version: u32,
    capabilities: u32,
    product_id: [u8; 16],
}

const APPLICATION_PROPERTIES_MAGIC: [u8; 16] = [
    0x13, 0xb7, 0x79, 0xfa, 0xc9, 0x25, 0xdd, 0xb7,
    0xad, 0xf3, 0xcf, 0xe0, 0xf1, 0xb6, 0x14, 0xb8,
];

#[used]
#[link_section = ".app_properties"]
static APP_PROPERTIES: ApplicationProperties = ApplicationProperties {
    header: [0x02, 0x01, 0x02, 0x02, 0x02, 0x03, 0x02, 0x04],
    magic: APPLICATION_PROPERTIES_MAGIC,
    struct_version: 0x0201,
    signature_type: 0,
    signature_location: 0xFFFFFFFF,
    app_info: ApplicationData {
        app_type: 0x22,              // Must match Arduino (34 decimal)
        version: 0x00000001,         // v0.0.1 for rollback protection
        capabilities: 0,
        product_id: [0u8; 16],
    },
    cert: 0,
    long_token_section_address: 0,
    decrypt_key: [0u8; 16],
};
```

### Field Requirements

| Field | Value | Notes |
|-------|-------|-------|
| `magic` | Exact 16-byte sequence | Must match Silicon Labs constant |
| `struct_version` | 0x0201 | Version 2.1 |
| `signature_type` | 0 | No signature |
| `signature_location` | 0xFFFFFFFF | No signature |
| `app_type` | **0x22** | **Must match Arduino!** |
| `version` | 0x00000001 | For rollback protection |
| `cert` | 0 (NULL) | No certificate |
| `long_token_section_address` | 0 (NULL) | No long tokens |
| `decrypt_key` | All zeros | No encryption |

**Critical**: `app_type = 0x22` (34 decimal) matches Arduino and is required!

---

## Tools and Verification

### 1. Simplicity Commander

**Download**: https://www.silabs.com/developers/simplicity-studio/simplicity-commander

**Analyze ApplicationProperties**:
```bash
$ commander util appinfo firmware.hex
```

**Flash Firmware**:
```bash
$ commander flash firmware.hex --device EFR32MG24
```

**Get Device Info**:
```bash
$ commander device info --device efr32mg24
Part Number    : EFR32MG24B210F1536IM48
Die Revision   : A1
Flash Size     : 1536 kB
SRAM Size      : 256 kB
```

### 2. Vector Table Finder (start-page.py)

Find vector tables pointing to ApplicationProperties:

```bash
$ python3 start-page.py firmware.hex 0x08006228
vector table located at 0x08006000 pointing to application properties at 0x08006228
```

This confirms Word 13 is correctly set!

### 3. Auto-Patch Script

Since linker places `.app_properties` at variable addresses, auto-detection is required:

```python
def find_app_properties(hex_file):
    """Find ApplicationProperties by searching for magic signature"""
    magic_hex = "13B779FAC925DDB7"
    with open(hex_file, 'r') as f:
        for line in f:
            if magic_hex in line.upper():
                addr_offset = int(line[3:7], 16)
                data_hex = line[9:-3]
                magic_pos = data_hex.upper().find(magic_hex)
                if magic_pos >= 0:
                    byte_offset = magic_pos // 2
                    magic_addr = 0x08000000 + addr_offset + byte_offset
                    struct_addr = magic_addr - 8  # Header is 8 bytes before magic
                    return struct_addr, magic_addr
    return None, None

# Usage:
struct_addr, magic_addr = find_app_properties('firmware.hex')
print(f"Structure at 0x{struct_addr:08X}, Magic at 0x{magic_addr:08X}")

# Patch Word 13
subprocess.run(['python3', 'patch_word13.py', 'firmware.hex',
                'firmware_patched.hex', f'0x{struct_addr:08X}'])
```

---

## Complete Working Example

### Minimal Rust Firmware

```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as _;
use panic_halt as _;

// ApplicationProperties structure (88 bytes with Arduino header)
#[repr(C)]
pub struct ApplicationProperties {
    header: [u8; 8],
    magic: [u8; 16],
    struct_version: u32,
    signature_type: u32,
    signature_location: u32,
    app_info: ApplicationData,
    cert: u32,
    long_token_section_address: u32,
    decrypt_key: [u8; 16],
}

#[repr(C)]
pub struct ApplicationData {
    app_type: u32,
    version: u32,
    capabilities: u32,
    product_id: [u8; 16],
}

const APPLICATION_PROPERTIES_MAGIC: [u8; 16] = [
    0x13, 0xb7, 0x79, 0xfa, 0xc9, 0x25, 0xdd, 0xb7,
    0xad, 0xf3, 0xcf, 0xe0, 0xf1, 0xb6, 0x14, 0xb8,
];

#[used]
#[link_section = ".app_properties"]
static APP_PROPERTIES: ApplicationProperties = ApplicationProperties {
    header: [0x02, 0x01, 0x02, 0x02, 0x02, 0x03, 0x02, 0x04],
    magic: APPLICATION_PROPERTIES_MAGIC,
    struct_version: 0x0201,
    signature_type: 0,
    signature_location: 0xFFFFFFFF,
    app_info: ApplicationData {
        app_type: 0x22,
        version: 0x00000001,
        capabilities: 0,
        product_id: [0u8; 16],
    },
    cert: 0,
    long_token_section_address: 0,
    decrypt_key: [0u8; 16],
};

#[entry]
fn main() -> ! {
    // Your code here
    loop {
        cortex_m::asm::nop();
    }
}
```

### Memory Layout

```
┌───────────────────────────────────────┐
│ Gecko Bootloader                      │ 0x08000000 - 0x08005FFF
│ - Validates ApplicationProperties     │
│ - Checks Word 13 pointer             │
│ - Jumps to reset handler             │
├───────────────────────────────────────┤
│ Rust Application                      │ 0x08006000 - 0x0817FFFF
│                                       │
│ Vector Table (0x08006000):           │
│   Word 0:  SP  = 0x20001208          │
│   Word 1:  PC  = 0x08006159 (Thumb)  │
│   Word 13: Ptr = 0x080061E8 ─────┐   │
│                                   │   │
│ ApplicationProperties:            │   │
│   0x080061E0: Header (8 bytes)    │   │
│   0x080061E8: Magic  (16 bytes) ←─┘   │
│   0x080061F8: Metadata            │   │
│   0x08006228: (88 bytes total)    │   │
│                                   │   │
│ Application Code:                 │   │
│   0x08006158: Reset handler       │   │
└───────────────────────────────────────┘
```

---

## Build and Flash Workflow

### Step 1: Build

```bash
cargo build --example minimal_test --features rt --release \
            --target thumbv8m.main-none-eabihf
```

### Step 2: Convert to HEX

```bash
arm-none-eabi-objcopy -O ihex \
    target/thumbv8m.main-none-eabihf/release/examples/minimal_test \
    minimal_test.hex
```

### Step 3: Auto-Detect and Patch Word 13

```bash
python3 << 'EOF'
import subprocess

def find_app_properties(hex_file):
    magic_hex = "13B779FAC925DDB7"
    with open(hex_file, 'r') as f:
        for line in f:
            if magic_hex in line.upper():
                addr_offset = int(line[3:7], 16)
                data_hex = line[9:-3]
                magic_pos = data_hex.upper().find(magic_hex)
                if magic_pos >= 0:
                    byte_offset = magic_pos // 2
                    magic_addr = 0x08000000 + addr_offset + byte_offset
                    return magic_addr - 8, magic_addr
    return None, None

struct_addr, magic_addr = find_app_properties('minimal_test.hex')
print(f"Found ApplicationProperties at 0x{struct_addr:08X}")
print(f"Magic signature at 0x{magic_addr:08X}")

subprocess.run([
    'python3', 'patch_word13.py',
    'minimal_test.hex',
    'minimal_test_patched.hex',
    f'0x{struct_addr:08X}'
])
EOF
```

### Step 4: Verify with Simplicity Commander

```bash
commander util appinfo minimal_test_patched.hex
```

Expected output:
```
Found application properties in image.
Application properties location : 0x080061e8
App type                        : Unknown application type 00000022
App version                     : 0x00000001
```

✅ If this shows, bootloader will accept it!

### Step 5: Verify with start-page.py

```bash
python3 start-page.py minimal_test_patched.hex 0x080061e8
```

Expected output:
```
vector table located at 0x08006000 pointing to application properties at 0x080061E8
```

✅ Confirms Word 13 correctly points to magic!

### Step 6: Flash with OpenOCD

```bash
#!/bin/bash
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init" \
  -c "program minimal_test_patched.hex verify" \
  -c "reset run" \
  -c "exit"
```

### Step 7: Power Cycle and Verify

**Power cycle the board** (unplug/replug USB).

LED behavior:
- **Red for ~10 seconds**: SAMD11 bootloader
- **Then**: Your application runs!

### Step 8: Check PC Location

```bash
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
        -c "init" -c "halt" -c "reg pc" -c "exit"
```

Expected: `pc (/32): 0x080061xx` (application code)
NOT: `pc (/32): 0x08000170` (bootloader stuck)

✅ **If PC is in application range (0x08006xxx), SUCCESS!**

---

## Troubleshooting

### Issue: Bootloader Stuck (PC = 0x08000170)

**Symptoms**:
- PC remains at 0x08000170 after boot
- Application never runs

**Causes**:
1. Word 13 not pointing to ApplicationProperties
2. ApplicationProperties structure invalid
3. Magic signature incorrect

**Debug**:
```bash
# Check what Commander sees
$ commander util appinfo firmware.hex

# If it says "Found application properties" ✅
# If it says "No application properties found" ❌

# Check Word 13 manually
$ python3 start-page.py firmware.hex 0x08006xxx

# Should show vector table at 0x08006000
```

**Fix**:
- Re-run auto-patch script
- Verify magic signature exactly matches
- Check structure size (should be 88 bytes with header)

### Issue: Commander Says "No application properties found"

**Cause**: Structure not in hex file or magic incorrect

**Debug**:
```bash
# Search for magic in hex file
$ grep "13B779FAC925DDB7" firmware.hex

# Should find it. If not, check:
# 1. Is APP_PROPERTIES marked #[used]?
# 2. Is it in section .app_properties?
# 3. Is section included in linker script?
```

**Fix**:
- Add `#[used]` attribute
- Verify linker script includes `.app_properties` section
- Check that structure is not optimized out

### Issue: App Runs But GPIO Doesn't Work

**Symptoms**:
- PC in application code ✅
- LED doesn't blink ❌

**Cause**: Clock not enabled (current known issue)

**Debug**:
```bash
# Check if GPIO clock enabled
$ openocd -c "init" -c "halt" -c "mdw 0x50008308 1" -c "exit"
0x50008308: 00000000  # ← GPIO clock disabled!

# Check GPIO configuration
$ openocd -c "mdw 0x50000004 1"
0x50000004: 40000000  # ← PA7 configured ✅
```

**Workaround**:
- Use Arduino HAL functions (clock handled automatically)
- Investigate CMU lock mechanism
- Check bootloader clock initialization

---

## Success Criteria

### ✅ Bootloader Integration Complete

- [x] ApplicationProperties structure validates with Commander
- [x] Word 13 points to magic signature
- [x] Bootloader jumps to Rust application
- [x] PC confirmed in application code (NOT bootloader)
- [x] Auto-detection and patching working

### ⏳ Hardware Control (In Progress)

- [ ] GPIO clock enable working
- [ ] LED blink functional
- [ ] Clock initialization understood

---

## Resources

### Silicon Labs Official

- [Gecko SDK on GitHub](https://github.com/SiliconLabs/gecko_sdk)
- [Simplicity Commander](https://www.silabs.com/developers/simplicity-studio/simplicity-commander)
- [EFR32MG24 Reference Manual](https://www.silabs.com/documents/public/reference-manuals/efr32xg24-rm.pdf)
- [Gecko Bootloader User Guide](https://www.silabs.com/documents/public/user-guides/ug266-gecko-bootloader-user-guide.pdf)

### API Documentation

- [ApplicationProperties_t API](https://docs.silabs.com/mcu-bootloader/latest/gecko-bootloader-api/application-properties-t)
- [Bootloader API Reference](https://docs.silabs.com/mcu-bootloader/latest/gecko-bootloader-api/)

### Source Code

- [application_properties.h](https://github.com/SiliconLabs/gecko_sdk/blob/gsdk_4.5/platform/bootloader/api/application_properties.h)
- [application_properties.c](https://github.com/SiliconLabs/gecko_sdk/blob/gsdk_4.5/platform/bootloader/app_properties/app_properties.c)

---

## Conclusion

This guide documents the **first successful Rust bare-metal boot** on EFR32MG24, based on official Silicon Labs engineer guidance and comprehensive testing.

**Key Achievement**: Gecko Bootloader integration is complete and working!

**Remaining Work**: GPIO clock initialization (separate issue from bootloader)

**Impact**: Establishes EFR32MG24 as a viable Rust embedded target! 🎉

---

**Created**: January 10, 2026
**Status**: Bootloader ✅ COMPLETE | GPIO ⏳ IN PROGRESS
**Last Updated**: January 10, 2026
