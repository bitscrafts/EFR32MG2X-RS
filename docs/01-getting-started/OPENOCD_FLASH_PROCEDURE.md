# OpenOCD Flash Procedure for XIAO MG24 via CMSIS-DAP

**Date**: 2026-01-03
**Hardware**: Seeed Studio XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
**Interface**: CMSIS-DAP (onboard SAMD11 USB bridge)
**Status**: VERIFIED WORKING

---

## Summary

This document describes the **verified working procedure** for flashing the XIAO MG24 Sense board using OpenOCD via the onboard CMSIS-DAP interface (USB).

**Key Discovery**: OpenOCD with CMSIS-DAP works correctly for flashing EFR32MG24, unlike:
- Simplicity Commander (requires J-Link)
- probe-rs (erases bootloader, wrong memory map initially)

---

## Prerequisites

### Software Requirements

**OpenOCD** (Arduino-bundled version recommended):
```bash
# Location (macOS)
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd

# Test connection
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset halt; exit"
```

### Hardware Requirements

1. **XIAO MG24 Sense** board
2. **USB Cable** (USB-C)
3. **CMSIS-DAP** interface (built into XIAO MG24 via SAMD11 chip)

---

## Flash Memory Layout

**EFR32MG24B220F1536IM48 Memory Map**:

```
0x0800_0000 - 0x0800_3FFF: Gecko Bootloader (16 KB)
0x0800_4000 - 0x0817_FFFF: Application Code (1520 KB)
0x2000_0000 - 0x2003_FFFF: SRAM (256 KB)
```

**CRITICAL**: EFR32MG24 has flash at **0x08000000**, NOT 0x00000000!

---

## Verified Working Procedure

### Step 1: Connect Device

```bash
# Connect XIAO MG24 via USB
# Device should enumerate as /dev/cu.usbmodem*

# Verify connection
ls /dev/cu.usbmodem*
# Expected: /dev/cu.usbmodem24D15EE43 (or similar)
```

### Step 2: Flash with OpenOCD

**Command Structure**:
```bash
openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program <hexfile> verify; reset; exit"
```

**Example (Flash Arduino Application with Bootloader)**:
```bash
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program MicrophoneVolume.ino.with_bootloader.hex verify; reset; exit"
```

**Expected Output**:
```
Info : Using CMSIS-DAPv2 interface with VID:PID=0x2886:0x0062, serial=24D15EE4
Info : CMSIS-DAP: SWD supported
Info : CMSIS-DAP: FW Version = 2.0.0
Info : SWD DPIDR 0x6ba02477
Info : [efm32s2.cpu] Cortex-M33 r0p4 processor detected
[efm32s2.cpu] halted due to debug-request
** Programming Started **
Info : detected part: MG24B220, rev 22
Info : flash size = 1536 KiB
Info : flash page size = 8192 B
** Programming Finished **
** Verify Started **
** Verified OK **
```

### Step 3: Verify Operation

```bash
# Device should reset automatically
# Application should start running

# For Rust blink example:
# - LED should blink rapidly 10 times (debug trap)
# - Then blink slowly at 1 Hz
```

---

## Configuration Files

### interface/cmsis-dap.cfg

Located in OpenOCD scripts directory:
```
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/share/openocd/scripts/interface/cmsis-dap.cfg
```

**Content** (standard OpenOCD):
```tcl
# CMSIS-DAP interface adapter

adapter driver cmsis-dap
transport select swd
```

### target/efm32s2_g23.cfg

Located in OpenOCD scripts directory:
```
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/share/openocd/scripts/target/efm32s2_g23.cfg
```

**Content**:
```tcl
# Family group 23 has flash base address 0x08000000
set FLASHBASE 0x08000000
source [find target/efm32s2.cfg]
```

**Note**: This is correct for EFR32MG24B220 (our chip).

---

## Custom Target Configuration (Optional)

For clearer naming, create `efr32mg24.cfg`:

```tcl
#
# Silicon Labs EFR32MG24 target configuration
# Family group 24 - Flash base address 0x08000000
#

set FLASHBASE 0x08000000
source [find target/efm32s2.cfg]
```

**Usage**:
```bash
openocd -f interface/cmsis-dap.cfg -f efr32mg24.cfg -c "..."
```

---

## Rust Firmware Flashing

### Memory Configuration

**Cargo.toml** (efr32mg24-pac):
```toml
[dependencies]
cortex-m-rt = "0.7"

[features]
rt = ["cortex-m-rt/device"]
```

**memory.x**:
```
MEMORY
{
  /* Application flash - starts after 16 KB Gecko bootloader */
  /* NOTE: EFR32MG24 has flash at 0x08000000 (like STM32), not 0x00000000 */
  FLASH : ORIGIN = 0x08004000, LENGTH = 1520K

  /* SRAM - 256 KB total */
  RAM : ORIGIN = 0x20000000, LENGTH = 256K
}

/* Required by cortex-m-rt */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
```

### Build and Flash Procedure

```bash
# Step 1: Build firmware
cd efr32mg24-pac
cargo build --release --example blink --features rt

# Step 2: Generate hex file
cargo objcopy --release --example blink --features rt -- -O ihex blink.hex

# Step 3: Flash with OpenOCD
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program blink.hex verify; reset; exit"
```

