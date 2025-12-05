# EFR32MG24 HAL (Hardware Abstraction Layer)

High-level, type-safe Hardware Abstraction Layer for EFR32MG24 microcontrollers.

> ✅ **Status**: Phase 5 Tier 2 - USART Complete, I2C/SPI/Timers Next

## Overview

This crate provides high-level, safe abstractions over EFR32MG24 peripherals with embedded-hal v1.0 trait implementations.

**Target Hardware**: Seeed Studio XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
- **CPU**: ARM Cortex-M33 @ 78 MHz with FPU and TrustZone-M
- **Flash**: 1536 KB
- **RAM**: 256 KB
- **Radio**: 2.4 GHz (Bluetooth 5.3, Zigbee, Thread, Matter)

## Status

**Current Version**: 0.1.0
**Phase 5 Tier 2** - USART complete, I2C/SPI/Timers next

### Implemented Peripherals

**Tier 1 (Complete):**
- ✅ **Clock Management (CMU)** - HFXO/HFRCO configuration with hardware register access
- ✅ **GPIO** - Type-safe pin modes with hardware register manipulation
- ✅ **Delay** - SysTick-based delays integrated with CMU

**Tier 2 (In Progress):**
- ✅ **USART0** - Serial communication with embedded-hal-nb traits
- ⏳ I2C (I2C0, I2C1) - Planned next
- ⏳ SPI (USART in SPI mode) - Planned
- ⏳ Timer (TIMER0-4) and PWM - Planned

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
efr32mg24-hal = { path = "../efr32mg24-hal", features = ["rt"] }
cortex-m = "0.7"
cortex-m-rt = "0.7"
panic-halt = "0.2"
```

### Minimal Example

```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_hal::{
    clock::{Clocks, ClockConfig, HfxoConfig, LfxoConfig},
    delay::Delay,
    gpio::GpioExt,
    pac,
    prelude::*,
};
use panic_halt as _;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks with XIAO MG24's 39 MHz crystal
    let clocks = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)),
            lfxo: Some(LfxoConfig::default()),
        }
    ).freeze();

    // Create delay provider
    let mut delay = Delay::new(cp.SYST, &clocks);

    // Split GPIO into individual pins
    let gpio = dp.GPIO_S.split();
    let mut led = gpio.portb.pb2.into_push_pull_output();

    loop {
        led.set_high().unwrap();
        delay.delay_ms(500);
        led.set_low().unwrap();
        delay.delay_ms(500);
    }
}
```

## Documentation

### Getting Started

- **[docs/](docs/)** - Complete HAL documentation index
- **[docs/BUILD_SYSTEM.md](docs/BUILD_SYSTEM.md)** - Build system and toolchain setup
- **[docs/LINKER_SETUP.md](docs/LINKER_SETUP.md)** - Linker script configuration

### Module Documentation

- **[src/clock/README.md](src/clock/README.md)** - Clock Management Unit (CMU)
- **[src/gpio/README.md](src/gpio/README.md)** - GPIO digital I/O
- **[src/delay/README.md](src/delay/README.md)** - SysTick-based delays

### API Documentation

Generate complete API documentation:

```bash
cargo doc --no-deps --features rt --open
```

## Examples

Three working examples demonstrating all implemented peripherals:

```bash
# Build all examples
cargo build --examples --features rt

# Build specific example
cargo build --example 01_clock --features rt

# Flash to hardware (requires probe-rs)
cargo run --example 03_gpio --features rt
```

### Available Examples

1. **01_clock.rs** - Clock configuration with HFXO/HFRCO/LFXO
2. **02_delay.rs** - Millisecond/microsecond/nanosecond delays
3. **03_gpio.rs** - LED control, button input, type-safe pins

## Building

### Prerequisites

```bash
# Install ARM Cortex-M33 target
rustup target add thumbv8m.main-none-eabihf

# Install probe-rs for flashing (optional)
cargo install probe-rs-tools
```

### Build Commands

```bash
# Check code
cargo check --features rt

# Build library
cargo build --features rt --release

# Build examples
cargo build --examples --features rt

# Run tests
cargo test --features rt
```

## Features

- **Safe abstractions**: Zero unsafe code in public API
- **Type-safe GPIO**: Pin modes enforced at compile time
- **embedded-hal v1.0**: Compatible with embedded-hal ecosystem
- **Hardware register access**: Direct manipulation of EFR32MG24 registers
- **Critical sections**: Safe atomic operations via cortex-m
- **Modular design**: Clean separation of concerns, all files <400 lines

## Development Status

See [docs/STATUS.md](docs/STATUS.md) for detailed implementation status and [docs/PHASE2_PLAN.md](docs/PHASE2_PLAN.md) for the Phase 2 development plan.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.

---

**Version**: 0.1.0
**Last Updated**: December 4, 2025
**Status**: Phase 2 Complete - Tier 1 Peripherals Implemented
