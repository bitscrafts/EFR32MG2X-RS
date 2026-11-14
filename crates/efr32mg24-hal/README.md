# EFR32MG24 HAL (Hardware Abstraction Layer)

High-level, type-safe Hardware Abstraction Layer for EFR32MG24 microcontrollers.

> **Status**: 20 peripheral drivers in the tree. What is measured on the board, and what merely compiles, is listed
> below rather than summarised in a phase number that has been stale since June 2026.

## Overview

This crate provides high-level, safe abstractions over EFR32MG24 peripherals with embedded-hal v1.0 trait implementations.

**Target Hardware**: Seeed Studio XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
- **CPU**: ARM Cortex-M33 @ 78 MHz with FPU and TrustZone-M
- **Flash**: 1536 KB
- **RAM**: 256 KB
- **Radio**: 2.4 GHz (Bluetooth 5.3, Zigbee, Thread, Matter)

## What is in the tree

Twenty modules, each one a peripheral or a bus:

```
adc  board  bus  buzzer  clock  delay  display  dma  dwt  emu
i2c  microphone  pin  pwm  rtc  sensors  spi  sysrtc  timer  usart  wdog
```

The previous version of this file said "USART complete, I2C/SPI/Timers next". That was wrong in both directions: I2C,
SPI and the timers are here, and USART is here but has never been watched doing anything on the board.

## What is in the tree, measured

Twenty modules. All of them are implementations, not skeletons: the smallest has 2 public functions, the largest 18, and
the only `unimplemented!` in the crate is inside a doc example. What differs is how far each one has been taken.

**Watched on the board**

| module | what was measured |
|---|---|
| `bus` + `display` + `sensors` | `hal_env_display`: `T=31.68C P=1015hPa H=68.2%` on the OLED, matching the PAC twin within sensor noise |
| `pin` | the LED toggling, seen |
| `timer` + `buzzer` | `TOP` calibrated, `CC0_CFG=0x02`, counter climbing — registers confirmed, **no sound heard** (open trace or no buzzer fitted) |
| `rtc` | `base_rtc` runs |
| `dwt` | used as the time base by several examples that were run |

**Exercised by an example, not yet watched:** `board`, `microphone`, `sysrtc`, `delay`.

**Implemented, no example reaches it:** `adc`, `clock`, `dma`, `emu`, `i2c`, `pwm`, `spi`, `usart`, `wdog`.

Two of those deserve naming rather than a row in a table:

- `i2c` drives the hardware I2C0 peripheral, and that path has a **documented defect**: after `CMD.START` the state reads
  `0xd7` (BUSY|MASTER|TRANSMITTER|NACKED) and `PSTART` never clears. Every working I2C path in this repository is
  `bus::BitBangI2c`, which is a different module. See `docs/04-reference/I2C0-READ-DEFECT.md`.
- `sysrtc` had two wrong offsets corrected (CLKCTRL `0x240`, EN_SET `0x1004`) and accepts `EN`, but the counter is not
  read back correctly.



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

- **[src/clock/](src/clock/)** - Clock Management Unit (CMU)
- **[src/pin/](src/pin.rs)** - GPIO digital I/O
- **[src/delay.rs](src/delay/)** - SysTick-based delays

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

## Where the state lives

[docs/STATUS.md](docs/STATUS.md) carries the dated status; this README's own summary above is the short version. The
workspace-wide authority is `docs/03-development/STATUS.md` in the project repository.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/LICENSE-APACHE))
- MIT license ([LICENSE-MIT](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/LICENSE-MIT))

at your option.

---

**Version**: 0.1.1
**Last Updated**: November 14, 2026
**Status**: 20 drivers in the tree; the measured/unmeasured split is at the top of this file
