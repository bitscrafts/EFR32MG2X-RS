# Gecko Bootloader Quick Reference

**Quick reference guide for Rust firmware development with Gecko Bootloader on EFR32MG24**

---

## Memory Layout Summary

```
FLASH:
  0x08000000 - 0x08005FFF  Bootloader (24 KB)
  0x08006000 - 0x0817DFFF  Application (1504 KB)
  0x0817E000 - 0x0817FFFF  NVM3 Storage (8 KB)

RAM:
  0x20000000 - 0x20000003  Bootloader Reset Region (4 B)
  0x20000004 - 0x2003FFFF  Application RAM (262140 B)
```

## Critical Vector Table Offsets

| Offset | Word | Address    | Required Value |
|--------|------|------------|----------------|
| 0x00   | 0    | 0x08006000 | Initial SP (e.g., 0x20040000) |
| 0x04   | 1    | 0x08006004 | Reset_Handler address |
| 0x34   | 13   | 0x08006034 | **ApplicationProperties pointer** |

**CRITICAL**: Word 13 MUST point to valid ApplicationProperties_t structure!

## ApplicationProperties_t Structure

### Minimal Rust Implementation

```rust
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

const APPLICATION_PROPERTIES_MAGIC: [u8; 16] = [
    0x13, 0xb7, 0x79, 0xfa, 0xc9, 0x25, 0xdd, 0xb7,
    0xad, 0xf3, 0xcf, 0xe0, 0xf1, 0xb6, 0x14, 0xb8,
];

#[used]
#[link_section = ".app_properties"]
static APP_PROPERTIES: ApplicationProperties = ApplicationProperties {
    magic: APPLICATION_PROPERTIES_MAGIC,
    struct_version: 0x0102,     // Version 1.2
    signature_type: 0,           // No signature (dev mode)
    signature_location: 0,
    app_info: ApplicationData {
        app_type: 0x10,          // MCU application
        version: 0x00010000,     // v1.0.0
        capabilities: 0,
        product_id: [0u8; 16],
    },
};
```

### Linker Script

Add to `device.x`:

```ld
SECTIONS
{
    .app_properties : ALIGN(4)
    {
        KEEP(*(.app_properties));
        KEEP(*(.app_properties.*));
    } > FLASH
}
```

## Build and Flash Workflow

### 1. Build Firmware

```bash
cargo build --release --example blink_pac --features rt
```

### 2. Generate Hex

```bash
cargo objcopy --release --example blink_pac --features rt -- -O ihex blink_pac.hex
```

### 3. Extract APP_PROPERTIES Address

```bash
ELF_FILE=$(find target/thumbv8m.main-none-eabihf/release/examples -name "blink_pac-*" -type f ! -name "*.d")
APP_PROPS_ADDR=$(arm-none-eabi-nm "$ELF_FILE" | grep APP_PROPERTIES | awk '{print "0x" $1}')
echo "APP_PROPERTIES at: $APP_PROPS_ADDR"
```

### 4. Patch Word 13

```bash
python3 patch_word13.py blink_pac.hex blink_pac_patched.hex $APP_PROPS_ADDR
```

### 5. Verify Hex File

```bash
# Check word 13 is NOT zero
sed -n '5p' blink_pac_patched.hex | cut -c18-25
# Should output: XX XX XX 08 (not 00 00 00 00)

# Check magic signature exists
grep "13B779FAC925DDB7ADF3CFE0F1B614B8" blink_pac_patched.hex
# Should find match
```

### 6. Flash to Device

```bash
OPENOCD=~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd

$OPENOCD -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program blink_pac_patched.hex verify; reset; exit"
```

## Verification Checklist

Before flashing:

- [ ] ApplicationProperties_t structure defined
- [ ] Magic = `13 B7 79 FA C9 25 DD B7 AD F3 CF E0 F1 B6 14 B8`
- [ ] Structure in `.app_properties` section
- [ ] Linker script includes section
- [ ] Hex generated successfully
- [ ] Word 13 patched (not 0x00000000)
- [ ] Magic signature present in hex
- [ ] SP and PC reasonable (words 0-1)

## Boot Validation Sequence

```
1. Bootloader reads word 13 at 0x08006034
   ├─ If NULL → FAIL (stay in bootloader)
   └─ If valid pointer → Continue

2. Read ApplicationProperties at pointer address
   ├─ Check magic signature
   ├─ Verify structure version
   └─ Validate signature (if required)

3. If all checks pass:
   ├─ Set VTOR = 0x08006000
   ├─ Load SP from word 0
   └─ Jump to Reset_Handler (word 1)
```

