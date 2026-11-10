# efr32mg24-pac

Peripheral Access Crate (PAC) for Silicon Labs EFR32MG24 microcontrollers.

## Overview

This crate provides low-level register access to all peripherals of the EFR32MG24 family. It is automatically generated from Silicon Labs CMSIS-SVD files using svd2rust.

**Status**: ✅ Generated - Modular structure (2,633 files across 104 directories)

## Features

- **Device Support**: EFR32MG24B220F1536IM48 (primary target - XIAO MG24 Sense)
  - Additional variants can be added via feature flags
- **Register Access**: Direct memory-mapped register manipulation
- **Type Safety**: Rust type system ensures compile-time peripheral access safety
- **Interrupt Handling**: Full interrupt vector table support (with `rt` feature)
- **TrustZone**: Both secure (_S) and non-secure peripherals exposed

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
efr32mg24-pac = "0.1.0"
cortex-m = "0.7"
cortex-m-rt = "0.7"
```

### Basic Example

```rust
use efr32mg24_pac as pac;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    // Take peripheral singletons
    let peripherals = pac::Peripherals::take().unwrap();
    let core_peripherals = pac::CorePeripherals::take().unwrap();

    // Access GPIO peripheral
    let gpio = &peripherals.GPIO_S;

    // Configure PA5 as output
    gpio.porta_model.modify(|_, w| w.mode5().pushpull());

    // Set PA5 high
    gpio.porta_dout.write(|w| w.dout5().set_bit());

    loop {}
}
```

## Features Flags

### Device Variants

```toml
# Select specific device variant
efr32mg24-pac = { version = "0.1.0", features = ["efr32mg24a020f1536gm48"] }
```

Available variants (planned):
- `efr32mg24a010f1024im40` - 1024KB Flash, 128KB RAM, 40-pin
- `efr32mg24a010f1024im48` - 1024KB Flash, 128KB RAM, 48-pin
- `efr32mg24a010f1536gm48` - 1536KB Flash, 256KB RAM, 48-pin (GM package)
- `efr32mg24a020f1536gm48` - 1536KB Flash, 256KB RAM, 48-pin (default)
- ... (16 more variants)

### Runtime Features

- `rt` - Include runtime support (interrupt vectors, startup code)
- `secure` - Enable secure peripheral access (TrustZone)

## Peripheral List

### Core Peripherals
- SCRATCHPAD - Scratch pad registers
- EMU - Energy Management Unit
- CMU - Clock Management Unit
- HFRCO - High Frequency RC Oscillator
- FSRCO - Fast Startup RC Oscillator
- DPLL - Digital Phase-Locked Loop
- LFXO - Low Frequency Crystal Oscillator
- LFRCO - Low Frequency RC Oscillator
- ULFRCO - Ultra Low Frequency RC Oscillator
- MSC - Memory System Controller
- ICACHE - Instruction Cache

### Communication
- USART0 - Universal Synchronous/Asynchronous Receiver/Transmitter
- EUSART0/1/2 - Enhanced USART
- I2C0/1 - Inter-Integrated Circuit

### Timers
- TIMER0-4 - General Purpose Timers
- LETIMER - Low Energy Timer
- SYSRTC - System Real-Time Counter
- BURTC - Backup Real-Time Counter
- WDOG0/1 - Watchdog Timers
- PCNT - Pulse Counter

### Analog
- IADC0 - Incremental Analog-to-Digital Converter
- VDAC0/1 - Voltage Digital-to-Analog Converter
- ACMP0/1 - Analog Comparator

### Digital I/O
- GPIO - General Purpose Input/Output
- PRS - Peripheral Reflex System

### DMA
- LDMA - Linked Direct Memory Access

### System
- SYSCFG - System Configuration
- BURAM - Backup RAM
- GPCRC - General Purpose CRC
- DCDC - DC-DC Converter
- PDM - Pulse Density Modulation
- RFSENSE - RF Sense

### Security (Secure Vault)
- CRYPTOACC - Cryptographic Accelerator
- SEMAILBOX - Secure Mailbox

### Radio
- AGC - Automatic Gain Control
- FRC - Frame Controller
- MODEM - Modem
- PROTIMER - Protocol Timer
- RAC - Radio Controller
- RFCRC - Radio CRC
- SYNTH - Frequency Synthesizer

(See SVD files for complete peripheral list)

## Memory Map

### Flash
- Start: `0x0000_0000`
- End: `0x0818_0000`
- Size: 1536 KB (variant dependent)

### RAM
- Start: `0x2000_0000`
- End: `0x2003_FFFF`
- Size: 256 KB (variant dependent)

### Peripherals
- Secure peripherals: `0x4000_0000 - 0x4FFF_FFFF`
- Non-secure peripherals: `0x5000_0000 - 0x5FFF_FFFF`

## Module Structure

The PAC is organized into 104 separate modules for optimal IDE performance and incremental compilation:

- **2,633 Rust files** instead of one monolithic file
- Each peripheral in its own module directory
- Each register in its own file
- TrustZone variants (NS/S) separated

### Structure Example

```
efr32mg24-pac/src/
├── lib.rs                  # Main entry (~48 KB)
├── generic.rs              # Common traits
├── cmu_ns/                 # Clock Management (Non-Secure)
│   ├── mod.rs             # Module coordinator
│   ├── status.rs          # STATUS register
│   ├── clken0.rs          # CLKEN0 register
│   └── ... (28 more registers)
├── gpio_ns/                # GPIO (Non-Secure)
│   ├── porta_model.rs     # Port A mode register
│   └── ... (27 more registers)
└── ... (102 more peripherals)
```

### Benefits

- **IDE Performance**: Fast navigation and autocomplete
- **Incremental Compilation**: Only recompile changed peripherals
- **Git Diffs**: Readable peripheral-level changes
- **Code Review**: Easier to review register-specific changes

## Generation Process

```bash
# Generate modular PAC from SVD
cd efr32mg24-pac

