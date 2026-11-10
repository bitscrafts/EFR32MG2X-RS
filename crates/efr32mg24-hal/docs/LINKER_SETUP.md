# Linker Script Setup for EFR32MG24 HAL

This document explains the linker script configuration for building ARM Cortex-M33 binaries for the EFR32MG24 microcontroller.

## Overview

The EFR32MG24 HAL uses the standard Cortex-M linker script setup from `cortex-m-rt`, with device-specific memory layout and interrupt vector configuration.

## File Structure

```
efr32mg24-hal/
├── memory.x          # Memory layout (Flash/RAM regions)
├── device.x          # Interrupt vector weak aliases
├── build.rs          # Build script to copy linker scripts
└── .cargo/
    └── config.toml   # Cargo configuration with linker flags
```

## Memory Layout (memory.x)

Located at: `efr32mg24-hal/memory.x`

```ld
/* Memory layout for EFR32MG24B220F1536IM48-B (XIAO MG24 Sense) */
MEMORY
{
  /* Main Flash - starts at 0x08000000 on EFR32MG24 */
  FLASH : ORIGIN = 0x08000000, LENGTH = 1536K
  /* SRAM - starts at 0x20000000 */
  RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* NOTE Do NOT modify `_stack_start` unless you know what you are doing */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
```

### Memory Regions

- **FLASH**: 1536 KB starting at `0x08000000`
  - Contains: Vector table, .text (code), .rodata (constants)
  - Non-volatile, read-only at runtime

- **RAM**: 256 KB starting at `0x20000000`
  - Contains: .data (initialized variables), .bss (zero-initialized), heap, stack
  - Volatile, read-write

- **Stack**: Grows downward from end of RAM (`0x20040000`)
  - Full descending stack (ARM standard)
  - 8-byte aligned as required by ARM EABI

## Device-Specific Interrupts (device.x)

Located at: `efr32mg24-hal/device.x`

This file provides weak aliases for all EFR32MG24-specific interrupt handlers. Each interrupt defaults to `DefaultHandler` unless explicitly defined by user code.

### Key Interrupt Categories

**Security & System**
- SMU_SECURE, SMU_S_PRIVILEGED - Security Management Unit
- SYSCFG - System Configuration
- EMU, EMUEFP, EMUDG - Energy Management Unit

**Timers**
- TIMER0-4 - General purpose timers
- LETIMER0 - Low Energy Timer
- BURTC - Backup RTC
- RTCC - Real-Time Counter and Calendar
- SYSRTC_APP, SYSRTC_SEQ - System RTC

**Communication Peripherals**
- USART0_RX, USART0_TX - Universal Synchronous/Asynchronous Receiver/Transmitter
- EUSART0_RX, EUSART0_TX, EUSART1_RX, EUSART1_TX, EUSART2_RX, EUSART2_TX
- I2C0, I2C1 - I2C buses

**GPIO**
- GPIO_ODD, GPIO_EVEN - GPIO interrupt lines

**Clock & Oscillators**
- CMU - Clock Management Unit
- HFXO0, HFXO1 - High Frequency Crystal Oscillators
- HFRCO0, HFRCO1 - High Frequency RC Oscillators
- HFRCOEM23 - HFRCO for EM2/EM3
- LFXO - Low Frequency Crystal Oscillator
- ULFRCO - Ultra Low Frequency RC Oscillator

**Radio/RF**
- MODEM - Modem
- PROTIMER - Protocol Timer
- RAC_SEQ, RAC_RSM - Radio Controller
- SYNTH - Frequency Synthesizer
- AGC - Automatic Gain Control
- RFSENSE, RFSENSEMBOX - RF Sense
- RFECA0, RFECA1 - RF ECA

**Other Peripherals**
- LDMA - Linked DMA
- IADC - Incremental ADC
- VDAC0, VDAC1 - Voltage DAC
- ACMP0, ACMP1 - Analog Comparators
- MSC - Memory System Controller
- ICACHE0 - Instruction Cache
- WDOG0, WDOG1 - Watchdogs
- DCDC - DC-DC Converter
- PCNT0 - Pulse Counter

### Usage Example

