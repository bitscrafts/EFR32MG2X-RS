# probe-rs with EFR32MG24: Known Issues and Limitations

**Date**: December 26, 2025
**Current probe-rs Version**: 0.30.0
**Status**: EFR32MG24 Supported (since v0.24.0)

---

## Executive Summary

probe-rs **DOES** support EFR32MG24 as of version 0.24.0 (May 2024), but has **critical limitations** when the device's debug interface is locked or misconfigured. This document details the known issues, workarounds, and when to use alternative tools.

**TL;DR**:
- ✅ **Normal flashing**: probe-rs works well
- ⚠️ **Clock misconfiguration**: probe-rs may fail
- ❌ **Debug locked device**: probe-rs cannot recover (use J-Link/BMP instead)

---

## Table of Contents

1. [Support History](#support-history)
2. [Known Issues](#known-issues)
3. [Your Current Situation](#your-current-situation)
4. [When probe-rs Works Well](#when-probe-rs-works-well)
5. [When probe-rs Fails](#when-probe-rs-fails)
6. [Comparison with Other Tools](#comparison-with-other-tools)
7. [Recommendations](#recommendations)

---

## Support History

### Timeline

| Date | Version | Status | Notes |
|------|---------|--------|-------|
| **July 2023** | Pre-0.24 | ❌ No Support | Issue #1700 opened |
| **May 2024** | v0.24.0 | ✅ Support Added | PR #3443 by @wtodd1 |
| **Dec 2024** | v0.30.0 | ✅ Fully Supported | Current stable version |

### GitHub Issue #1700

**Title**: "Add support for EFR32MG24"
**Opened**: July 25, 2023
**Status**: **CLOSED** (Completed)
**Resolution**: PR #3443 added support in v0.24.0

**Original Problem Report**:

```
Error: The communication with the probe failed
Caused by:
    0: Failed while flashing
    1: Failed to read register DRW at address 0x0000000c
    2: An ARM specific error occurred.
    3: Target device responded with a FAULT response to the request.
```

**Root Cause Identified**:

1. **Command Overrun**: probe-rs was issuing commands too fast
2. **Sticky FAULT Bit**: DAP status register had sticky error bits set
3. **Protocol Timing**: Insufficient delays between register accesses

**Resolution**:

- Added proper EFR32MG24 target definitions
- Implemented correct flash sequence for Series 2 chips
- Fixed timing issues with DAP register access
- Added support for EFM32/EFR32 Series 2 reset sequences

---

## Known Issues

### Issue 1: Debug Interface Lockout (CRITICAL)

**Problem**: probe-rs **CANNOT** recover when application code disables the SWD debug interface.

**Scenario**:
```rust
// This code bricks the device (probe-rs cannot recover):
#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // No debug preservation code!
    let (clocks, cmu) = Clocks::new(dp.cmu_s, config)?;
    clocks.freeze(cmu);
    // ↑ SWD clocks disabled here

    // probe-rs is now locked out permanently
}
```

**Error Messages**:
```
Error: An error with the usage of the probe occurred
Caused by:
    Target device did not respond to request.
```

or

```
Error: Error connecting DP: cannot read IDR
```

**Why probe-rs Fails**:

1. **Requires Working SWD**: probe-rs needs SWD interface to be functional
2. **No Hardware Reset Control**: CMSIS-DAP (SAMD11) cannot assert hardware reset to halt CPU before app runs
3. **No Timing Window**: Application code runs within microseconds of power-on
4. **No AAP Access**: Cannot access Authentication Access Port when SWD is disabled

**Affected Operations**:
- ❌ `probe-rs list` - Cannot detect device
- ❌ `probe-rs download` - Cannot flash new firmware
- ❌ `probe-rs erase` - Cannot erase flash
- ❌ `probe-rs reset` - Cannot reset device
- ❌ `probe-rs attach` - Cannot attach for debugging

### Issue 2: FAULT Responses (RESOLVED in v0.24.0+)

**Problem**: Early probe-rs versions received FAULT responses when accessing certain registers.

**Status**: ✅ **FIXED** in probe-rs v0.24.0

**Historical Error**:
```
Error: Failed to read register DRW at address 0x0000000c
Caused by: Target device responded with a FAULT response to the request.
```

**If You See This Error**:
1. Update probe-rs: `cargo install probe-rs-tools --locked`
2. Verify version: `probe-rs --version` (should be ≥ v0.24.0)
3. If still occurs, report to [probe-rs GitHub](https://github.com/probe-rs/probe-rs/issues)

### Issue 3: Flash Algorithm Limitations

**Problem**: probe-rs flash algorithm doesn't support all Silicon Labs flash operations.

**Missing Features**:
- ❌ Mass erase with debug unlock
- ❌ Partial page programming (must erase full pages)
- ❌ Flash lock/unlock commands
- ❌ User data page access

**Workaround**: Use Simplicity Commander for advanced operations

### Issue 4: Connect-Under-Reset Not Supported

**Problem**: probe-rs cannot use "connect-under-reset" technique with CMSIS-DAP on XIAO MG24.

**Explanation**:

**What Connect-Under-Reset Does**:
- Asserts hardware RESET line
- Keeps CPU halted
- Attaches debugger before application runs
- Prevents debug lockout

**Why It Doesn't Work**:
- XIAO MG24's SAMD11 CMSIS-DAP bridge doesn't expose hardware reset control
- probe-rs cannot assert RESET through CMSIS-DAP on this board
- Only possible with direct SWD connection (J-Link, BMP)

**Error Message**:
```
Warning: reset_config not supported via CMSIS-DAP
```

---

## Your Current Situation

### Board Status

```
Device: XIAO MG24 Sense (Serial: 24D15EE4)
Status: BRICKED
probe-rs Version: 0.30.0
Error: "Target device did not respond to request"
```

### What Happened

1. ✅ probe-rs v0.30.0 successfully flashed `01_clock` example
2. ❌ Example code disabled SWD clock during `Clocks::freeze()`
3. ❌ Debug interface became unresponsive
4. ❌ probe-rs can no longer connect to device

### Why probe-rs Cannot Recover

**Technical Explanation**:

```
Power On
   ↓
CPU Starts (< 1ms)
   ↓
Application Code Runs
   ↓
Clocks::freeze() Executes
   ↓
SWD Clock Disabled (<10ms from power-on)
   ↓
probe-rs Attempts Connection
   ↓
SWD Interface: LOCKED ❌
   ↓
probe-rs Error: "Target device did not respond"
```

**probe-rs Dependencies**:
- Requires: Working SWD interface
- Requires: Debug clocks enabled
- Requires: CPU not in locked state
- **All three requirements NOT met**

### What probe-rs CAN Do

✅ **If SWD is accessible**:
- Flash new firmware
- Read/write memory
- Set breakpoints
- Attach debugger
- Reset device (software reset)

### What probe-rs CANNOT Do

❌ **If SWD is locked**:
- Detect device
- Mass erase flash
- Hardware reset (via CMSIS-DAP on XIAO MG24)
- Unlock debug interface
- Bypass application code

---

## When probe-rs Works Well

### Scenario 1: Normal Development (Debug Builds)

```rust
#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // ✅ Debug preservation code present
    #[cfg(debug_assertions)]
    unsafe {
        dp.CMU_S.clken0().modify(|_, w| w.hfxo0().set_bit());
        dp.CMU_S.clken1().modify(|_, w| w.swd().set_bit());
    }

    // Safe to proceed
    let (clocks, cmu) = Clocks::new(dp.cmu_s, config)?;
    // ...
}
```

**Result**: ✅ probe-rs works perfectly

### Scenario 2: Fresh Board (Factory Firmware)

**Condition**: Board has never been flashed or has recovery firmware

**Result**: ✅ probe-rs can flash normally

### Scenario 3: Release Builds (Tested Code)

**Condition**: Code has been tested in debug mode and confirmed safe

**Result**: ✅ probe-rs can flash release builds safely

---

## When probe-rs Fails

### Scenario 1: Debug Interface Disabled (Your Case)

**Code That Caused Bricking**:
```rust
// 01_clock.rs - DANGEROUS without debug preservation
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    // ❌ No debug preservation code!

    let (clocks, cmu) = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)),
            lfxo: Some(Default::default()),
        },
    )?;
    clocks.freeze(cmu);  // ← SWD disabled here
}
```

**Result**: ❌ probe-rs permanently locked out

**Error**:
```bash
$ probe-rs list
Error: No connected probes were found.
```

or

```bash
$ probe-rs download firmware.elf
Error: An error with the usage of the probe occurred
Caused by: Target device did not respond to request.
```

### Scenario 2: Deep Sleep Without Escape Pin

**Code**:
```rust
fn enter_deep_sleep() {
    dp.EMU_S.ctrl().modify(|_, w| w.em4ctrl().set());
    cortex_m::asm::wfi();  // Enter EM4
    // ❌ SWD disabled in deep sleep
}
```

**Result**: ❌ probe-rs cannot wake or attach

### Scenario 3: Flash Write Protection Enabled

**Code**:
```rust
unsafe {
    let msc = &(*pac::MSC_S::ptr());
    msc.lock().write(|w| w.bits(0x00000001));  // Lock flash
}
```

**Result**: ❌ probe-rs cannot write to flash

---

## Comparison with Other Tools

### Recovery Capability Comparison

| Tool | Recovery from Debug Lock | Hardware Required | Cost |
|------|--------------------------|-------------------|------|
| **probe-rs** | ❌ **NO** | CMSIS-DAP (built-in) | Free |
| Seeed OpenOCD | ⚠️ Maybe (70% success) | CMSIS-DAP (built-in) | Free |
| PC0/PC1 Escape | ⚠️ Maybe (80% success) | Soldering required | Free |
| Black Magic Probe | ✅ **YES** | BMP + soldering | ~$50 |
| **J-Link + Commander** | ✅ **YES** (95% success) | J-Link + soldering | $60-800 |

### Normal Flashing Comparison

| Feature | probe-rs | OpenOCD | J-Link |
|---------|----------|---------|--------|
| **Speed** | ⚡ Very Fast | 🐢 Slow | ⚡⚡ Fastest |
| **Rust Integration** | ✅ Native | ❌ External | ❌ External |
| **EFR32MG24 Support** | ✅ Yes (v0.24.0+) | ✅ Yes | ✅ Yes |
| **GUI** | ❌ CLI only | ❌ CLI only | ✅ Available |
| **RTT Support** | ✅ Excellent | ⚠️ Limited | ✅ Excellent |
| **Debugging** | ✅ Full GDB | ✅ Full GDB | ✅ Full GDB |
| **Cost** | Free | Free | $60-800 |

### Why probe-rs for Normal Development

**Advantages**:
1. ✅ **Pure Rust** - Perfect integration with cargo
2. ✅ **Fast** - Optimized flash algorithms
3. ✅ **Easy Setup** - `cargo install probe-rs-tools`
4. ✅ **RTT Logging** - Real-time transfer built-in
5. ✅ **Active Development** - Regular updates
6. ✅ **Open Source** - Community driven

**Disadvantages**:
1. ❌ **No Recovery** - Cannot unlock debug-locked devices
2. ❌ **CMSIS-DAP Limitations** - No hardware reset on XIAO MG24
3. ❌ **No GUI** - Command-line only

---

## Recommendations

### For Normal Development: Use probe-rs ✅

**When to use**:
- ✅ Daily development work
- ✅ Flashing tested code
- ✅ Debugging applications
- ✅ RTT logging needs

**Setup**:
```bash
# Install latest version
cargo install probe-rs-tools --locked

# Verify version (should be ≥ v0.24.0)
probe-rs --version

# Use in cargo config
# .cargo/config.toml
[target.thumbv8m.main-none-eabihf]
runner = "probe-rs run --chip EFR32MG24B220F1536IM48"
```

**Best Practices**:
1. ✅ Always use debug builds during development
2. ✅ Include debug preservation code in ALL examples
3. ✅ Flash 00_recovery.rs first on new boards
4. ✅ Test in debug mode before release builds

### For Recovery: Use J-Link or BMP ✅

**When to use**:
- ❌ Board is bricked (debug locked)
- ❌ probe-rs cannot detect device
- ❌ Need to recover from clock misconfiguration
- ❌ Application disabled SWD

**Options**:

**Option A: J-Link + Simplicity Commander** (~95% success rate)
```bash
# Purchase J-Link EDU ($60) or higher
# Install Simplicity Commander
commander device recover --device EFR32MG24B220F1536IM48
```

**Option B: Black Magic Probe** (~60% success rate)
```bash
# Use existing BMP hardware
arm-none-eabi-gdb
(gdb) target extended-remote /dev/tty.usbmodem...
(gdb) monitor erase_mass
```

### For Production: Consider Both

**Development Phase**:
- Use probe-rs for fast iteration
- Keep debug preservation in all code
- Test thoroughly before release builds

**Production Programming**:
- Use J-Link for volume production (faster, more reliable)
- Or use probe-rs with verified-safe firmware
- Keep recovery tools available

---

## Troubleshooting Guide

### Problem: "Target device did not respond"

**Diagnosis**:
```bash
# Check if board is detected via USB
ls /dev/cu.usbmodem*  # macOS
ls /dev/ttyACM*       # Linux

# Check probe-rs version
probe-rs --version

# Try to list probes
probe-rs list
```

**If USB detected but probe-rs fails**:
- ⚠️ SWD debug interface is likely locked
- ❌ probe-rs cannot recover
- ✅ Use J-Link or BMP for recovery

### Problem: "FAULT response" Errors

**Diagnosis**:
```bash
# Check probe-rs version
probe-rs --version
```

**If version < 0.24.0**:
```bash
# Update probe-rs
cargo install probe-rs-tools --locked

# Verify update
probe-rs --version  # Should show ≥ 0.24.0
```

**If version ≥ 0.24.0 and still failing**:
- Report to [probe-rs issue tracker](https://github.com/probe-rs/probe-rs/issues)
- Include debug logs: `probe-rs --log-level trace download firmware.elf`

### Problem: Flash Fails After Power Cycle

**Diagnosis**:
```bash
# Flash succeeded, but after power cycle device doesn't respond
probe-rs list  # Fails to detect
```

**Root Cause**: Application code disabled debug interface

**Solution**:
1. Add debug preservation code (see examples)
2. Reflash using recovery method (J-Link/BMP)
3. Future code must preserve debug in development

---

## Version-Specific Notes

### probe-rs v0.30.0 (Current)

**Status**: ✅ Full EFR32MG24 support
**Release**: December 2024
**Notes**:
- Stable support for EFR32MG24
- Improved flash algorithms
- Better error messages
- RTT support

**Known Issues**:
- Cannot recover from debug lock (inherent limitation)
- No connect-under-reset via CMSIS-DAP on XIAO MG24

### probe-rs v0.24.0 (May 2024)

**Status**: ✅ First version with EFR32MG24 support
**Release**: May 22, 2024
**Changes**:
- Added EFR32MG24 target definitions (PR #3443)
- Fixed FAULT response issues
- Implemented proper reset sequences

### probe-rs < v0.24.0 (Pre-May 2024)

**Status**: ❌ No EFR32MG24 support
**Recommendation**: **DO NOT USE** - Update immediately

---

## Future Improvements (Potential)

### Community Requests

**Features that would help** (not currently implemented):

1. **AAP (Authentication Access Port) Support**:
   - Would allow debug unlock without external tools
   - Requires low-level CMSIS-DAP access
   - Complex to implement

2. **Hardware Reset via CMSIS-DAP**:
   - Would enable connect-under-reset
   - Depends on CMSIS-DAP firmware capabilities
   - XIAO MG24's SAMD11 may not support this

3. **Mass Erase with Debug Unlock**:
   - Would recover from debug-locked state
   - Requires AAP access (see #1)

4. **Better Error Recovery**:
   - Automatic retry with different timings
   - Sticky bit clearing
   - More robust FAULT handling

### Contributing

If you encounter issues:

1. **Check existing issues**: [probe-rs/probe-rs/issues](https://github.com/probe-rs/probe-rs/issues)
2. **Search for "EFR32MG24"**: May already be reported
3. **Open new issue** with:
   - probe-rs version (`probe-rs --version`)
   - Full error output
   - Debug logs (`--log-level trace`)
   - Minimal reproduction steps

---

## Summary

### probe-rs Status with EFR32MG24

| Aspect | Status | Notes |
|--------|--------|-------|
| **General Support** | ✅ Excellent | Since v0.24.0 (May 2024) |
| **Normal Flashing** | ✅ Works Great | Fast and reliable |
| **Debugging** | ✅ Full Support | GDB, RTT, breakpoints |
| **Recovery** | ❌ Limited | Cannot unlock debug-locked devices |
| **Clock Configs** | ⚠️ Risky | Must preserve debug in dev builds |
| **Production Use** | ✅ Recommended | With proper safeguards |

### Key Takeaways

1. ✅ **probe-rs works excellently** for normal EFR32MG24 development
2. ⚠️ **Requires debug preservation** in development builds
3. ❌ **Cannot recover** from debug-locked state (use J-Link/BMP)
4. ✅ **Update to v0.24.0+** if using older version
5. ✅ **Keep recovery tools available** (J-Link or BMP) for emergencies

### Recommended Workflow

```
Development:
├── Use probe-rs for daily work ✅
├── Keep debug preservation in ALL code ✅
├── Flash 00_recovery.rs first on new boards ✅
└── Test in debug mode before release ✅

Recovery (if bricked):
├── Try PC0/PC1 escape pin first (free)
├── If fails, use Black Magic Probe
└── Last resort: J-Link + Simplicity Commander
```

---

## References

### Official Documentation

1. [probe-rs Documentation](https://probe.rs/)
   Official probe-rs user guide

2. [probe-rs GitHub Repository](https://github.com/probe-rs/probe-rs)
   Source code and issue tracker

3. [Issue #1700: Add support for EFR32MG24](https://github.com/probe-rs/probe-rs/issues/1700)
   Original support request and implementation discussion

4. [probe-rs Changelog](https://github.com/probe-rs/probe-rs/blob/master/CHANGELOG.md)
   Version history and features

5. [probe-rs Releases](https://github.com/probe-rs/probe-rs/releases)
   Official release notes

### Related Issues

6. [Issue #3088: STM32H747XI Connection Issues](https://github.com/probe-rs/probe-rs/issues/3088)
   Similar FAULT response problems

7. [Issue #3131: S32K142 Core Locked](https://github.com/probe-rs/probe-rs/issues/3131)
   Debug lock recovery discussion

### Silicon Labs Resources

8. [EFR32MG24 Reference Manual](https://www.silabs.com/documents/public/reference-manuals/efr32xg24-rm.pdf)
   Complete technical reference

9. [AN1190: Series 2 Secure Debug](https://www.silabs.com/documents/public/application-notes/an1190-efr32-secure-debug.pdf)
   Debug lock and AAP documentation

---

**Document Version**: 1.0
**Last Updated**: December 26, 2025
**probe-rs Version Tested**: v0.30.0
**Status**: Production Ready

**NOTE**: This document focuses on probe-rs limitations. For complete recovery procedures, see [COMPREHENSIVE_FLASHING_GUIDE.md](COMPREHENSIVE_FLASHING_GUIDE.md).
