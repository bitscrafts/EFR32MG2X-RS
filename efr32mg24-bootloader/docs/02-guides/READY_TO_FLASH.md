# Ready to Flash - Custom Bootloader + Blink Test

**Status**: ✅ **ALL BUILDS SUCCESSFUL** - Ready for hardware testing

---

## What We've Built

### 1. Minimal Custom Bootloader
- **Size**: 792 bytes (~0.8 KB flash, 2 KB RAM)
- **Location**: 0x08000000 - 0x08000318
- **Features**:
  - Application validation (stack pointer + reset vector check)
  - VTOR setup for Cortex-M33
  - Safe application jump with memory barriers
  - HardFault handler (fast LED blink for debugging)
  - Bootloader mode (slow LED blink)

### 2. Test Blink Application
- **Size**: 7.8 KB
- **Location**: 0x08004000 (starts after 16 KB bootloader space)
- **Function**: Simple 1 Hz LED blink (500ms ON, 500ms OFF)
- **Uses**: Standard cortex-m-rt (no special requirements)

---

## Build Summary

**Bootloader**:
```
text:    792 bytes (code)
data:      0 bytes (initialized data)
bss:    2048 bytes (stack)
Total:  ~0.8 KB flash, 2 KB RAM
```

**Application**:
```
Size:   7.8 KB
```

**Total flash usage**: ~8.6 KB (out of 1536 KB available)

---

## Files Created

```
efr32mg24-bootloader/
├── bootloader.hex          # Bootloader Intel HEX (0x08000000)
├── test-app.hex            # Application Intel HEX (0x08004000)
└── flash-separate.sh       # Flash script (executable)
```

---

## Next Step: Flash to Hardware

### Option 1: Use the Flash Script (Recommended)

```bash
cd efr32mg24-bootloader
./flash-separate.sh
```

This will:
1. Connect via CMSIS-DAP (SAMD11 debug interface)
2. Flash bootloader at 0x08000000
3. Flash application at 0x08004000
4. Reset and run

### Option 2: Manual OpenOCD Commands

```bash
cd efr32mg24-bootloader

openocd -f interface/cmsis-dap.cfg -f ../efr32mg24.cfg \
  -c "init" \
  -c "program bootloader.hex verify" \
  -c "program test-app.hex verify" \
  -c "reset run" \
  -c "exit"
```

---

## Expected Behavior

### SUCCESS:
- LED blinks at 1 Hz (500ms ON, 500ms OFF)
- This means:
  - ✅ Bootloader validated application
  - ✅ Bootloader set VTOR correctly
  - ✅ Application is running
  - ✅ LED control works

### If LED blinks FAST (~100ms):
- HardFault occurred
- Check PC location: `openocd -c "init" -c "halt" -c "reg pc" -c "exit"`
- PC should NOT be in HardFault handler

### If LED blinks SLOW (~1 second):
- Bootloader mode activated
- Application didn't pass validation
- Check application vector table

### If LED doesn't blink at all:
- Check CMSIS-DAP connection
- Verify flash succeeded
- Check PC location (should be 0x08000xxx or 0x08004xxx)

---

## Debugging Commands

### Check PC (Program Counter) Location
```bash
openocd -f interface/cmsis-dap.cfg -f ../efr32mg24.cfg \
  -c "init" -c "halt" -c "reg pc" -c "exit"
```

Expected PC values:
- **0x08000000 - 0x08003FFF**: In bootloader
- **0x08004000 - 0x08180000**: In application ✅
- **Other**: Problem!

### Check Registers
```bash
openocd -f interface/cmsis-dap.cfg -f ../efr32mg24.cfg \
  -c "init" -c "halt" \
  -c "mdw 0x5003C03C 1" \  # GPIO DOUT (should toggle)
  -c "mdw 0xE000ED08 1" \  # VTOR (should be 0x08004000)
  -c "exit"
```

### Full Register Dump
```bash
openocd -f interface/cmsis-dap.cfg -f ../efr32mg24.cfg \
  -c "init" -c "halt" -c "reg" -c "exit"
```

---

## Key Differences from Gecko Bootloader

| Aspect | Gecko Bootloader | Custom Bootloader |
|--------|------------------|-------------------|
| **Size** | 24 KB | 0.8 KB |
| **Build** | Hex patching required | Direct build |
| **ApplicationProperties_t** | Required | Not needed |
| **Vector Table Word 13** | Points to magic | Standard ARM |
| **Application Start** | 0x08006000 | 0x08004000 |
| **App Requirements** | Special structure | Standard cortex-m-rt |
| **Debugability** | Black box (C) | Full visibility (Rust) |

---

## What This Proves

If the LED blinks at 1 Hz, this definitively proves:

1. ✅ **Custom bootloader works** - Successfully replaced 24 KB Gecko bootloader with 0.8 KB Rust bootloader
2. ✅ **VTOR setup correct** - Cortex-M33 vector table relocation working
3. ✅ **Application handoff works** - Clean jump from bootloader to application
4. ✅ **LED control works** - PA7 GPIO configured and responding
5. ✅ **Delays work** - 500ms timing (or close enough)
6. ✅ **Loop executes continuously** - The mystery from Gecko bootloader attempts solved!

---

## What We Learned from Gecko Bootloader Attempts

All knowledge from 13 Gecko bootloader attempts was applied:

1. ✅ CMU unlock (0x93F7 to 0x40008010)
2. ✅ GPIO clock enable (CMU_CLKEN0 bit 26)
3. ✅ PORTA offset (0x30 from GPIO base)
4. ✅ All register addresses correct
5. ✅ Active-low LED (0x00 = ON, 0x80 = OFF)
6. ✅ TrustZone-M non-secure state handling

---

## Next Steps After Successful Blink

### Phase 2: Add Firmware Update (Week 3-4)
- UART driver
- XMODEM-CRC protocol
- Host-side upload tool
- CRC32 validation

### Phase 3: Production Features (Week 5-6)
- Embassy-boot swap algorithm
- Power-fail-safe updates
- Rollback protection
- Size optimization

---

## If Flashing Fails

### CMSIS-DAP Not Found
```bash
# Check USB device
lsusb | grep -i "seeed\|samd"

# Try unplugging/replugging USB cable
```

### Permission Denied (Linux)
```bash
# Add udev rule for CMSIS-DAP
sudo tee /etc/udev/rules.d/99-cmsis-dap.rules <<EOF
SUBSYSTEM=="usb", ATTR{idVendor}=="2886", MODE="0666"
EOF
sudo udevadm control --reload-rules
```

### OpenOCD Connection Issues
```bash
# Try lower SWD clock
openocd -f interface/cmsis-dap.cfg -f ../efr32mg24.cfg \
  -c "adapter speed 1000" \  # 1 MHz instead of 10 MHz
  -c "init" ...
```

---

## Ready to Test!

**Current directory**: `efr32mg24-bootloader/`

**To flash**: `./flash-separate.sh`

**Expected result**: LED blinking at 1 Hz 🎯

---

**Date**: January 25, 2026
**Build status**: ✅ SUCCESS
**Files ready**: bootloader.hex, test-app.hex
**Next action**: Flash to hardware and observe LED!
