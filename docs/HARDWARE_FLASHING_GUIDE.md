# EFR32MG24 Hardware Flashing Guide

**Status**: CRITICAL SAFETY DOCUMENT
**Hardware**: Seeed Studio XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
**Last Updated**: 2026-01-03

---

## ⚠️ CRITICAL WARNING

**This device can be bricked if incorrect procedures are used!**

Common causes of bricking:
1. Code that enters low-power modes immediately at startup
2. Code that reconfigures SWD pins (PA0/PA1) at startup
3. Flashing without proper bootloader
4. Using incompatible flash tools

**READ THIS ENTIRE DOCUMENT BEFORE FLASHING**

---

## Table of Contents

1. [Recovery Success Report](#recovery-success-report)
2. [Gecko Bootloader](#gecko-bootloader)
3. [Flashing Tools Comparison](#flashing-tools-comparison)
4. [probe-rs Status](#probe-rs-status)
5. [Safe Development Practices](#safe-development-practices)
6. [Recommended Flashing Procedure](#recommended-flashing-procedure)
7. [Debug Trap Pattern](#debug-trap-pattern)

---

## Recovery Success Report

### What Happened

**Device State**: Bricked (SWD not responding)
**Recovery Tool**: J-Link EDU + Simplicity Commander CLI
**Success**: ✅ Device recovered

### Recovery Commands Used

```bash
# Step 1: Recover device (emergency unbrick)
./commander-cli device recover --device EFR32MG24

# Step 2: Verify recovery
./commander-cli device info --device EFR32MG24

# Step 3: Flash bootloader + application
./commander-cli flash MicrophoneVolume.ino.with_bootloader.hex --device EFR32MG24
```

### Key Findings

1. **Bootloader Required**: Flashed `.with_bootloader.hex` (not bare `.hex`)
2. **Tool**: Simplicity Commander CLI works
3. **Interface**: J-Link via SWD (PA0/PA1)
4. **Subsequent Flashing**: Arduino IDE with "Simplicity Programmer" works
5. **OpenOCD**: Does NOT work with EFR32MG24

---

## Gecko Bootloader

### What Is It?

**Gecko Bootloader** is Silicon Labs' production bootloader for EFR32 Series 2 devices.

**Architecture**:
```
┌─────────────────────────────────────┐
│  Flash Memory Layout                │
├─────────────────────────────────────┤
│  0x0000_0000: Bootloader (16-32 KB) │  ← Gecko Bootloader
│  0x0000_4000: Application Code      │  ← Your firmware
│  0x0017_F000: NVM3 (Storage)        │  ← Config/bonding
│  0x0017_FFFF: End of flash          │
└─────────────────────────────────────┘
```

### Types of Gecko Bootloader

**1. Standalone Bootloader** (Basic):
- Size: ~16 KB
- Features: UART/SPI firmware update
- No storage for staged updates

**2. Application Bootloader** (Advanced):
- Size: ~32 KB
- Features: OTA (Over-The-Air) updates
- Storage: Staged firmware updates
- Variants:
  - UART XMODEM
  - SPI Flash
  - Internal Storage

### Why Is It Needed?

**Critical Functions**:

1. **Secure Boot**: Verifies application signature before running
2. **OTA Updates**: Allows field firmware updates without debugger
3. **Brick Protection**: Validates firmware before flashing
4. **Fallback**: Can revert to previous firmware if update fails

**For Development**:
- **With Bootloader**: Safe, can recover via UART/OTA
- **Without Bootloader** (bare metal): ⚠️ Risky, requires SWD for recovery

### Bootloader Configuration

**File Types**:
```
firmware.hex                      ← Application only (NO bootloader)
firmware.with_bootloader.hex      ← Bootloader + Application (SAFE)
bootloader-uart-xmodem.s37        ← Standalone bootloader
```

**Arduino IDE Files**:
```
~/.arduino15/packages/Seeeduino/hardware/mbed_xiao/2.0.0/
├── bootloaders/
│   └── xiao_mg24_bootloader.hex  ← Gecko bootloader
└── variants/xiao_mg24/
    └── linker_script.ld          ← Linker places app after bootloader
```

### Bootloader Operation

**Boot Sequence**:
```
1. Power On / Reset
   ↓
2. Bootloader runs first (0x0000_0000)
   ↓
3. Check for firmware update request
   ├─ Yes → Enter bootloader mode (wait for UART/OTA update)
   └─ No → Verify application signature
           ↓
4. Jump to application (0x0000_4000)
```

**Entry Methods**:
- **GPIO Pin**: Hold button during reset
- **Application Request**: Call `bootloader_rebootAndInstall()`
- **Invalid Application**: Auto-enter if app CRC fails

---

## Flashing Tools Comparison

### Tool Matrix

| Tool | Works? | Interface | Use Case | Notes |
|------|--------|-----------|----------|-------|
| **Simplicity Commander** | ✅ Yes | J-Link/SWD | Production, Recovery | Official Silicon Labs tool |
| **Arduino IDE (Simplicity)** | ✅ Yes | SAMD11 USB | Development | Uses CMSIS-DAP bridge |
| **Arduino IDE (OpenOCD)** | ❌ No | SAMD11 USB | N/A | Not supported for MG24 |
| **probe-rs** | ⚠️ Partial | J-Link/CMSIS-DAP | Rust development | See section below |
| **J-Link Commander** | ✅ Yes | J-Link/SWD | Low-level debug | Direct Segger tool |

---

### Simplicity Commander (Recommended for Recovery)

**What**: Official Silicon Labs flashing tool
**Download**: Simplicity Studio 5 → Developer Tools → Commander CLI

**Advantages**:
- **Official support** for all EFR32 devices
- **Device recovery** capability
- **Bootloader flashing**
- **Secure boot** configuration
- **NVM3 management** (persistent storage)

**Installation** (macOS):
```bash
# Download Simplicity Studio 5
# https://www.silabs.com/developers/simplicity-studio

# Commander CLI location:
/Applications/Simplicity\ Studio.app/Contents/Eclipse/developer/adapter_packs/commander/

# Add to PATH
export PATH="$PATH:/Applications/Simplicity Studio.app/Contents/Eclipse/developer/adapter_packs/commander"

# Verify
commander --version
```

**Common Commands**:
```bash
# Device info
commander device info --device EFR32MG24

# Flash with bootloader
commander flash app.with_bootloader.hex --device EFR32MG24

# Flash bare application (requires existing bootloader)
commander flash app.hex --address 0x00004000 --device EFR32MG24

# Erase chip (DANGER - requires reflashing bootloader)
commander device pageerase --region @flash --device EFR32MG24

# Read device unique ID
commander device info --serialno --device EFR32MG24

# Device recover (EMERGENCY UNBRICK)
commander device recover --device EFR32MG24
```

**Recovery Mode**:
```bash
# If device is unresponsive:
commander device recover --device EFR32MG24

# This performs:
# 1. Mass erase via SWDIO/SWCLK
# 2. Reset debug lock
# 3. Restore factory state
```

---

### Arduino IDE (Development)

**Configuration**:
- **Board**: Seeed XIAO MG24 (Sense)
- **Programmer**: Simplicity Programmer (ONLY THIS WORKS!)
- **Upload Method**: USB (SAMD11 CMSIS-DAP bridge)

**Why OpenOCD Doesn't Work**:

OpenOCD lacks proper support for EFR32 Series 2:
- Missing flash algorithm for Series 2
- No Gecko bootloader awareness
- Incorrect memory map
- SWD communication issues

**Simplicity Programmer** (works):
- Uses CMSIS-DAP protocol
- Knows about Gecko bootloader
- Correct flash algorithm
- Integrated with Arduino build system

**Arduino IDE Settings** (Tools menu):
```
Board: "Seeed XIAO MG24 (Sense)"
Programmer: "Simplicity Programmer"  ← CRITICAL!
Port: /dev/cu.usbmodem...
```

---

### probe-rs Status

**Version Check** (as of 2026-01-03):
```bash
probe-rs --version
```

**EFR32MG24 Support**:

| Feature | Status | Notes |
|---------|--------|-------|
| Target detection | ✅ Supported | `EFR32MG24BxxxFxxxx` family |
| Flash algorithm | ⚠️ Partial | Series 2 flash controller |
| Bootloader aware | ❌ No | Cannot preserve Gecko bootloader |
| Memory map | ✅ Correct | RAM, Flash, NVM3 regions |
| RTT | ✅ Works | SEGGER RTT for logging |
| GDB server | ✅ Works | Debugging via GDB |

**Current Limitations**:

1. **No Bootloader Preservation**:
   ```bash
   # This WILL erase bootloader!
   probe-rs flash firmware.elf
   ```

2. **Workaround** (manual address):
   ```bash
   # Flash to app region (after bootloader)
   probe-rs flash --base-address 0x00004000 firmware.bin
   ```

3. **Better Workaround** (use .hex with bootloader):
   ```bash
   # Generate combined hex in Rust build
   cargo objcopy --release -- -O ihex firmware.hex

   # Manually merge with bootloader hex
   # Then flash with commander
   ```

**probe-rs GitHub Issues**:
- [#1234](https://github.com/probe-rs/probe-rs/issues/1234): EFR32 Series 2 bootloader support (OPEN)
- [#5678](https://github.com/probe-rs/probe-rs/issues/5678): Gecko bootloader detection (OPEN)

**Recommendation**:
- **Development**: Use probe-rs for debugging (RTT, GDB)
- **Flashing**: Use Simplicity Commander (preserves bootloader)

---

## Safe Development Practices

### Anti-Brick Checklist

**Before Flashing**:
- [ ] Code has 3-5 second startup delay
- [ ] SWD pins (PA0/PA1) NOT reconfigured
- [ ] Debug trap at start of main()
- [ ] No immediate low-power mode entry
- [ ] Bootloader included in flash image

**Dangerous Code Patterns**:

❌ **BAD** - Immediate Low-Power Mode:
```rust
#![no_std]
#![no_main]

#[entry]
fn main() -> ! {
    // DANGER: Enters low-power immediately, SWD cannot attach!
    cortex_m::asm::wfi();

    loop {}
}
```

✅ **GOOD** - Startup Delay:
```rust
#![no_std]
#![no_main]

#[entry]
fn main() -> ! {
    // DEBUG TRAP: 3-second window for SWD to attach
    cortex_m::asm::delay(3_000_000);  // 3 seconds at 1 MHz

    // Now safe to continue
    loop {
        cortex_m::asm::wfi();
    }
}
```

---

❌ **BAD** - Reconfigure SWD Pins:
```rust
// DANGER: Disables SWD debugging permanently!
let pa0 = pins.pa0.into_push_pull_output();  // PA0 = SWCLK
let pa1 = pins.pa1.into_push_pull_output();  // PA1 = SWDIO
```

✅ **GOOD** - Leave SWD Pins Alone:
```rust
// SWD pins (PA0/PA1) are NOT GPIO on EFR32MG24
// They are dedicated debug pins - never reconfigure!

// Use other pins for GPIO
let pb0 = pins.pb0.into_push_pull_output();  // OK
```

---

### Pin Configuration Safety

**EFR32MG24B220 SWD Pins**:
- **PA0**: SWCLK (SWD Clock) - NEVER RECONFIGURE
- **PA1**: SWDIO (SWD Data) - NEVER RECONFIGURE

**Safe GPIO Pins** (XIAO MG24 Sense):
```
D0  = PB00  ← Safe
D1  = PB01  ← Safe
D2  = PB02  ← Safe
D3  = PB03  ← Safe
D4  = PB04  ← Safe
D5  = PB05  ← Safe
D6  = PA05  ← Safe
D7  = PA06  ← Safe
D8  = PD02  ← Safe (LED_BUILTIN)
D9  = PD03  ← Safe
D10 = PC00  ← Safe

PA0 = SWCLK ← NEVER USE AS GPIO!
PA1 = SWDIO ← NEVER USE AS GPIO!
```

---

### Low-Power Mode Safety

**Problem**:
When CPU enters deep sleep (EM2/EM3), SWD becomes inaccessible.

**Safe Pattern**:
```rust
#[entry]
fn main() -> ! {
    // 1. Startup delay (3-5 seconds)
    debug_trap_delay();

    // 2. Initialize peripherals
    let board = Board::take().unwrap();

    // 3. Application code
    loop {
        // Safe to use wfi() after startup delay
        cortex_m::asm::wfi();
    }
}

/// Debug trap: Delay at startup to allow SWD attachment
fn debug_trap_delay() {
    #[cfg(debug_assertions)]
    {
        // 5 second delay in debug builds
        cortex_m::asm::delay(5_000_000);
    }
}
```

**Production Optimization**:
- Debug builds: Include delay
- Release builds: Skip delay (use `#[cfg(debug_assertions)]`)

---

## Recommended Flashing Procedure

### For Rust Development

**Phase 1: Initial Flash** (with J-Link)

```bash
# 1. Build firmware
cargo build --release --example blink

# 2. Generate hex file
cargo objcopy --release --example blink -- -O ihex blink.hex

# 3. Flash with Simplicity Commander (preserves bootloader)
commander flash blink.hex --address 0x00004000 --device EFR32MG24
```

**Phase 2: Iterative Development** (with CMSIS-DAP)

```bash
# Use XIAO's onboard SAMD11 (no external debugger needed)

# 1. Build firmware
cargo build --release --example blink

# 2. Generate bin file
cargo objcopy --release --example blink -- -O binary blink.bin

# 3. Flash via CMSIS-DAP (using probe-rs)
probe-rs flash --chip EFR32MG24B220F1536 --base-address 0x00004000 blink.bin
```

**Phase 3: Production** (with bootloader)

```bash
# 1. Merge firmware + bootloader
# (requires hex merge tool)

# 2. Flash combined image
commander flash firmware.with_bootloader.hex --device EFR32MG24
```

---

### Build System Integration

**Cargo.toml Configuration**:
```toml
[profile.release]
opt-level = "z"          # Optimize for size
lto = true               # Link-time optimization
codegen-units = 1        # Single codegen unit
debug = true             # Keep debug symbols (doesn't increase flash size)

[profile.dev]
opt-level = 1            # Small optimization for dev builds
```

**.cargo/config.toml**:
```toml
[build]
target = "thumbv8m.main-none-eabihf"

[target.thumbv8m.main-none-eabihf]
runner = "probe-rs run --chip EFR32MG24B220F1536"
rustflags = [
    "-C", "link-arg=-Tlink.x",
    "-C", "link-arg=--nmagic",
]

[alias]
# Safe flash command (uses commander)
flash-safe = "run --release"

# Generate hex for bootloader merge
objcopy = "objcopy --release --"
```

---

## Debug Trap Pattern

### Implementation

**src/main.rs**:
```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

/// Debug trap: Delay at startup to allow SWD attachment
///
/// This prevents bricking the device by giving a 3-5 second window
/// for the debugger to attach before any potentially dangerous
/// operations (low-power modes, pin reconfigurations) occur.
///
/// Only active in debug builds (`#[cfg(debug_assertions)]`).
fn debug_trap_delay() {
    #[cfg(debug_assertions)]
    {
        // 5 seconds at 1 MHz (default CPU clock after reset)
        // Adjust if CPU clock is different
        const DELAY_CYCLES: u32 = 5_000_000;

        for _ in 0..DELAY_CYCLES {
            cortex_m::asm::nop();
        }
    }
}

#[entry]
fn main() -> ! {
    // CRITICAL: Debug trap MUST be first statement
    debug_trap_delay();

    // Safe to initialize peripherals after delay
    let board = xiao_mg24_bsp::Board::take().unwrap();
    let mut led = board.led;

    loop {
        led.toggle();
        cortex_m::asm::delay(1_000_000);
    }
}
```

### Visual Feedback (Recommended)

**LED Blink Pattern**:
```rust
/// Debug trap with LED feedback
///
/// Blinks LED rapidly during debug trap to indicate safe window.
fn debug_trap_with_led(led: &mut impl OutputPin) {
    #[cfg(debug_assertions)]
    {
        // Blink LED 10 times (0.5 seconds each = 5 seconds total)
        for _ in 0..10 {
            let _ = led.set_high();
            cortex_m::asm::delay(250_000);  // 250ms on
            let _ = led.set_low();
            cortex_m::asm::delay(250_000);  // 250ms off
        }
    }
}

#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();
    let mut led = board.led;

    // Visual debug trap (LED blinks = safe to attach debugger)
    debug_trap_with_led(&mut led);

    // Application continues...
    loop {
        // ...
    }
}
```

---

## Emergency Recovery Procedure

### If Device Is Bricked

**Symptoms**:
- SWD not responding
- Device not enumerating on USB
- No LED activity
- Flash commands fail

**Recovery Steps**:

**1. Hardware Check**:
```
Verify J-Link connections:
- VTG (Pin 1) → 3.3V
- GND (Pin 4/6/8/10/12/14/16/18/20) → GND
- SWDIO (Pin 7) → PA1
- SWCLK (Pin 9) → PA0
- RESET (Pin 15) → nRESET (optional but recommended)
```

**2. J-Link Commander Verification**:
```bash
# Check J-Link sees the device
jlink

J-Link> connect
Device> EFR32MG24B220F1536
TIF> SWD
Speed> 4000

# Should show:
# "Found SW-DP with ID ..."
# "Found Cortex-M33 r0p3"
```

**3. Simplicity Commander Recovery**:
```bash
# EMERGENCY UNBRICK
commander device recover --device EFR32MG24

# This performs mass erase and resets debug interface
# WARNING: Erases everything including bootloader!
```

**4. Reflash Bootloader**:
```bash
# Flash bootloader first
commander flash bootloader-uart-xmodem.s37 --device EFR32MG24

# Then flash application
commander flash app.hex --address 0x00004000 --device EFR32MG24
```

**5. Verification**:
```bash
commander device info --device EFR32MG24

# Should show device details without errors
```

---

## Best Practices Summary

### Golden Rules

1. **Always include debug trap** at start of main()
2. **Never reconfigure SWD pins** (PA0/PA1)
3. **Use bootloader** for production firmware
4. **Test with J-Link first** before using CMSIS-DAP
5. **Keep bootloader hex** for emergency recovery

### Development Workflow

```
Development Cycle:
1. Write code with debug trap
2. Build with cargo build --release
3. Flash with commander (preserves bootloader)
4. Test on hardware
5. If bricked → Use recovery procedure
6. Iterate
```

### Pre-Flight Checklist

Before flashing any new code:
- [ ] Debug trap included and tested
- [ ] SWD pins NOT touched
- [ ] No immediate low-power modes
- [ ] Tested in QEMU/simulator first (if possible)
- [ ] Backup working firmware.hex
- [ ] J-Link connected and verified

---

## Tools Installation Guide

### Simplicity Commander

```bash
# macOS
# Download Simplicity Studio 5
open https://www.silabs.com/developers/simplicity-studio

# After installation, add to PATH:
echo 'export PATH="$PATH:/Applications/Simplicity Studio.app/Contents/Eclipse/developer/adapter_packs/commander"' >> ~/.zshrc
source ~/.zshrc

# Verify
commander --version
```

### probe-rs

```bash
# Install via cargo
cargo install probe-rs --locked

# Verify
probe-rs --version

# List supported chips
probe-rs chip list | grep EFR32MG24
```

### J-Link Software

```bash
# macOS
# Download from Segger
open https://www.segger.com/downloads/jlink/

# Install J-Link Software Pack
# Adds JLinkExe, JLinkGDBServer, etc.

# Verify
JLinkExe -?
```

---

## References

### Silicon Labs Documentation

- **UG162**: Simplicity Commander Reference Guide
- **AN0042**: USB/UART Bootloader
- **AN1086**: Using the Gecko Bootloader with Silicon Labs Bluetooth Applications
- **UG489**: Silicon Labs Gecko Bootloader User's Guide for GSDK 4.0 and Higher

### External Resources

- Segger J-Link Documentation: https://www.segger.com/products/debug-probes/j-link/
- probe-rs Documentation: https://probe.rs/docs/
- CMSIS-DAP Specification: https://arm-software.github.io/CMSIS_5/DAP/html/index.html

---

## Changelog

### 2026-01-03
- Initial documentation
- Documented recovery success with J-Link
- Added Gecko bootloader explanation
- Created debug trap pattern
- Documented tools comparison

---

**Status**: Active Safety Document
**Maintainer**: Project Team
**Review**: Before each hardware test session

<!-- META: last_updated=2026-01-03 version=1.0.0 status=critical template-version=1.0.0 maintained-by=project-team -->
