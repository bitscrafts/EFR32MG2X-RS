# efr32mg2x-hal — HAL for the EFR32MG24 (XIAO MG24 Sense)

High-level, type-safe Hardware Abstraction Layer for EFR32MG24 microcontrollers.

> **Status**: 20 peripheral drivers in the tree. What is measured on the board, and what merely compiles, is listed
> below rather than summarised in a phase number that has been stale since June 2026.

## Overview

This crate provides high-level, safe abstractions over EFR32MG24 peripherals with embedded-hal v1.0 trait implementations.

### Where this crate sits

```
efr32mg2x-pac          the register access layer: one type per peripheral, offsets from the SVD
      |
efr32mg2x-hal   <--    this crate: peripherals, buses and drivers, no board facts
      |
efr32mg2x-xiao-bsp     the board: which pin the LED is on, which pins power the IMU and the microphone
```

### What this crate uses from `efr32mg2x-pac`

Read from the source, so it cannot drift:

| what it takes | where it is used |
|---|---|
| `CmuNs` | `src/mvp/mod.rs` |
| `CmuS` | `src/adc/mod.rs`, `src/clock/clocks.rs`, `src/clock/frozen.rs` (+3 more) |
| `EmuS` | `src/board.rs`, `src/emu/mod.rs` |
| `Eusart0S` | `src/spi/mod.rs` |
| `Eusart1S` | `src/spi/mod.rs` |
| `GpioNs` | `src/board.rs` |
| `GpioS` | `src/board.rs` |
| `Hfxo0Ns` | `src/clock/clocks.rs` |
| `I2c0Ns` | `src/board.rs`, `src/i2c/async_impl.rs`, `src/i2c/mod.rs` |
| `I2c0S` | `src/i2c/async_impl.rs`, `src/i2c/mod.rs` |
| `I2c1S` | `src/i2c/mod.rs` |
| `Iadc0S` | `src/adc/mod.rs`, `src/board.rs`, `src/microphone.rs` |
| `Interrupt` | `src/sysrtc.rs` |
| `LdmaS` | `src/dma/mod.rs` |
| `LdmaxbarS` | `src/dma/mod.rs` |
| `MvpNs` | `src/board.rs`, `src/mvp/mod.rs` |
| `Peripherals` | `src/adc/mod.rs`, `src/board.rs`, `src/buzzer.rs` (+13 more) |
| `Sysrtc0Ns` | `src/rtc/mod.rs`, `src/sysrtc.rs` |
| `Timer0S` | `src/timer/mod.rs` |
| `Usart0S` | `src/spi/mod.rs`, `src/usart/mod.rs` |
| `Wdog0Ns` | `src/board.rs`, `src/wdog/mod.rs` |
| `cmu_s` | `src/board.rs` |
| `emu_ns` | `src/sleep.rs` |
| `gpio_s` | `src/board.rs`, `src/bus/bitbang.rs`, `src/pin.rs` |
| `i2c0_ns` | `src/i2c/async_impl.rs` |
| `iadc0_s` | `src/board.rs`, `src/microphone.rs` |
| `timer0_s` | `src/pwm.rs` |


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

Twenty-six public modules (`pub mod` in `src/lib.rs`) and twenty examples. Four of them carry a date, and a date means a
record was read back off the part — not that the code compiled.

**Watched on the board (2026-11-15)**

| example | what came back |
|---|---|
| `hal_led_blink` | `PASS hal: pin primitive drives PA7, 1 Hz`, and the LED was watched blinking |
| `hal_i2c_bitbang` | `PASS scan n=3 3c 51 76` — the panel, the RTC and the BME680 answered (measured with the Expansion Base fitted) |
| `hal_rtt_hello` | heartbeats 0..8 read off RTT with `probe-rs` while the core ran |

**Watched on the board (2026-11-23)**

| example | what came back |
|---|---|
| `hal_i2c_recovery` | `PASS i2c-rec: antes=0x6a erro=1 depois=0x6a wr=1 rd6=1 nz=1 volta=0x40` — a refusal on an address where nothing is wired, the bus still answering after it, a plain `write`, and a six-byte IMU burst carrying real data |

**Ran, and the record says it did not work**

| example | record |
|---|---|
| `hal_mic_level` | `FAIL pwm: TIMER0 is not counting (clock?)` — the level is computed, the hardware PWM path is not |
| `hal_wdog_feed` | the `CFG` write faults at `0x4b00400c` and `0x5b00400c`; reads work in both windows. See `src/wdog/mod.rs` |
| `hal_emu_voltage` | `ok , ok` on one boot and `ERR , ERR` on another, same binary — not deterministic, so no date |
| `hal_timer_compare` | passed with a stale compare flag (`late = 2^32-3200`); the flag is now drained first, and it has not been re-measured since |
| `hal_imu_read` | no record: the example writes nothing the harness can read |
| `hal_timer_ticks` | step records only (`step2: sysrtc configured`), never a verdict |

