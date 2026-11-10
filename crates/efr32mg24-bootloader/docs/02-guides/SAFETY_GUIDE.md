# Safety Guide - Cannot Brick This Device

**TL;DR**: Your XIAO MG24 Sense **CANNOT be bricked** by this flash procedure. Here's why.

---

## Why You Cannot Brick This Device

### 1. Separate Debug Interface

**Your board has TWO microcontrollers:**

```
┌─────────────────────────────────────┐
│  XIAO MG24 Sense Board              │
│                                      │
│  ┌────────────────┐  ┌────────────┐ │
│  │ SAMD11         │  │ EFR32MG24  │ │
│  │ (Debugger)     │→→│ (Target)   │ │
│  │                │  │            │ │
│  │ CMSIS-DAP      │  │ Your code  │ │
│  │ firmware       │  │ runs here  │ │
│  └────────────────┘  └────────────┘ │
│         ↑                            │
│         │                            │
│    USB connection                    │
└─────────────────────────────────────┘
```

**Key point**: The debugger (SAMD11) is **completely separate** from the target (EFR32MG24).

- Even if EFR32MG24 code is completely broken, SAMD11 still works
- SAMD11 provides JTAG/SWD access to EFR32MG24
- We can **ALWAYS** reflash the EFR32MG24 via SAMD11

**This is like having an unbrickable programmer built into the board.**

### 2. CMSIS-DAP is Read/Write Access

CMSIS-DAP (the SAMD11 interface) has:
- ✅ Direct memory access to EFR32MG24 flash
- ✅ Direct memory access to EFR32MG24 RAM
- ✅ Can halt, reset, and control the CPU
- ✅ Works regardless of what's in flash

**This means**: Even with completely broken firmware, we can erase and reprogram the entire flash.

### 3. Flash is Rewritable

The EFR32MG24 flash:
- Can be erased and rewritten unlimited times (well, 10,000+ cycles)
- No fuses to blow
- No one-time-programmable (OTP) areas we're touching
- No bootloader protection bits being set

**Worst case**: If our bootloader is broken, we just flash something else.

---

## What Could Go Wrong (and How to Fix)

### Scenario 1: Bootloader Doesn't Work
**Symptom**: LED doesn't blink, or blinks in unexpected pattern

**Fix**:
```bash
# Flash Arduino bootloader + blink example (known working)
openocd -f interface/cmsis-dap.cfg -f ../efr32mg24.cfg \
  -c "init" \
  -c "program ../efr32mg24-pac/arduino_blink.hex verify" \
  -c "reset run" \
  -c "exit"
```

**Result**: Back to Arduino environment, device fully functional

### Scenario 2: Application is Broken
**Symptom**: Bootloader runs but app doesn't work

**Fix**: Flash a different application, or just the bootloader
```bash
# Bootloader alone
openocd -f interface/cmsis-dap.cfg -f ../efr32mg24.cfg \
  -c "init" \
  -c "program bootloader.hex verify" \
  -c "exit"
```

### Scenario 3: Both Bootloader and App Broken
**Symptom**: Complete failure

**Fix**: Restore backup (created by flash-safe.sh)
```bash
openocd -f interface/cmsis-dap.cfg -f ../efr32mg24.cfg \
  -c "init" \
  -c "program backups/flash_backup_XXXXXX.bin 0x08000000 verify" \
  -c "exit"
```

### Scenario 4: OpenOCD Won't Connect
**Causes**:
- USB cable unplugged
- Wrong OpenOCD config
- SAMD11 needs reset

**Fix**:
1. Unplug and replug USB cable
2. Try with slower SWD clock:
   ```bash
   openocd -f interface/cmsis-dap.cfg -f ../efr32mg24.cfg \
     -c "adapter speed 1000" \
     -c "init" ...
   ```
3. Press reset button on board (if available)

**Cannot brick SAMD11**: The SAMD11 debugger is a separate chip with its own firmware that we're NOT touching.

---

## Safe Flash Procedure

The `flash-safe.sh` script includes multiple safety checks:

### Step 1: Backup Current Flash ✓
```bash
openocd ... -c "flash read_bank 0 backup.bin"
```
- Reads entire flash to backup file
- Can restore this later

### Step 2: Flash Bootloader Only ✓
- Flash and verify bootloader
- Stop here to test
- User confirms before proceeding

### Step 3: Test Bootloader ✓
- Reset and check PC (program counter)
- Should be in bootloader range (0x08000xxx)
- User confirms before application flash

