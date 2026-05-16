# Working Arduino Examples

This directory contains verified working Arduino examples for XIAO MG24 Sense.

These examples use the Gecko bootloader and have been confirmed working on hardware.

## Files

### arduino_blink.hex
- **Description**: Simple LED blink example from Arduino IDE
- **Size**: ~2.5 MB (includes Gecko bootloader)
- **Behavior**: LED blinks at 1 Hz
- **Status**: Verified working

### microphone_volume_with_bootloader.hex
- **Description**: Microphone volume indicator example with Gecko bootloader
- **Size**: ~2.5 MB (includes Gecko bootloader)
- **Behavior**: LED responds to microphone input
- **Status**: Verified working (tested 2026-01-25)

### microphone_volume_arduinocli.hex
- **Description**: Microphone volume indicator compiled with Arduino CLI
- **Size**: ~2.5 MB (includes Gecko bootloader)
- **Behavior**: LED responds to microphone input
- **Status**: Available (compiled version)

## Flashing

To flash any of these examples:

```bash
cd /Users/mvcorrea/Private/20250626_rust_projects/2025-10-29/EFR32MG24/efr32mg24-bootloader

# Flash volume example
../openocd/bin/openocd -s ../openocd/scripts \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program arduino-working-examples/microphone_volume_with_bootloader.hex verify; reset; exit"

# Flash blink example
../openocd/bin/openocd -s ../openocd/scripts \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program arduino-working-examples/arduino_blink.hex verify; reset; exit"
```

## Purpose

These examples serve as:
1. Hardware verification (confirm XIAO MG24 is working correctly)
2. Reference implementation (compare with custom Rust bootloader)
3. Recovery fallback (restore working firmware if custom bootloader fails)
4. Baseline comparison (Gecko bootloader vs custom Rust bootloader)

## Notes

- All examples include Silicon Labs Gecko bootloader (~24 KB)
- Flash address: 0x08000000 (bootloader + application combined)
- Uses Gecko bootloader's boot sequence
- Requires CMSIS-DAP interface (SAMD11 on XIAO MG24)

## History

- 2026-01-25: Copied from efr32mg24-pac directory
- 2026-01-25: Verified microphone_volume_with_bootloader.hex working on hardware
