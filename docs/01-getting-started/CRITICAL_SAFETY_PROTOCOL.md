# CRITICAL SAFETY PROTOCOL - XIAO MG24 Flashing

**Date**: 2026-01-10
**Severity**: CRITICAL - Device Bricking Prevention

## ⚠️ NEVER DO THIS ⚠️

### ❌ DO NOT Flash Firmware Directly to 0x08000000

**CRITICAL DANGER**: Flashing firmware directly to flash base address 0x08000000 will **ERASE THE GECKO BOOTLOADER** and render the device non-functional!

**What happens:**
1. Gecko Bootloader at 0x08000000 - 0x08005FFF is erased
2. Application firmware is written starting at 0x08000000
3. Device can no longer boot Arduino applications
4. Requires manual bootloader restoration

**Example of DANGEROUS command:**
```bash
# ❌ NEVER RUN THIS ❌
openocd ... -c "program firmware_at_0x08000000.hex"
```

### ❌ DO NOT Use Direct Flash Memory Layout

**DANGEROUS memory.x configuration:**
```ld
/* ❌ NEVER USE THIS ❌ */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 1536K  /* ← Overwrites bootloader! */
  RAM : ORIGIN = 0x20000000, LENGTH = 256K
}
```

### ❌ DO NOT Erase Full Flash

**DANGEROUS OpenOCD commands:**
```bash
# ❌ NEVER RUN THESE ❌
openocd ... -c "flash erase_sector 0 0 last"  # Erases bootloader
openocd ... -c "flash erase_address 0x08000000 0x180000"  # Erases everything
```

## ✅ ALWAYS DO THIS ✅

### ✅ Use Application Start Address (0x08006000)

**SAFE memory.x configuration:**
```ld
/* ✅ CORRECT - Preserves bootloader ✅ */
MEMORY
{
  /* Bootloader: 0x08000000 - 0x08005FFF (24 KB) */
  FLASH : ORIGIN = 0x08006000, LENGTH = 1512K  /* ← Application area only */
  RAM : ORIGIN = 0x20000000, LENGTH = 256K - 8
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
```

### ✅ Flash Only Application Area

**SAFE OpenOCD command:**
```bash
# ✅ SAFE - Preserves bootloader ✅
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program application.hex verify; reset; exit"
```

**Requirements:**
- Hex file MUST start at 0x08006000 (application area)
- Hex file MUST NOT contain data at 0x08000000 - 0x08005FFF
- Bootloader region remains untouched

### ✅ Verify Hex File Before Flashing

**Check hex file start address:**
```bash
head -3 firmware.hex
# Should see:
# :020000040800F2          ← Extended address 0x08000000
# :106000000812...         ← First data at 0x6000 (→ 0x08006000) ✓
```

**If you see data at offset 0x0000:**
```
:020000040800F2
:100000000812...         ← DANGER: Data at 0x0000 (→ 0x08000000) ✗
```
**DO NOT FLASH THIS FILE!** It will erase the bootloader.

## 🚨 Recovery Procedure (If Bootloader Erased)

If you accidentally erased the bootloader:

### Step 1: Verify Device is Accessible
```bash
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; reset halt; exit"
# Should see: "Info : detected part: MG24B220, rev 22"
```

### Step 2: Restore Gecko Bootloader
```bash
cd ~/Library/Arduino15/packages/SiliconLabs/hardware/silabs/3.0.0/bootloaders

openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program seeed-studio-xiao-mg24-bootloader-storage-internal-single-512k.hex verify; reset; exit"
```

**Expected output:**
```
** Programming Started **
Info : detected part: MG24B220, rev 22
** Programming Finished **
** Verified OK **
```

### Step 3: Flash Application
```bash
cd /path/to/your/project

openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program application.hex verify; reset; exit"
```

### Step 4: Verify Device Works
Flash a known-working example (e.g., Arduino microphone example) and verify it runs.

## 📋 Pre-Flight Checklist

Before every flash operation:

- [ ] Verified hex file starts at 0x08006000 (not 0x08000000)
- [ ] memory.x uses `ORIGIN = 0x08006000`
- [ ] Not using `flash erase` commands
- [ ] Have recovery plan ready (bootloader hex file location known)
- [ ] Have known-working firmware to verify recovery

## 🔍 Why This Matters

### Gecko Bootloader Architecture

```
Flash Memory Layout:
┌──────────────────────────────────────┐
│ 0x08000000 - 0x08005FFF (24 KB)     │
│ ┌──────────────────────────────────┐ │
│ │   GECKO BOOTLOADER              │ │ ← NEVER TOUCH!
│ │   - Entry point: 0x08000170     │ │
│ │   - Validates app properties    │ │
│ │   - Jumps to application        │ │
│ └──────────────────────────────────┘ │
├──────────────────────────────────────┤
│ 0x08006000 - 0x0817FFFF (1512 KB)   │
│ ┌──────────────────────────────────┐ │
│ │   APPLICATION AREA              │ │ ← Flash here only
│ │   - Vector table at 0x08006000  │ │
│ │   - Application code            │ │
│ │   - Application data            │ │
│ └──────────────────────────────────┘ │
└──────────────────────────────────────┘
```

### Boot Sequence

1. **Power-on reset** → PC = 0x08000000 (bootloader entry)
2. **Bootloader runs** → Validates application properties
3. **Application validated** → Jump to 0x08006000
4. **Application runs** → Your code executes

**If bootloader is erased:**
1. Power-on reset → PC = 0x08000000 (invalid code)
2. Device crashes or hangs
3. Arduino applications cannot run
4. Manual recovery required

## 📖 Related Documentation

- [ARDUINO_BOOTLOADER_ANALYSIS.md](ARDUINO_BOOTLOADER_ANALYSIS.md) - Root cause analysis
- [RUST_BOOTLOADER_SOLUTION.md](RUST_BOOTLOADER_SOLUTION.md) - ApplicationProperties_t solution
- [OPENOCD_FLASH_PROCEDURE.md](OPENOCD_FLASH_PROCEDURE.md) - Safe flashing procedures

## 🚀 Quick Reference: Safe Flash Commands

### Flash Application (Preserves Bootloader)
```bash
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program app.hex verify; reset; exit"
```

### Restore Bootloader (Recovery Only)
```bash
cd ~/Library/Arduino15/packages/SiliconLabs/hardware/silabs/3.0.0/bootloaders
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program seeed-studio-xiao-mg24-bootloader-storage-internal-single-512k.hex verify; reset; exit"
```

### Verify Device
```bash
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; mdw 0x08000000 16; exit"
```

---

**REMEMBER**: When in doubt, **DO NOT FLASH**. Ask for help first!

**Emergency Contact**: Restore bootloader from `~/Library/Arduino15/packages/SiliconLabs/hardware/silabs/3.0.0/bootloaders/`

**Last Updated**: 2026-01-10
**Incident**: Bootloader accidentally erased during direct flash test
**Recovery**: Successfully restored using official bootloader hex file