User code can override any interrupt handler:

```rust
use cortex_m_rt::interrupt;

#[interrupt]
fn TIMER0() {
    // Handle TIMER0 interrupt
}
```

If not overridden, interrupts route to `DefaultHandler` (infinite loop with breakpoint).

## Build Script (build.rs)

Located at: `efr32mg24-hal/build.rs`

```rust
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Copy memory.x to OUT_DIR so the linker can find it via INCLUDE directive
    fs::copy("memory.x", out_dir.join("memory.x"))
        .expect("Failed to copy memory.x");

    // Copy device.x to OUT_DIR so the linker can find it via INCLUDE directive
    fs::copy("device.x", out_dir.join("device.x"))
        .expect("Failed to copy device.x");

    // Add OUT_DIR to the linker search path
    // This allows link.x to find memory.x and device.x via INCLUDE directives
    // DO NOT add -Tmemory.x flag here - cortex-m-rt handles that
    println!("cargo:rustc-link-search={}", out_dir.display());

    // Re-run if linker scripts change
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=device.x");
}
```

### What it Does

1. **Copies linker scripts** to `OUT_DIR` (the build output directory)
2. **Adds OUT_DIR to linker search path** so `link.x` can find them via `INCLUDE` directives
3. **Triggers rebuild** when linker scripts change

### Important Notes

- Does NOT add `-Tmemory.x` flag (would cause duplicate processing)
- The `cortex-m-rt` crate's `link.x` already includes `memory.x` at line 23
- Adding `-Tmemory.x` would process memory.x twice, causing "region 'FLASH' already defined" error

## Cargo Configuration (.cargo/config.toml)

Located at: `.cargo/config.toml` (workspace root)

```toml
[build]
# Set default target for Cortex-M33 with hardware FPU
target = "thumbv8m.main-none-eabihf"

[target.thumbv8m.main-none-eabihf]
# Use ARM's link.x linker script from cortex-m-rt
# link.x will INCLUDE memory.x automatically - do NOT add -Tmemory.x here
runner = "probe-rs run --chip EFR32MG24A020F1536GM48"
rustflags = [
  "-C", "link-arg=-Tlink.x",
]
```

### Key Points

- **Target**: `thumbv8m.main-none-eabihf`
  - `thumbv8m.main` = ARMv8-M Mainline architecture (Cortex-M33)
  - `none` = No operating system (bare metal)
  - `eabihf` = ARM EABI with hardware floating point

- **Linker Flags**: Only `-Tlink.x`
  - `link.x` is generated by `cortex-m-rt` and includes `memory.x` automatically
  - Previously had `-Tmemory.x` which caused duplicate FLASH region errors

- **Runner**: probe-rs for flashing and debugging
  - Configured for XIAO MG24 Sense chip variant

## Linker Script Flow

```
┌─────────────────────────────────────────────────────┐
│ 1. Cargo invokes rustc with -Tlink.x                │
└─────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────┐
│ 2. Linker searches for link.x in:                   │
│    - cortex-m-rt OUT_DIR (found!)                   │
└─────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────┐
│ 3. link.x processes:                                 │
│    Line 23: INCLUDE memory.x                         │
│    Line 283: INCLUDE device.x                        │
└─────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────┐
│ 4. Linker searches for memory.x and device.x in:    │
│    - efr32mg24-hal OUT_DIR (found via build.rs!)    │
└─────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────┐
│ 5. Binary generated with correct memory layout      │
│    and interrupt vector table                        │
└─────────────────────────────────────────────────────┘
```

## Critical Section Support

The HAL requires `critical-section` implementation for safe peripheral access. This is provided by the `cortex-m` crate with the `critical-section-single-core` feature.

### Configuration

In `efr32mg24-hal/Cargo.toml`:

```toml
[dependencies]
cortex-m = { workspace = true, features = ["critical-section-single-core"] }
critical-section = { workspace = true }
```

### What This Provides

- `_critical_section_1_0_acquire` - Disables interrupts
- `_critical_section_1_0_release` - Re-enables interrupts

Used by:
- `Peripherals::take()` for safe singleton access
- Any code using `critical_section::with()`

