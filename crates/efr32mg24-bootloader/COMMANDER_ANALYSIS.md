# Simplicity Commander Binary Analysis

**Tool**: Simplicity Commander CLI
**Purpose**: Analyze, flash, and debug EFR32 binaries

---

## Installation

Simplicity Commander comes with Simplicity Studio 5:

**macOS**:
```bash
# After installing Simplicity Studio 5
/Applications/Simplicity\ Studio.app/Contents/Eclipse/developer/adapter_packs/commander/commander
```

**Or standalone download**:
https://www.silabs.com/developers/mcu-programming-options

---

## Common Commands

### 1. Device Information

```bash
commander device info

# Output shows:
# - Part number
# - Unique ID
# - Flash size
# - Page lock status
# - Debug lock status
```

### 2. Read Flash

```bash
# Read entire flash
commander readmem --range 0x08000000:0x0817FFFF -o flash_dump.bin

# Read specific region
commander readmem --range 0x08000000:0x08003FFF -o bootloader.bin
commander readmem --range 0x08004000:0x08004FFF -o app.bin
```

### 3. Analyze Binary

```bash
# Convert binary to hex (with analysis)
commander convert bootloader.bin --address 0x08000000 -o bootloader_analyzed.hex

# Show hex file contents
commander convert bootloader.hex --device EFR32MG24B220F1536 --info
```

### 4. Flash Binary

```bash
# Flash hex file
commander flash bootloader.hex

# Flash with verification
commander flash --verify bootloader.hex

# Flash multiple files
commander flash bootloader.hex app.hex
```

### 5. Erase Flash

```bash
# Mass erase
commander device masserase

# Page erase
commander flash --erase-range 0x08000000:+0x4000  # Erase 16 KB
```

### 6. Lock/Unlock Device

```bash
# Read lock status
commander device lock --status

# Unlock debug
commander device lock --debug disable
```

---

## Analyzing Our Binaries

### Bootloader Analysis

```bash
# Check bootloader size and structure
commander convert bootloader.hex --info

# Expected output:
# Address range: 0x08000000 - 0x08000318 (792 bytes)
# Segments: 1
# Format: Intel HEX
```

### Application Analysis

```bash
# Check application structure
commander convert test-app.hex --info

# Expected output:
# Address range: 0x08004000 - 0x080048C8 (2248 bytes)
# Segments: 1
# Format: Intel HEX
```

### Combined Flash Analysis

```bash
# Read current flash and save
commander readmem --range 0x08000000:0x0817FFFF -o current_flash.bin

# Convert to hex for inspection
commander convert current_flash.bin --address 0x08000000 -o current_flash.hex

# Check what's in flash
hexdump -C current_flash.bin | head -100
```

---

## Debugging with Commander

### 1. Check Reset Cause

```bash
# Read reset cause register
commander util readmem --range 0x400E5030:+4
```

### 2. Check Watchdog Status

```bash
# Read WDOG0 enable register
commander util readmem --range 0x4A018000:+4

# Read WDOG1 enable register
commander util readmem --range 0x4A01C000:+4
```

### 3. Check Vector Tables

```bash
# Read bootloader vector table
commander util readmem --range 0x08000000:+64

# Read application vector table
commander util readmem --range 0x08004000:+64
```

### 4. Check VTOR Register

```bash
# Read VTOR (Vector Table Offset Register)
commander util readmem --range 0xE000ED08:+4

# Should be 0x08004000 if bootloader jumped to app
```

---

## Comparing Binaries

### Compare with Working Arduino Binary

```bash
# Read working Arduino binary from flash
commander flash --erase
commander flash arduino-working-examples/microphone_volume_with_bootloader.hex
commander readmem --range 0x08000000:0x0817FFFF -o arduino_flash.bin

# Read our custom bootloader flash
commander flash --erase
commander flash bootloader.hex test-app.hex
commander readmem --range 0x08000000:0x0817FFFF -o custom_flash.bin

# Compare
diff <(hexdump -C arduino_flash.bin) <(hexdump -C custom_flash.bin) | head -100
```

---

## Advanced: Application Properties

Silicon Labs bootloaders expect an ApplicationProperties structure:

```bash
# Check if app has properties (usually at end of flash)
commander util readmem --range 0x080048C8:+256

# Look for magic bytes: 0x13579BDF
```

---

## Troubleshooting

### Command Not Found

```bash
# Add to PATH
export PATH="/Applications/Simplicity Studio.app/Contents/Eclipse/developer/adapter_packs/commander:$PATH"

# Or create alias
alias commander="/Applications/Simplicity Studio.app/Contents/Eclipse/developer/adapter_packs/commander/commander"
```

### Device Not Found

```bash
# List devices
commander adapter probe

# Select specific adapter
commander adapter select J-Link:123456

# Or use CMSIS-DAP
commander adapter select CMSIS-DAP:24D15EE4
```

### Permission Denied

```bash
# macOS: Grant terminal permissions in System Preferences > Privacy
# Or run with sudo (not recommended)
```

---

## Quick Reference

| Task | Command |
|------|---------|
| Device info | `commander device info` |
| Flash hex | `commander flash file.hex` |
| Read flash | `commander readmem --range 0x0:+0x180000 -o dump.bin` |
| Erase flash | `commander device masserase` |
| Convert format | `commander convert input.bin --address 0x0 -o output.hex` |
| Read memory | `commander util readmem --range 0xADDRESS:+SIZE` |
| Debug unlock | `commander device lock --debug disable` |

---

## Integration with Our Build

Add to `build-and-flash.sh`:

```bash
# After flashing, verify with commander
echo "Verifying with Simplicity Commander..."
commander readmem --range 0x08000000:+792 -o bootloader_verify.bin
commander readmem --range 0x08004000:+2248 -o app_verify.bin

# Compare with built binaries
cmp bootloader.bin bootloader_verify.bin && echo "✓ Bootloader matches"
cmp app.bin app_verify.bin && echo "✓ Application matches"
```

---

## References

- [Simplicity Commander User Guide](https://www.silabs.com/documents/public/user-guides/ug162-simplicity-commander-reference-guide.pdf)
- [AN0015.2: EFR32 Series 2 Watchdog](https://www.silabs.com/documents/public/application-notes/an0015.2-efr32-series-2-watchdog.pdf)

---

**Note**: Commander is more reliable than OpenOCD for EFR32 devices as it's Silicon Labs' official tool.