## Common Issues

### LED Doesn't Blink

**Cause**: Word 13 is NULL

**Debug**:
```bash
sed -n '5p' firmware.hex | cut -c18-25
```

**Expected**: NOT `00000000`

**Fix**: Run patch_word13.py script

### Bootloader Stays Active

**Cause**: Invalid magic signature

**Debug**:
```bash
grep "13B779FAC925DDB7" firmware.hex
```

**Expected**: Should find match

**Fix**: Verify ApplicationProperties_t structure definition

### Immediate Crash After Boot

**Cause**: Invalid SP or PC

**Debug**:
```bash
arm-none-eabi-objdump -s -j .vectors firmware.elf
```

**Fix**: Check linker script memory layout

## Signature Types

| Value | Type | Description |
|-------|------|-------------|
| 0x00  | NONE | No signature (development) |
| 0x01  | ECDSA_P256 | ECDSA with P-256 curve |
| 0x02  | CRC32 | CRC-32 checksum |

**Recommendation**: Use `signature_type = 0` (NONE) for development

## Application Types

| Bit | Type | Value |
|-----|------|-------|
| 0   | Zigbee | 0x01 |
| 1   | Thread | 0x02 |
| 2   | Flex | 0x04 |
| 3   | Bluetooth | 0x08 |
| 4   | MCU | 0x10 |
| 5   | Bluetooth App | 0x20 |
| 6   | Bootloader | 0x40 |
| 7   | Z-Wave | 0x80 |

**For bare-metal Rust**: Use `app_type = 0x10` (MCU)

## Clock Configuration

### State at Application Startup

- **HFRCO**: Enabled (19 MHz or 38 MHz)
- **HFXO**: Disabled
- **DPLL**: Disabled
- **LFXO**: Disabled

### Required Initialization

```rust
// Switch to HFXO and configure for higher frequency
// See full example in main documentation
```

## Security State at Handoff

- **TrustZone-M**: Configured by bootloader
- **Application**: Runs in Non-Secure state
- **SMU**: Peripheral security attributes set
- **SE**: Accessible via mailbox interface

## Automated Build Script

Save as `build.sh`:

```bash
#!/bin/bash
set -e

EXAMPLE="$1"
if [ -z "$EXAMPLE" ]; then
    echo "Usage: $0 <example_name>"
    exit 1
fi

echo "Building $EXAMPLE..."
cargo build --release --example $EXAMPLE --features rt

echo "Generating hex..."
cargo objcopy --release --example $EXAMPLE --features rt -- -O ihex ${EXAMPLE}.hex

echo "Extracting APP_PROPERTIES address..."
ELF_FILE=$(find target/thumbv8m.main-none-eabihf/release/examples -name "${EXAMPLE}-*" -type f ! -name "*.d" | head -1)
APP_PROPS_ADDR=$(arm-none-eabi-nm "$ELF_FILE" | grep APP_PROPERTIES | awk '{print "0x" $1}')

echo "APP_PROPERTIES at: $APP_PROPS_ADDR"
echo "Patching word 13..."
python3 patch_word13.py ${EXAMPLE}.hex ${EXAMPLE}_patched.hex $APP_PROPS_ADDR

echo "Verifying..."
WORD13=$(sed -n '5p' ${EXAMPLE}_patched.hex | cut -c18-25)
echo "Word 13: $WORD13"

if grep -q "13B779FAC925DDB7" ${EXAMPLE}_patched.hex; then
    echo "✓ Magic signature found"
else
    echo "✗ Magic signature NOT found"
    exit 1
fi

echo "✓ Ready to flash: ${EXAMPLE}_patched.hex"
```

Usage:
```bash
chmod +x build.sh
./build.sh blink_pac
```

## Flash Script

Save as `flash.sh`:

```bash
#!/bin/bash
set -e

HEX_FILE="$1"
if [ -z "$HEX_FILE" ]; then
    echo "Usage: $0 <hex_file>"
    exit 1
fi

OPENOCD=~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd

echo "Flashing $HEX_FILE..."
$OPENOCD -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program $HEX_FILE verify; reset; exit"
```

Usage:
```bash
chmod +x flash.sh
./flash.sh blink_pac_patched.hex
```

## Complete Workflow

```bash
# 1. Build and patch
./build.sh blink_pac

# 2. Flash to device
./flash.sh blink_pac_patched.hex

# 3. Observe LED blinking
# Device should boot and run application
```

---

**For complete details, see**: [GECKO_BOOTLOADER_RESEARCH.md](GECKO_BOOTLOADER_RESEARCH.md)