## Common Issues and Solutions

### Issue 1: "region 'FLASH' already defined"

**Symptom:**
```
rust-lld: error: region 'FLASH' already defined
```

**Cause:**
memory.x is being processed twice - once via `INCLUDE memory.x` in link.x, and once via `-Tmemory.x` flag.

**Solution:**
Remove `-Tmemory.x` from rustflags in `.cargo/config.toml`. The `link.x` script will include it automatically.

### Issue 2: "undefined symbol: _critical_section_1_0_acquire"

**Symptom:**
```
rust-lld: error: undefined symbol: _critical_section_1_0_acquire
```

**Cause:**
Missing critical-section implementation.

**Solution:**
Add `critical-section-single-core` feature to cortex-m dependency:
```toml
cortex-m = { workspace = true, features = ["critical-section-single-core"] }
```

### Issue 3: "undefined symbol: TIMER0" (or other interrupts)

**Symptom:**
```
rust-lld: error: undefined symbol: TIMER0
```

**Cause:**
Missing weak aliases in device.x for device-specific interrupts.

**Solution:**
Add `PROVIDE(TIMER0 = DefaultHandler);` to device.x for each missing interrupt.

### Issue 4: "cannot find linker script memory.x"

**Symptom:**
```
rust-lld: error: cannot find linker script memory.x
```

**Cause:**
build.rs not copying memory.x to OUT_DIR, or OUT_DIR not in linker search path.

**Solution:**
Ensure build.rs:
1. Copies memory.x: `fs::copy("memory.x", out_dir.join("memory.x"))`
2. Adds search path: `println!("cargo:rustc-link-search={}", out_dir.display());`

## Building Examples

```bash
# Build single example
cargo build --example 01_clock --features rt --release

# Build all examples
cargo build --examples --features rt --release

# Check binary size
arm-none-eabi-size target/thumbv8m.main-none-eabihf/release/examples/01_clock

# Flash to device (requires probe-rs)
cargo run --example 01_clock --features rt --release
```

## Binary Output

Successfully built binaries for EFR32MG24:

```
-rwxr-xr-x  707K  01_clock
-rwxr-xr-x  725K  02_delay
-rwxr-xr-x  722K  03_gpio
```

All binaries are:
- ARM 32-bit LSB executables
- Statically linked
- Include debug info
- Ready to flash to EFR32MG24

## References

- [cortex-m-rt documentation](https://docs.rs/cortex-m-rt/)
- [ARM Cortex-M33 Technical Reference Manual](https://developer.arm.com/documentation/100230/)
- [EFR32MG24 Reference Manual](https://www.silabs.com/documents/public/reference-manuals/efr32xg24-rm.pdf)
- [Rust Embedded Book - Memory Layout](https://docs.rust-embedded.org/book/start/memory.html)

## Troubleshooting Checklist

When encountering linker errors:

- [ ] Verify memory.x defines FLASH and RAM regions
- [ ] Check build.rs copies memory.x and device.x to OUT_DIR
- [ ] Confirm .cargo/config.toml only has `-Tlink.x` (NOT `-Tmemory.x`)
- [ ] Ensure cortex-m has `critical-section-single-core` feature
- [ ] Verify device.x has PROVIDE statements for all interrupts
- [ ] Run `cargo clean` and rebuild
- [ ] Check `cargo build -vv` output for actual linker command

## Architecture Notes

### ARM Cortex-M33 (ARMv8-M Mainline)

- 32-bit RISC processor
- Harvard architecture (separate instruction/data buses)
- Thumb-2 instruction set
- Hardware floating-point unit (FPU)
- TrustZone-M security extension support
- Nested Vectored Interrupt Controller (NVIC)
- Memory Protection Unit (MPU)
- Debug and trace capabilities

### EFR32MG24 Specific Features

- 78 MHz maximum CPU frequency
- 1536 KB Flash memory
- 256 KB RAM
- Hardware cryptographic accelerators
- Low energy modes (EM0-EM4)
- 2.4 GHz radio for Bluetooth, Zigbee, Thread, Matter
- Rich peripheral set (timers, USART, I2C, SPI, ADC, DAC, etc.)
