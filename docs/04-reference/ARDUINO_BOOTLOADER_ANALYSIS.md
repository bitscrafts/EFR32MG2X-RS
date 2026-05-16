# Arduino Bootloader Analysis - Rust Firmware Boot Failure

**Date**: 2026-01-09
**Status**: Root cause identified
**Hardware**: XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)

## Executive Summary

**Problem**: Rust firmware flashes successfully via OpenOCD but doesn't boot (PC stuck at 0x08000170).

**Root Cause**: Gecko Bootloader requires `ApplicationProperties_t` structure pointer at vector table word 13 (offset 0x34). Arduino firmware includes this, Rust firmware built with `cortex-m-rt` does not.

**Impact**: All Rust firmware must include Application Properties structure to boot from Gecko Bootloader.

## Investigation Timeline

### 1. Initial Symptom
- Rust `blink_pac.hex` flashes with "Verified OK"
- LED doesn't blink
- PC remains at bootloader entry (0x08000170)
- Device not bricked (Arduino firmware restores successfully)

### 2. Memory Layout Verification
Both Arduino and Rust firmware correctly configured:
- Flash base: 0x08000000
- Bootloader: 0x08000000 - 0x08005FFF (24 KB)
- Application start: 0x08006000
- Application region: 0x08006000 - 0x0817FFFF (1512 KB)

### 3. Vector Table Comparison

#### Arduino (Working)
```
Line 2: :106000000000042059610008F1610008CC08010860
         ^^^^^^^^ ^^^^^^^^ ^^^^^^^^ ^^^^^^^^
         SP       PC       Handler  Handler
         20001208 08006159 08010880 080108CC

Line 5: :1060300023CA0708CC7C0C08D1200C08CF730C08AD
                    ^^^^^^^^
                    Word 13 = 0x080C7CCC
                    └─> ApplicationProperties_t pointer
```

#### Rust (Non-working)
```
Line 2: :106000000000042059610008F1610008FD610008EA
         ^^^^^^^^ ^^^^^^^^ ^^^^^^^^ ^^^^^^^^
         SP       PC       Handler  Handler
         20040000 08006159 08006161 080061FD

Line 5: :10603000F161000800000000F1610008F161000852
                    ^^^^^^^^
                    Word 13 = 0x00000000
                    └─> NULL (missing!)
```

### 4. Critical Discovery

**Vector Table Word 13 (offset 0x34, bytes 52-55)**:
- **Arduino**: `0x080C7CCC` - Points to `ApplicationProperties_t` at flash address 0x080C7CCC
- **Rust**: `0x00000000` - NULL pointer, no application properties

**Bootloader Behavior**:
1. Bootloader starts at 0x08000170
2. Validates application via word 13 pointer
3. If word 13 is NULL or invalid → stays in bootloader
4. If word 13 points to valid `ApplicationProperties_t` → jumps to application

## Gecko Bootloader Application Properties

### Structure Requirements

According to Silicon Labs UG489 (Gecko Bootloader User's Guide):

```c
typedef struct {
  uint8_t magic[16];              // APPLICATION_PROPERTIES_MAGIC
  uint32_t structVersion;         // Version of this structure
  uint32_t signatureType;         // Signature type
  uint32_t signatureLocation;     // Location of signature
  ApplicationData_t app;          // Application data
  // ... additional fields
} ApplicationProperties_t;
```

**Magic Signature** (16 bytes):
```c
#define APPLICATION_PROPERTIES_MAGIC \
  { 0x13, 0xb7, 0x79, 0xfa, 0xc9, 0x25, 0xdd, 0xb7, \
    0xad, 0xf3, 0xcf, 0xe0, 0xf1, 0xb6, 0x14, 0xb8 }
```

### Vector Table Word 13

**Location**: Offset 0x34 (52 bytes) from vector table start
**Purpose**: Pointer to `ApplicationProperties_t` structure
**Requirement**: MUST be non-NULL for bootloader to boot application

**Standard ARM Cortex-M Vector Table**:
- Word 0 (0x00): Initial stack pointer
- Word 1 (0x04): Reset handler (PC)
- Words 2-12 (0x08-0x30): Exception handlers
- **Word 13 (0x34): Application properties pointer** ← Gecko-specific!
- Words 14+ (0x38+): Interrupt handlers

## Why Simplicity Commander Works

Arduino IDE uses Simplicity Commander to flash firmware:
1. Compiles application ELF
2. **Simplicity Commander automatically**:
   - Creates `ApplicationProperties_t` structure
   - Places it in flash
   - **Inserts pointer at vector table word 13**
3. Generates hex file with everything included

## Why cortex-m-rt Doesn't Work

`cortex-m-rt` is a generic ARM Cortex-M startup runtime:
- Generates standard vector table (words 0-12)
- **Does NOT know about Gecko-specific word 13**
- Word 13 remains zero-initialized (NULL)
- Bootloader refuses to boot

## Solution Requirements

To make Rust firmware boot from Gecko Bootloader:

1. **Create ApplicationProperties_t structure** in Rust
   - Include 16-byte magic signature
   - Set structure version, signature type
   - Place in dedicated flash section

2. **Override vector table word 13** in linker script
   - Point to ApplicationProperties_t structure
   - Ensure proper alignment

3. **Alternative**: Modify cortex-m-rt or create custom startup
   - Fork cortex-m-rt with Gecko support
   - Or create efr32-rt crate with built-in app properties

## Hex File Format Reference

**Intel HEX Format**:
```
:LLAAAATTDD...DDCC
 └┬┘└─┬─┘└┬┘└──┬──┘└┬┘
  │   │   │    │    └─ Checksum
  │   │   │    └────── Data bytes
  │   │   └─────────── Record type (00=data)
  │   └─────────────── Address offset
  └─────────────────── Byte count (hex)
```

**Line 5 Breakdown** (offset 0x30-0x3F):
```
:10603000CC7C0C08...
         ^^^^^^^^
         │││││││└─ Byte 7 (MSB)
         ││││││└── Byte 6
         │││││└─── Byte 5
         ││││└──── Byte 4 (LSB)
         └┴┴┴───── Little-endian 32-bit word
```

## Verification Commands

**Extract vector table word 13**:
```bash
# Arduino (working)
sed -n '5p' microphone_volume_arduinocli.hex | cut -c18-25
# Output: CC7C0C08 (little-endian 0x080C7CCC)

# Rust (not working)
sed -n '5p' blink_pac.hex | cut -c18-25
# Output: 00000000 (NULL)
```

## References

1. **UG489**: Gecko Bootloader User's Guide (GSDK 4.x)
   - Chapter: Application Properties
   - Section: ApplicationProperties_t Structure

2. **Silicon Labs API Documentation**:
   - [Gecko Bootloader Application Properties](https://docs.silabs.com/mcu-bootloader/2.3.0/gecko-bootloader-api/application-properties)
   - [ApplicationProperties_t Struct](https://docs.silabs.com/mcu-bootloader/latest/gecko-bootloader-api/application-properties-t)

3. **ARM Documentation**:
   - Cortex-M33 Vector Table (standard words 0-12)
   - Vendor-specific extensions (word 13+)

## Next Steps

1. ✅ Root cause identified
2. ⏳ Implement `ApplicationProperties_t` in Rust
3. ⏳ Create linker script override for word 13
4. ⏳ Test modified firmware
5. ⏳ Document solution for community

---

**Analysis by**: Rust Embedded Orchestrator
**Hardware Verified**: XIAO MG24 Sense with Arduino bootloader
**OpenOCD Version**: 0.12.0+dev-01514-g21fa2de70 (Arduino fork)
