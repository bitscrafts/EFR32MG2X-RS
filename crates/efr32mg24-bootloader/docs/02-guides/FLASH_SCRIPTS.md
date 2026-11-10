# Flash Scripts Guide

## Custom Bootloader Scripts

### flash-final.sh
**Purpose**: Flash custom Rust bootloader + test application

**Use when**: Ready to test custom bootloader

**What it does**:
1. Flash bootloader.hex to 0x08000000 (792 bytes)
2. Flash test-app.hex to 0x08004000 (7.8 KB)
3. Reset and run

**Expected result**: LED blinking at 1 Hz (500ms ON/OFF)

**Usage**:
```bash
./flash-final.sh
```

---

### flash-safe.sh
**Purpose**: Safe flash with backup and interactive confirmation

**Use when**: You want maximum safety with backup capability

**What it does**:
1. Backup current flash to backups/ directory
2. Flash bootloader only
3. Verify bootloader and check PC register
4. Ask for confirmation before flashing application
5. Flash application
6. Reset and run

**Expected result**: LED blinking at 1 Hz (500ms ON/OFF)

**Usage**:
```bash
./flash-safe.sh
```

---

## Arduino Recovery Scripts

### flash-arduino-blink.sh
**Purpose**: Restore working Arduino blink example

**Use when**: Need to restore device to known-good state

**What it does**:
1. Flash arduino-working-examples/arduino_blink.hex
2. Reset and run

**Expected result**: LED blinking at 1 Hz (Gecko bootloader + Arduino app)

**Usage**:
```bash
./flash-arduino-blink.sh
```

---

### flash-arduino-volume.sh
**Purpose**: Restore working Arduino volume example

**Use when**: Need to verify hardware or restore to working state

**What it does**:
1. Flash arduino-working-examples/microphone_volume_with_bootloader.hex
2. Reset and run

**Expected result**: LED responds to microphone input

**Usage**:
```bash
./flash-arduino-volume.sh
```

---

## Build Script

### build-all.sh
**Purpose**: Build bootloader and test application

**Use when**: After modifying source code

**What it does**:
1. Build bootloader in release mode
2. Convert ELF to HEX (bootloader.hex)
3. Build test application in release mode
4. Convert ELF to HEX (test-app.hex)

**Usage**:
```bash
./build-all.sh
```

---

## Quick Reference

| Goal | Script |
|------|--------|
| Test custom bootloader | `./flash-final.sh` |
| Test with backup/safety | `./flash-safe.sh` |
| Restore Arduino blink | `./flash-arduino-blink.sh` |
| Restore Arduino volume | `./flash-arduino-volume.sh` |
| Rebuild after changes | `./build-all.sh` |

---

## LED Behavior Reference

| Blink Pattern | Meaning |
|--------------|---------|
| 1 Hz (500ms ON/OFF) | SUCCESS - Application running correctly |
| Fast (~100ms) | HardFault - Check bootloader error handling |
| Slow (~1 second) | Bootloader mode - App validation failed |
| Solid ON | Check with debugger (stuck state) |
| OFF | Check connection or power |

---

## Safety Notes

All scripts use local OpenOCD from `./openocd/bin/openocd`

The SAMD11 CMSIS-DAP debugger is separate from the target MCU and cannot be bricked.

Recovery is always possible by flashing Arduino examples.

Backups are saved in `backups/` directory with timestamps.
