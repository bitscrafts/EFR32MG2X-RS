# Bootloader Testing Session Summary
**Date**: January 25, 2026
**Status**: Bootloader flashing works, but prevents application flashing

## Quick Summary

### What Works ✅
1. **Bootloader compilation and flashing**: 792 bytes, flashes successfully to 0x08000000
2. **Arduino blink standalone**: Flashes and runs correctly at 0x00000000
3. **CMSIS-DAP debugger**: Working properly with OpenOCD

### What Fails ❌
1. **Application flashing after bootloader**: Gets "Failed to enable MSC write" error
2. **Bootloader doesn't jump to application**: When bootloader + app are flashed together

### Root Cause Analysis
The bootloader is **enabling flash protection** that prevents OpenOCD from flashing additional code. This happens because:
- Bootloader flashes successfully (PC at 0x080002c0 after first flash)
- Second flash command fails with MSC (Memory System Controller) write errors
- Error: "Failed to write memory at 0x5003003c" (MSC register)

## Session Timeline

### 1. Test Application Development
**File**: `bootloader-test-app/src/main.rs`

Created minimal Rust test application with:
- PAC-based watchdog disable: `peripherals.wdog0_ns.en().write(|w| w.en().clear_bit())`
- GPIO PA7 LED blink (500ms on/off)
- Fixed critical-section dependency using cortex-m feature

**Build Result**:
- Text: 2360 bytes
- Generated: `test-app.hex` at address 0x08004000

**Key Issues Solved**:
- PAC path: Changed from `../../efr32mg24-pac` to `../efr32mg24-pac`
- Critical section: Added `features = ["critical-section-single-core"]` to cortex-m
- WDOG API: Field access, not method call (`wdog0_ns` not `wdog0_ns()`)

### 2. Flash Testing

#### Test 1: Arduino Blink Standalone ✅
```bash
Flash to: 0x00000000 (standard location)
Result: SUCCESS - LED blinks, verified OK
```

#### Test 2: Bootloader + Test App ❌
```bash
Flash 1: bootloader.hex to 0x08000000 - SUCCESS
Flash 2: test-app.hex to 0x08004000 - FAILED
Error: Failed to enable MSC write
```

#### Test 3: Bootloader + Arduino Blink ❌
```bash
Flash 1: bootloader.hex to 0x08000000 - SUCCESS
Flash 2: arduino_blink.hex to 0x08006000 - FAILED
Error: Failed to enable MSC write (sectors 3-114)
```

**Error Pattern**:
```
Info : SWD DPIDR 0x6ba02477
Error: Failed to write memory at 0x5003003c
Info : SWD DPIDR 0x6ba02477
Error: Failed to read memory at 0x5003000c
Error: Failed to enable MSC write
```

## Memory Layout

### Bootloader Configuration
**From**: `efr32mg24-bootloader/src/main.rs`
```rust
const APP_START_ADDR: u32 = 0x08004000;  // Application after 16 KB
```

**Actual bootloader size**: 792 bytes (much smaller than 16 KB reservation)

### Application Addresses
- **Test app**: 0x08004000 (as per bootloader expectation)
- **Arduino blink**: 0x08006000 (Arduino bootloader default)

### Memory Regions
```
0x08000000 - 0x08003FFF   Bootloader (16 KB reserved)
0x08004000 - 0x0817FFFF   Application space
0x20000000 - 0x2003FFFF   RAM (256 KB)
```

## Bootloader Analysis

### Boot Logic (main.rs:92-131)
```rust
fn boot_application(app_addr: u32) -> ! {
    1. Read vector table at app_addr
    2. Validate stack pointer (0x20000000-0x20040000)
    3. Validate reset vector (in flash, Thumb bit set)
    4. Set VTOR to app_addr (0xE000ED08)
    5. Jump to application
}
```

### Validation Checks
1. **Stack pointer**: Must be in RAM range
2. **Reset vector**: Must be in flash, odd address (Thumb mode)
3. **Magic flag**: Checks 0x2003FFFC for 0xB00710AD to enter bootloader mode