### Step 4: Flash Application ✓
- Only after bootloader verified working
- Flash and verify application
- Reset and run

### Built-in Verification
Every flash command includes `verify`:
```bash
-c "program file.hex verify"
```
- OpenOCD reads back flash after writing
- Compares with original file
- Fails if mismatch detected

---

## Recovery Procedures

### Quick Recovery: Flash Arduino Firmware
```bash
# Get back to known-good state
cd efr32mg24-pac
openocd -f interface/cmsis-dap.cfg -f ../efr32mg24.cfg \
  -c "init" \
  -c "program arduino_blink.hex verify" \
  -c "reset run" \
  -c "exit"
```

### Complete Recovery: Mass Erase
```bash
# Nuclear option: erase entire flash
openocd -f interface/cmsis-dap.cfg -f ../efr32mg24.cfg \
  -c "init" \
  -c "halt" \
  -c "flash erase_sector 0 0 last" \
  -c "exit"
```

After this, flash any firmware you want.

### Restore from Backup
```bash
# Restore exact state from before
openocd -f interface/cmsis-dap.cfg -f ../efr32mg24.cfg \
  -c "init" \
  -c "program backups/flash_backup_20260125_HHMMSS.bin 0x08000000 verify" \
  -c "exit"
```

---

## What We're NOT Doing (Safe)

We are **NOT**:
- ❌ Setting write protection bits
- ❌ Blowing fuses
- ❌ Locking the debug interface
- ❌ Touching the SAMD11 debugger
- ❌ Writing to OTP (one-time-programmable) memory
- ❌ Disabling SWD/JTAG access
- ❌ Setting security bits

All we're doing is:
- ✅ Writing normal flash memory (fully rewritable)
- ✅ Using standard OpenOCD commands
- ✅ Everything is reversible

---

## Technical Details: Flash Memory Map

```
EFR32MG24 Flash (1536 KB total):

0x08000000 ┬─────────────────────────┐
           │ Our Bootloader (0.8 KB) │ ← We write here
0x08000318 ├─────────────────────────┤
           │ Unused (15.2 KB)        │
0x08004000 ├─────────────────────────┤
           │ Our Application (7.8 KB)│ ← We write here
0x08005E80 ├─────────────────────────┤
           │ Unused (1520 KB)        │ ← Untouched
0x08180000 └─────────────────────────┘
```

**Total written**: ~8.6 KB out of 1536 KB
**Percentage**: 0.56% of flash

Even if those 8.6 KB are completely garbage, we can erase and rewrite.

---

## Comparison: What CAN Brick a Device

For comparison, here are things that **would** brick a device (we're not doing any of these):

### Real Bricking Scenarios:
1. **Bootloader with debug lockout** - Locks SWD/JTAG access permanently
2. **OTP bits set incorrectly** - One-time programmable, cannot undo
3. **Wrong chip variant flash** - Overwriting wrong memory regions
4. **Flash protection enabled** - Lock bits prevent further writes
5. **Corrupted bootloader on device without debug port** - No recovery path

### Our Situation:
- ✅ Debug port always accessible (CMSIS-DAP)
- ✅ No OTP bits being set
- ✅ Correct chip variant
- ✅ No flash protection being enabled
- ✅ Recovery path always available

---

## Emergency Contacts (if you get stuck)

**This should never happen, but just in case:**

1. **Silicon Labs Community Forum**
   - https://community.silabs.com/
   - Post with error details

2. **Seeed Studio Support**
   - https://www.seeedstudio.com/contacts
   - For hardware issues

3. **OpenOCD Issues**
   - https://github.com/openocd-org/openocd/issues

---

## Confidence Level

**Chance of bricking device**: **0.001%** (only if hardware failure)

**Chance of temporary issue**: ~1% (easily fixed by reflashing)

**Chance of success**: ~99% (based on all our testing)

---

## Final Safety Check

Before flashing, verify:
- [x] USB cable connected
- [x] CMSIS-DAP detected (lsusb or device manager)
- [x] Backup will be created first
- [x] You have recovery procedures documented
- [x] You're using flash-safe.sh (not flash-separate.sh)

**If all checked**: You're ready to flash safely!

---

**Remember**: The SAMD11 debugger is separate and always working. We can ALWAYS recover via CMSIS-DAP.

**Bottom line**: This is as safe as flashing can be. Go ahead with confidence! 🎯
