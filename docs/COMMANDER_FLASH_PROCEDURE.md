# Simplicity Commander Flash Procedure for XIAO MG24

**Date**: 2026-01-03
**Device**: XIAO MG24 Sense
**Connection**: CMSIS-DAP (onboard SAMD11 USB bridge)

---

## Key Command Format

```bash
commander flash <firmware.hex> --identifybyserialport <serial_port>
```

### Your Device

- **Serial Port**: `/dev/cu.usbmodem24D15EE43`
- **Commander**: `~/.local/bin/commander` (wrapper script)

---

## Flash with Bootloader (SAFE)

```bash
commander flash firmware.with_bootloader.hex \
  --identifybyserialport /dev/cu.usbmodem24D15EE43
```

**When to use**: First flash, recovery, production

---

## Flash Application Only (FAST)

```bash
commander flash firmware.hex \
  --address 0x00004000 \
  --identifybyserialport /dev/cu.usbmodem24D15EE43
```

**When to use**: Development iteration (bootloader already present)

**Important**: `--address 0x00004000` places app after bootloader

---

## Test Flash (Verify Connection)

```bash
cd /Users/mvcorrea/Private/20250626_rust_projects/2025-10-29/EFR32MG24/xiao_mg24_flash_erase_macos/SimplicityCommander-Mac/Commander-cli.app/Contents/MacOS/

commander flash MicrophoneVolume.ino.with_bootloader.hex \
  --identifybyserialport /dev/cu.usbmodem24D15EE43
```

This should:
1. Connect via CMSIS-DAP
2. Flash the firmware
3. Device should reboot and run

---

## Rust Firmware Flash Procedure

### Step 1: Build

```bash
cd /path/to/xiao-mg24-bsp
cargo build --release --example 00_minimal_blink
```

### Step 2: Generate HEX

```bash
cargo objcopy --release --example 00_minimal_blink -- -O ihex 00_minimal_blink.hex
```

### Step 3: Flash

```bash
commander flash 00_minimal_blink.hex \
  --address 0x00004000 \
  --identifybyserialport /dev/cu.usbmodem24D15EE43
```

---

## Quick Reference

| Scenario | Command |
|----------|---------|
| **First flash / Recovery** | `commander flash fw.with_bootloader.hex --identifybyserialport /dev/cu.usbmodem...` |
| **Development iteration** | `commander flash fw.hex --address 0x00004000 --identifybyserialport /dev/cu.usbmodem...` |
| **Device info** | `commander device info --identifybyserialport /dev/cu.usbmodem...` |

---

## Expected Output (Successful Flash)

```
Parsing file MicrophoneVolume.ino.with_bootloader.hex...
Identifying device using serial port /dev/cu.usbmodem24D15EE43...
Connected to adapter 2886:0062:24D15EE4
Chip ID: 0x19 (EFR32MG24B220F1536IM48)
Erasing flash...
Writing flash...
Verifying flash...
DONE
```

---

## Troubleshooting

**Error: Cannot identify device**
- Check serial port: `ls /dev/cu.usb*`
- Try unplugging and replugging USB
- Check device manager

**Error: Flash verification failed**
- Bootloader might be corrupted
- Use recovery: `commander device recover --device EFR32MG24`

---

Last updated: 2026-01-03