# Step 1: Generate with svd2rust
svd2rust -i svd/EFR32MG24B220F1536IM48.svd \
         --output-dir src/ \
         --atomics \
         --target cortex-m \
         --generic-mod

# Step 2: Split into modules using form
form -i src/lib.rs -o src/

# Step 3: Format generated code
cargo fmt

# Step 4: Verify build
cargo build --target thumbv8m.main-none-eabihf
```

### Regenerating for Other Variants

```bash
# Use different SVD file for other device variants
svd2rust -i svd/EFR32MG24A020F1536GM48.svd \
         --output-dir src/ \
         --atomics \
         --target cortex-m \
         --generic-mod
form -i src/lib.rs -o src/
cargo fmt
```

## Dependencies

```toml
[dependencies]
cortex-m = "0.7.7"
vcell = "0.1.3"

[dependencies.cortex-m-rt]
version = "0.7.3"
optional = true

[build-dependencies]
# For device.x linker script generation
```

## Target Architecture

- **Target Triple**: `thumbv8m.main-none-eabihf`
- **CPU**: ARM Cortex-M33
- **FPU**: Single-precision (hardware)
- **DSP**: Enabled
- **TrustZone**: Supported

## Linker Script

The PAC will provide `device.x` linker script with memory layout:

```ld
MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 1536K
  RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}
```

Variants will have different memory sizes.

## Safety

This crate provides `unsafe` access to hardware registers. Users must ensure:
- No data races (use `critical-section` or RTIC)
- Correct peripheral configuration
- Proper clock/power management before peripheral access
- Respect TrustZone security boundaries

## Development

### Building from Source

```bash
# Clone repository
git clone https://github.com/bitscrafts/efr32-rs
cd efr32-rs/efr32mg24-pac

# Install target
rustup target add thumbv8m.main-none-eabihf

# Build
cargo build --target thumbv8m.main-none-eabihf
```

### Testing

PAC testing is limited without hardware:
- Compile tests verify code generation
- Hardware-in-loop tests require dev kit
- Register access tests can use QEMU (if supported)

## Related Crates

- [efr32mg24-hal](../efr32mg24-hal/) - High-level Hardware Abstraction Layer
- [cortex-m](https://crates.io/crates/cortex-m) - Core Cortex-M functionality
- [cortex-m-rt](https://crates.io/crates/cortex-m-rt) - Runtime support

## Resources

- [EFR32MG24 Reference Manual](https://www.silabs.com/documents/public/reference-manuals/)
- [SVD Specification](https://www.keil.com/pack/doc/CMSIS/SVD/html/)
- [svd2rust Documentation](https://docs.rs/svd2rust/)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.

---

**Status**: ✅ Generated - Modular Structure Complete
**Version**: 0.1.0-dev
**Device**: EFR32MG24B220F1536IM48 (XIAO MG24 Sense)
**Generated With**: svd2rust 0.37.1 + form
**Files**: 2,633 Rust files across 104 modules
**Last Updated**: December 4, 2025