**No example names these:** `adc`, `clock`, `display`, `dma`, `i2c`, `rtc`, `spi`, `usart`. (`timer` and `pwm` are reached
through `Board::buzzer_on` / `Board::pwm_on` rather than named directly.)

Two of those deserve naming rather than a row in a table:

- `i2c` drives the hardware I2C0 peripheral, and on this board it reads and writes: `hal_i2c_hardware_imu` and
  `hal_i2c_recovery` are the examples that reach it. What looked like a silicon defect was two driver defects — the
  last byte of a reception has to be read with NACK, and the flags are only cleared through the `IF_CLR` alias — and
  both are in `docs/04-reference/I2C0-READ-DEFECT.md` and `docs/04-reference/I2C-RECOVERY-STATE.md`. `bus::BitBangI2c`
  is a different bus, for the Expansion Base's PC4/PC5 pair; I2C0 reaches one pin pair at a time.
- `clock` carries a bricking warning in its own source (`clock/clocks.rs`): an earlier implementation switched SYSCLK to
  HFXO before the HFXO was running, which stopped the CPU clock and locked the debug access port. No example touches it yet,
  on purpose. See `docs/01-getting-started/BRICKING_INCIDENT_2026-06-29.md`.

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
    pac,
    prelude::*,
};
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::pin::Port;
use panic_halt as _;

#[entry]
fn main() -> ! {
    // `Board` takes the peripherals once and hands out the pieces: the clocks, the pins, the delay.
    let mut board = Board::take().expect("peripherals taken twice");
    let mut delay = board.delay();

    // PA7, active low: the LED the board ships with.
    let led = board.output_on(Port::A, 7, true);

    loop {
        led.on();
        delay.delay_ms(500);
        led.off();
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

20 examples, **3 of them watched on the part**. A `--` in the last column means nobody
has read a value back off the hardware for it; it does not mean the code is unverified and it does not
mean it works.

| example | what it shows | watched on hardware |
|---|---|---|
| `hal_adc_read.rs` | The IADC on this part converts one channel at a time into a 12-bit code. The hard part of an ADC | -- |
| `hal_dma_stop.rs` | Does `stop()` really disable the DMA channel? | -- |
| `hal_em2_sleep.rs` | EM2: dez sonecas de um segundo, com o SYSRTC a acordar. | -- |
| `hal_emu_voltage.rs` | The EMU's voltage scale, changed and put back, reported rather than assumed. | -- |
| `hal_hfxo.rs` | HFXO: correr do cristal, e provar que o relógio do sistema o seguiu. | -- |
| `hal_i2c_async.rs` | O IMU lido pelo caminho **assíncrono** do I2C, num executor mínimo. | -- |
| `hal_i2c_bitbang.rs` | The bit-banged transport: scan whatever two pins you give it. | 2026-11-15 |
| `hal_i2c_hardware_imu.rs` | The IMU on the hardware I2C bus: routed pins, a power gate, one byte back. | -- |
| `hal_i2c_recovery.rs` | After a NACK, does the bus still work? | -- |
| `hal_imu_read.rs` | The IMU at the HAL's level: two pins, a power gate and a driver. | -- |
| `hal_led_blink.rs` | Blink at the HAL's level: one pin primitive, no board. | 2026-11-15 |
| `hal_mic_level.rs` | Shout at the board and the light gets brighter — with hardware PWM. | -- |
| `hal_mvp_bench.rs` | Does the MVP pay for itself? One number, measured both ways. | -- |
| `hal_rtt_hello.rs` | Deferred-formatting logging over RTT, and a panic that says what happened. | 2026-11-15 |
| `hal_sysrtc_wide.rs` | O tick de 64 bits do SYSRTC, o comparador a acordar o núcleo, e um relógio independente a confirmá-lo. | -- |
| `hal_sysrtc_wrap.rs` | A volta do contador de 32 bits, exercida em milissegundos em vez de 36,4 horas. | -- |
| `hal_timer0_pwm.rs` | Buzzer through the HAL: a calibrated tone on the base's piezo. | -- |
| `hal_timer_compare.rs` | Does the compare channel fire? Measured against the counter itself, with a wait long enough to matter. | -- |
| `hal_timer_ticks.rs` | The tick timer, measured rather than assumed. | -- |
| `hal_wdog_feed.rs` | The watchdog, fed on purpose, and asked whether it took the lock. | -- |


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
