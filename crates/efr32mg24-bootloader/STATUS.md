# Custom Bootloader Status

**Date**: 2026-01-25
**Goal**: Minimal Rust bootloader (792 bytes) to replace Gecko bootloader (24 KB)

---

## Accomplished

### 1. Bootloader Implementation ✅
- Created minimal no_std bootloader (792 bytes total)
- Implemented proper ARM Cortex-M33 boot sequence:
  - RAM initialization (.bss zero-init)
  - Application validation (stack pointer + reset vector checks)
  - VTOR setup for application handoff
  - Safe application jump with memory barriers (DSB, ISB)
- **CRITICAL FIX**: Corrected Thumb bit in linker script (changed from `+1` to `|1`)
  - Original: `LONG(Reset + 1)` → Reset vector = 0x080002C2 (INVALID)
  - Fixed: `LONG(Reset | 1)` → Reset vector = 0x080002C1 (VALID)
  - This was causing "clearing lockup after double fault" errors

### 2. Infrastructure ✅
- Local OpenOCD setup (copied from Arduino IDE)
- Safe flash procedures with automatic backups
- Working Arduino examples preserved for recovery
- Documentation organized with doc-organizer skill
- Multiple flash scripts:
  - `flash-final.sh` - Quick flash
  - `flash-safe.sh` - With backups and confirmation
  - `flash-arduino-blink.sh` - Restore Arduino blink
  - `flash-arduino-volume.sh` - Restore Arduino volume (verified working)

### 3. Memory Layout ✅
```
0x08000000 - 0x08003FFF : Bootloader (16 KB allocated, 792 bytes used)
0x08004000 - 0x0817FFFF : Application (1520 KB available)
0x20000000 - 0x2003FFFF : RAM (256 KB)
```

---

## Solution Implemented ✅

### Separate Workspace Crates

Created independent crate structure to resolve build system conflicts:

```
EFR32MG24/
├── efr32mg24-bootloader/     # Standalone bootloader crate
│   ├── .cargo/config.toml     # Uses bootloader.x linker script
│   ├── Cargo.toml             # Not in workspace
│   └── build.rs               # Copies bootloader.x only
└── bootloader-test-app/       # Standalone test application crate
    ├── .cargo/config.toml     # Standard cortex-m-rt
    ├── Cargo.toml             # Not in workspace
    └── memory.x               # App memory: 0x08004000
```

**Results**:
```bash
# Bootloader
$ arm-none-eabi-size efr32mg24-bootloader
   text    data     bss     dec     hex filename
    792       0    2048    2840     b18 efr32mg24-bootloader  ✅

# Test Application
$ arm-none-eabi-size bootloader-test-app
   text    data     bss     dec     hex filename
   2248       0       0    2248     8c8 bootloader-test-app   ✅
```

**Hex Files**:
- Bootloader: 52 lines (was 52) ✅
- Test app: 144 lines (was 13 - FIXED!) ✅

**Flash Status**:
- Both binaries flashed successfully
- OpenOCD verify passed for both
- No errors during programming

**Next**: Observe LED behavior to confirm success

---

## What Works

### Verified Working ✅

1. **Build System**: Separate crates eliminate linker script conflicts
2. **Bootloader Build**: 792 bytes, correct Thumb bit in vector table
3. **Application Build**: 2248 bytes with non-zero sections
4. **Hex Generation**: Both files contain actual code
5. **Flash Procedure**: Both binaries program and verify successfully

### Pending Verification ⏳

**Current Status**: Application code executes but LED doesn't blink

**Evidence of bootloader success**:
- OpenOCD shows PC = 0x0800485e (inside application delay function) ✅
- Application code is executing in correct address range ✅
- Bootloader successfully validates and jumps to application ✅

**Remaining issue**: LED behavior
- Initially: Solid ON (code running, possibly stuck in tight loop)
- After changes: OFF or ON (unclear if watchdog or GPIO issue)

**Root cause**: UNKNOWN - needs further investigation

**Likely candidates**:
1. **Watchdog timer** - May need proper unlock/disable sequence ([AN0015.2](https://www.silabs.com/documents/public/application-notes/an0015.2-efr32-series-2-watchdog.pdf))
2. **GPIO pin assignment** - PA7 might not be the correct LED pin
3. **LED polarity** - Active-low vs active-high unclear
4. **Clock configuration** - Default clock might be different than expected

---

## Research Findings

### Silicon Labs Gecko Bootloader Analysis

Researched the [Gecko SDK](https://github.com/SiliconLabs/gecko_sdk) bootloader implementation:

1. **Application Validation** ([btl_bootload.c](https://github.com/SiliconLabs/gecko_sdk/blob/gsdk_4.1/platform/bootloader/core/btl_bootload.c)):
   - Validates stack pointer is in valid RAM range
   - Checks reset vector is in application flash with Thumb bit set
   - Optional CRC32 or ECDSA signature verification
   - ApplicationProperties structure for Series 2 devices

2. **Jump Sequence** (standard Cortex-M pattern):
   - Disable interrupts
   - Set VTOR to application address
   - Execute DSB/ISB memory barriers
   - Load stack pointer from application vector table
   - Jump to reset handler

3. **Key Resources**:
   - [Gecko Bootloader Documentation](https://docs.silabs.com/mcu-bootloader/latest/)
   - [Bootloaders on Cortex-M CPUs](https://pyjamabrah.com/posts/bootloader/)
   - [rustBoot](https://github.com/nihalpasham/rustBoot) - Rust bootloader examples

**Conclusion**: Our bootloader implementation is correct. The issue is purely the test application build system.

---

## Implementation Plan

See [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) for detailed solution strategies.

**Recommended Approach**: Create separate workspace crates
- Clean separation between bootloader and test app
- No build system conflicts
- Standard cortex-m-rt for test app
- Estimated time: 2-4 hours

**Alternative Approaches**:
1. Manual build commands with explicit RUSTFLAGS
2. Custom startup code without cortex-m-rt
3. Environment variable detection in build.rs (already tried, unreliable)

---

## Key Learnings

1. **Thumb Bit**: ARM Cortex-M requires LSB set in function pointers
   - Use `|1` not `+1` to preserve alignment

2. **VTOR**: Must be set before jumping to application
   - Address: 0xE000ED08
   - Must use DSB/ISB memory barriers

3. **Build System**: Cargo's build.rs can't reliably distinguish bins from examples
   - Need separate crates or explicit RUSTFLAGS

4. **OpenOCD**: Works well with local copy from Arduino IDE
   - No need for Homebrew installation

5. **Safety**: SAMD11 CMSIS-DAP is separate from target
   - Device cannot be bricked through flash operations

---

## Test When Fixed

Expected LED behavior:
- **1 Hz blink (500ms ON/OFF)** = SUCCESS
- **Fast blink (~100ms)** = HardFault
- **Slow blink (~1 second)** = Bootloader mode (app validation failed)
- **Solid ON** = Code running but stuck (current state)
- **OFF** = No power/connection

---

## Files

- `bootloader.hex` - 792 bytes, flashes successfully ✅
- `test-app.hex` - Currently invalid (13 bytes) ❌
- `bootloader.x` - Linker script (fixed Thumb bit) ✅
- `test-app-memory.x` - App memory layout ✅
- `build.rs` - Needs fix for example detection ❌

---

## Recovery

To restore working state:
```bash
./flash-arduino-volume.sh
```

Hardware verified working with Arduino volume example.