---

## Troubleshooting

### Issue: "no flash bank found for address 0x00004000"

**Cause**: Memory.x has flash at 0x00000000 instead of 0x08000000

**Fix**: Update memory.x:
```
- FLASH : ORIGIN = 0x00004000, LENGTH = 1520K
+ FLASH : ORIGIN = 0x08004000, LENGTH = 1520K
```

### Issue: "Warn : no flash bank found for address 0x00004000"

**Cause**: Hex file has wrong address base

**Fix**: Rebuild firmware with correct memory.x and regenerate hex file

### Issue: Device crashes at 0x08000170

**Cause**: Bootloader was erased (address 0x08000170 is in bootloader region)

**Fix**: Restore bootloader:
```bash
# Flash Arduino hex with bootloader
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program MicrophoneVolume.ino.with_bootloader.hex verify; reset; exit"
```

### Issue: "Error: busy command USB transfer"

**Cause**: USB communication timing issue (non-fatal)

**Fix**: Ignore if "Verified OK" appears. Device flashed successfully.

### Issue: "Cannot connect to J-Link"

**Cause**: Using Simplicity Commander instead of OpenOCD

**Fix**: Use OpenOCD for CMSIS-DAP, commander requires J-Link

---

## Comparison: OpenOCD vs probe-rs vs Commander

| Feature | OpenOCD | probe-rs | Commander |
|---------|---------|----------|-----------|
| **CMSIS-DAP Support** | ✅ Yes | ✅ Yes | ❌ No (J-Link only) |
| **Preserves Bootloader** | ✅ Yes | ❌ No | ✅ Yes |
| **Correct Memory Map** | ✅ Yes (0x08000000) | ⚠️ Yes but initially tried wrong address | ✅ Yes |
| **Verification** | ✅ Built-in | ⚠️ Optional | ✅ Built-in |
| **Ease of Use** | ⚠️ Manual commands | ✅ Cargo integration | ✅ Simple CLI |
| **Recovery Mode** | ❌ No | ❌ No | ✅ Yes (device recover) |

**Recommendation**:
- **Development**: OpenOCD via CMSIS-DAP (this procedure)
- **Production**: Simplicity Commander via J-Link (when available)
- **Debugging**: probe-rs for RTT/GDB (after fixing memory map)

---

## Safe Flashing Checklist

Before flashing any firmware:

- [ ] Hex file addresses start at 0x08004000 (after bootloader)
- [ ] Bootloader hex available for recovery
- [ ] CMSIS-DAP device detected (`ls /dev/cu.usbmodem*`)
- [ ] OpenOCD can connect (`init; reset halt; exit`)
- [ ] Firmware has debug trap (3-5 second delay at startup)
- [ ] SWD pins (PA0/PA1) NOT reconfigured in code

---

## Example: Flash Arduino MicrophoneVolume (VERIFIED WORKING)

This is the **exact command that successfully flashed** the Arduino example on 2026-01-03:

```bash
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset_config srst_nogate; reset halt; program /Users/${FLASH_USER:-$(id -un)}/Private/20250626_rust_projects/2025-10-29/EFR32MG24/xiao_mg24_flash_erase_macos/SimplicityCommander-Mac/Commander-cli.app/Contents/MacOS/MicrophoneVolume.ino.with_bootloader.hex verify; reset; exit"
```

**Result**:
- ✅ Flashed successfully
- ✅ Verified OK
- ✅ Device reset and bootloader running
- ✅ Application code functional

---

## Quick Reference Commands

### Test Connection
```bash
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; reset halt; exit"
```

### Flash Hex File
```bash
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program firmware.hex verify; reset; exit"
```

### Flash Binary at Specific Address
```bash
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; reset halt; flash write_image erase firmware.bin 0x08004000; verify_image firmware.bin 0x08004000; reset; exit"
```

### Erase Chip (DANGER)
```bash
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; reset halt; flash erase_sector 0 0 last; exit"
```

---

## Known Limitations

1. **USB Transfer Errors**: Occasional "busy command USB transfer" messages appear but don't affect flashing (firmware still verifies OK)

2. **Reset Timing**: Sometimes device doesn't auto-reset, manual power cycle may be needed

3. **Bootloader Awareness**: OpenOCD doesn't prevent overwriting bootloader if hex file includes 0x08000000 region

4. **Large Files**: Files >512 KB may take 30-60 seconds to program

---

## References

- OpenOCD Documentation: http://openocd.org/doc/
- CMSIS-DAP Specification: https://arm-software.github.io/CMSIS_5/DAP/html/index.html
- EFR32MG24 Reference Manual: Chapter 9 (Flash Controller)
- Seeed XIAO MG24 Schematic: Shows SAMD11 CMSIS-DAP configuration

---

## Changelog

### 2026-01-03
- Initial documentation
- Verified MicrophoneVolume flash procedure via CMSIS-DAP
- Documented correct flash base address (0x08000000)
- Added troubleshooting for common issues
- Confirmed OpenOCD is the working solution for CMSIS-DAP

---

**Status**: Production Ready
**Tested**: 2026-01-03 with XIAO MG24 Sense
**Author**: Project Team

