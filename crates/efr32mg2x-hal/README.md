# efr32mg2x-hal — HAL for the EFR32MG24 (XIAO MG24 Sense)

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

Twenty-one modules, each one a peripheral, a bus or a driver:

```
adc  board  bus  buzzer  clock  delay  display  dma  dwt  emu
i2c  microphone  pin  pwm  rtc  sensors  spi  sysrtc  timer  usart  wdog
```

The previous version of this file said "USART complete, I2C/SPI/Timers next". That was wrong in both directions: I2C,
SPI and the timers are here, and USART is here but has never been watched doing anything on the board.

**Runnable examples**: <https://github.com/bitscrafts/efr32mg2x-examples> — a clone, a build and a guarded
flash, one crate per layer (PAC, HAL, radio, board, Expansion Base). Not published to crates.io: an examples-only
crate has no binary, so it installs only with `--example`, and its crate page would render nothing.

## What is in the tree, measured

Twenty-one modules. All of them are implementations, not skeletons: the smallest has 2 public functions, the largest 18, and
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
efr32mg2x-hal = { version = "0.1.0", features = ["rt"] }
cortex-m = "0.7"
cortex-m-rt = "0.7"
panic-halt = "0.2"
```

### Minimal Example

```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg2x_hal::{
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
- **Build system and toolchain** — in the repository: `docs/02-guides/HAL-BUILD-SYSTEM.md`
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

Ten examples. The third column is the authority on what has actually been read back off the part, and the fourth says
whether the example needs the XIAO Expansion Base to mean anything.

| Example | What it shows | Watched on hardware | Needs the base |
| --- | --- | --- | --- |
| `hal_led_blink.rs` | Blink at the HAL's level: one pin primitive, no board. | 2026-11-15 — `hal: pin primitive drives PA7, 1 Hz` (mailbox, and the LED seen blinking) | no |
| `hal_rtt_hello.rs` | Deferred-formatting logging over RTT, and a panic that says what happened. | 2026-11-15 — heartbeats read off RTT with probe-rs while the core ran | no |
| `hal_imu_read.rs` | The IMU at the HAL's level: two pins, a power gate and a driver. | -- the run reported no record at all | no |
| `hal_mic_level.rs` | Shout at the board and the light gets brighter, with hardware PWM. | -- measured FAIL: `pwm: TIMER0 is not counting (clock?)` | no |
| `hal_timer_ticks.rs` | The tick timer, measured rather than assumed. | -- it reports steps, never a verdict: last one was `step2: sysrtc configured` | no |
| `hal_timer_compare.rs` | Does the compare channel fire? Measured against the counter itself. | -- it passed with a stale flag (`late` = a negative number); the flag is drained now and the example has not been re-measured since | no |
| `hal_emu_voltage.rs` | The EMU's voltage scale, changed and put back. | -- the same binary returned `ok , ok` on one boot and `ERR , ERR` on another | no, but not yet a reproducible measurement |
| `hal_wdog_feed.rs` | The watchdog, fed on purpose, and asked whether it took the lock. | -- writing `WDOG.CFG` is a precise data access error; see the file's own header | no |
| `hal_i2c_bitbang.rs` | The bit-banged transport: scan whatever two pins you give it. | -- the scan found nothing with the base removed | yes: OLED (0x3C), RTC (0x51) and BME680 (0x76) live on it |
| `hal_timer0_pwm.rs` | Buzzer through the HAL: a calibrated tone on the base's piezo. | -- the timer is configured; there was no piezo to hear | yes: the piezo |

A `--` is not a defect: the example compiles and is exercised, but nobody has read back an output that means anything, or
the hardware it talks to is missing. The same rule as the PAC crate: a date appears only when something was measured —
and the last column says when "something" needs the Expansion Base, which on 2026-11-15 was found faulty (a shorted rail
that heats and silences the module) and sent back. See `docs/01-getting-started/EXPANSION-BASE-SWD-FAULT-2026-11-15.md`.

Two of the ten are dated. The other eight say why not, in their own words, which is worth more than a column of tidy
dates that nobody can reproduce.

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

The status groups at the top of this file are the status; there is no second one to drift away from it. The
workspace-wide authority is `docs/03-development/STATUS.md` in the project repository.

## Staying out of the bootloader

The first 24 KiB of flash (0x08000000-0x08005FFF) hold the Gecko bootloader. An image that lands there bricks the
board. The linker script in this crate starts at **0x08006000**, and the flags are not optional either:

```toml
[target.thumbv8m.main-none-eabihf]
rustflags = ["-C", "link-arg=-Tlink.x", "-C", "link-arg=--nmagic"]
```

Without `--nmagic` the linker emits a small segment at 0x08000000 and some flash tools will happily write it. Read the
lowest load address of the image before flashing and refuse anything below 0x08006000.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://www.apache.org/licenses/LICENSE-2.0.txt))
- MIT license ([LICENSE-MIT](https://opensource.org/license/mit))

at your option.

---

**Version**: 0.1.0
**Last Updated**: November 14, 2026
**Status**: 21 modules in the tree; the measured/unmeasured split is at the top of this file