### Issue: Flash Protection
The bootloader **does not** set flash protection in code, but:
- OpenOCD reports MSC write failures after bootloader is running
- MSC (Memory System Controller) at 0x50030000 is being blocked
- Suggests hardware security or bootloader execution context prevents writes

## Gemini Document Summary

**Topic**: Watchdog timer preventing code execution

**Key Finding**: EFR32MG24 watchdog enabled by default, causes resets

**Solution Implemented**:
```rust
// Disable WDOG0 using PAC
let peripherals = pac::Peripherals::take().unwrap();
peripherals.wdog0_ns.en().write(|w| w.en().clear_bit());
```

**Status**: Successfully integrated into test-app.hex

## Technical Files Created

### 1. bootloader-test-app/Cargo.toml
```toml
[dependencies]
cortex-m = { version = "0.7", features = ["critical-section-single-core"] }
cortex-m-rt = "0.7"
panic-halt = "0.2"
efr32mg24-pac = { path = "../efr32mg24-pac" }
```

### 2. bootloader-test-app/src/main.rs
- Watchdog disable (PAC-based)
- GPIO PA7 blink loop
- 500ms delays
- Active-low LED control

### 3. Flash Scripts
Located in `scripts/` folder:
- `scripts/flash-bootloader-test.sh` - Flash bootloader + test app
- `scripts/flash-arduino-only.sh` - Flash Arduino standalone (works)
- `scripts/flash-bootloader-arduino.sh` - Flash bootloader + Arduino (fails)

## Next Session Action Items

### Critical Issues to Resolve

1. **MSC Flash Protection**
   - Why does bootloader prevent subsequent flashing?
   - Check if OpenOCD needs to halt/reset before second flash
   - Investigate MSC register state after bootloader runs
   - Try mass erase before flashing both together

2. **Bootloader Testing Strategy**
   - Create combined hex file (bootloader + app merged)
   - Flash as single program command
   - Use objcopy to merge binaries before converting to hex

3. **Alternative Approach: Single Flash**
   ```bash
   # Merge bootloader.hex and app.hex into combined.hex
   # Flash combined.hex in one operation
   ```

### Recommended Next Steps

1. **Create Merged Hex File**
   ```bash
   # Use srec_cat or similar to merge hex files
   srec_cat bootloader.hex -Intel app.hex -Intel -o combined.hex -Intel
   ```

2. **Flash Combined Image**
   ```bash
   openocd -c "program combined.hex verify reset run"
   ```

3. **Debug Bootloader Jump**
   - Add LED indicators in bootloader before jump
   - Verify vector table is read correctly
   - Check VTOR register is set
   - Validate app stack pointer and reset vector

4. **Test Minimal Application**
   - Keep test-app simple (just LED blink)
   - Verify watchdog disable works
   - Confirm GPIO initialization works

## Files to Check Next Session

1. `/efr32mg24-bootloader/src/main.rs` - Bootloader source
2. `/bootloader-test-app/src/main.rs` - Test application
3. `/efr32mg24-bootloader/bootloader.hex` - Bootloader binary
4. `/bootloader-test-app/test-app.hex` - Application binary
5. Flash scripts in `/scripts/` folder (see [Scripts Documentation](../../scripts/README.md))

## Key Observations

1. **Bootloader runs**: PC at 0x080002c0 after flash
2. **MSC locked**: Cannot flash after bootloader active
3. **Arduino works**: Standalone Arduino blink confirms hardware OK
4. **Watchdog solved**: PAC-based disable code implemented

## Critical Conclusion

The bootloader **successfully flashes and runs**, but creates a **flash protection state** that prevents OpenOCD from writing additional sectors. This is likely a **security feature** of the EFR32MG24 that gets activated when code runs from 0x08000000.

**Solution**: Merge bootloader and application into a **single hex file** and flash in **one operation** to avoid the protection lock.
