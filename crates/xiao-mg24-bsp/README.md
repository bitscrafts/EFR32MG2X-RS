# xiao-mg24-bsp

Board Support Package for **Seeed Studio XIAO MG24 Sense** (EFR32MG24B220F1536IM48-B).

**Runnable examples**: <https://github.com/bitscrafts/efr32mg2x-examples> — the same crates, a clone, a build and a
guarded flash, one crate per layer (PAC, HAL, radio, board, Expansion Base). They are not published to crates.io: an
examples-only crate has no binary, so it installs only with `--example`, and its crate page would render nothing.

## Where this crate sits

```
efr32mg2x-pac                   register access: one type per peripheral, offsets from the SVD
      |
efr32mg2x-hal                   peripherals, buses and drivers, no board facts
      |
efr32mg2x-xiao-bsp   <--        this crate: which pin the LED is on, which pins power the IMU
                                and the microphone, and the clock the module runs at
```

### What this crate uses from `efr32mg2x-hal`

Read from the source, so it cannot drift:

| what it takes | where it is used |
|---|---|
| `board` | `src/module.rs` |
| `bus` | `src/module.rs` |
| `delay` | `src/module.rs` |
| `microphone` | `src/module.rs` |
| `mvp` | `src/module.rs` |
| `pin` | `src/module.rs` |

### What this crate uses from `efr32mg2x-pac`

Nothing directly. This crate reaches `efr32mg2x-pac` through the layer above it; if a type
from `efr32mg2x-pac` ever appears in this crate's source, the layering has been broken.


### What this crate uses from `efr32mg2x-hal`

Read from the source, so it cannot drift:

| what it takes | where it is used |
|---|---|
| `board` | `src/module.rs` |
| `bus` | `src/module.rs` |
| `delay` | `src/module.rs` |
| `microphone` | `src/module.rs` |
| `mvp` | `src/module.rs` |
| `pin` | `src/module.rs` |

### What this crate uses from `efr32mg2x-pac`

Nothing directly. This crate reaches `efr32mg2x-pac` through the layer above it; if a type
from `efr32mg2x-pac` ever appears in this crate's source, the layering has been broken.


## Overview

This crate provides high-level abstractions for the XIAO MG24 Sense development board, including:

- Pin definitions for all 19 XIAO connector pins
- Board initialization with default clock configuration
- Pre-configured peripheral instances
- Examples demonstrating board features

## Hardware Specifications

**XIAO MG24 Sense**:
- **MCU**: EFR32MG24B220F1536IM48-B
- **CPU**: ARM Cortex-M33 @ 78 MHz with FPU and TrustZone-M
- **Memory**: 1536 KB Flash, 256 KB RAM
- **Radio**: 2.4 GHz (Bluetooth 5.3, Zigbee, Thread, Matter capable)
- **Package**: 48-pin QFN (IM48)
- **Form Factor**: XIAO (21mm × 17.5mm)

**On-board Peripherals**:
- 1× RGB LED (WS2812B, serial control)
- 1× User LED (PB02, active low)
- 1× User button (PB01, active low)
- 1× 6-axis IMU (LSM6DS3TR-C)
- 1× Microphone (MSM381ACT001 — analogue output on PC09/MIC_ADC, enable on PC08/MIC_EN)

  Corrected 2026-11-17 from the Seeed schematic. This line said `MSM261D3526H1CPM`, which is a **PDM** part: a PDM
  microphone has a clock and a data line and cannot drive an ADC pin, so it could not have been the fitted part. The
  analogue one is what makes the IADC path in `efr32mg2x-hal`'s `microphone.rs` possible, and its datasheet sensitivity
  (-38 dBV/Pa at 1 kHz) is the whole basis of the acoustic calibration in the dosimeter app.
- 1× Battery connector (JST 1.25mm)
- 1× USB-C connector

## Pin Mapping

```text
XIAO MG24 Sense Pin Layout:

       USB-C
    ┌─────────┐
D0  │ PA05    │ 5V
D1  │ PA06    │ GND
D2  │ PC00    │ 3V3
D3  │ PC01    │ PB02  D10
D4  │ PC02    │ PB03  D9
D5  │ PC03    │ PB04  D8
D6  │ PB00    │ PB01  D7
    └─────────┘
```

### Pin Capabilities

| Pin | GPIO  | Alternative Functions |
|-----|-------|-----------------------|
| D0  | PA05  | USART0 TX, I2C0 SDA   |
| D1  | PA06  | USART0 RX, I2C0 SCL   |
| D2  | PC00  | SPI0 MOSI             |
| D3  | PC01  | SPI0 MISO             |
| D4  | PC02  | SPI0 SCK              |
| D5  | PC03  | SPI0 CS               |
| D6  | PB00  | TIMER0 CC0, PWM       |
| D7  | PB01  | TIMER0 CC1, BUTTON    |
| D8  | PB04  | ADC0 CH0              |
| D9  | PB03  | ADC0 CH1              |
| D10 | PB02  | PWM, LED              |

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
xiao-mg24-bsp = "0.1"
cortex-m = "0.7"
cortex-m-rt = "0.7"
```

### Blink Example

```rust
#![no_std]
#![no_main]

use xiao_mg24_bsp::Board;
use cortex_m_rt::entry;
use embedded_hal::delay::DelayNs;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut board = Board::take().unwrap();
    let mut delay = board.delay();

    // Configure LED pin (PB02, active low)
    let mut led = board.pins.led.into_output();

    loop {
        led.set_low();  // LED on (active low)
        delay.delay_ms(500);
        led.set_high(); // LED off
        delay.delay_ms(500);
    }
}
```

### USART Example

```rust
use xiao_mg24_bsp::{Board, hal};
use hal::usart::{Usart0, UsartConfig};
use embedded_hal_nb::serial::Write;

let mut board = Board::take().unwrap();

// Configure USART0 on D0 (TX) and D1 (RX)
let usart = Usart0::new(
    board.peripherals.usart0,
    UsartConfig::default().baudrate(115200),
    &board.clocks,
);

// Send data
for byte in b"Hello, XIAO!\n" {
    nb::block!(usart.write(*byte)).unwrap();
}
```

## Clock Configuration

Default configuration:
- **HFXO**: 39 MHz (external crystal)
- **SYSCLK**: 78 MHz (HFXO × 2 via PLL)
- **HCLK**: 78 MHz (AHB bus clock)
- **PCLK**: 78 MHz (peripheral clock)

Custom clock configuration:

```rust
use xiao_mg24_bsp::{Board, hal::clock::ClockConfig};

let config = ClockConfig::hfxo(39_000_000)
    .sysclk_frequency(78_000_000)
    .pclk_prescaler(2);  // PCLK @ 39 MHz

let board = Board::with_config(config).unwrap();
```

## Radio Support

### RAIL Foundation (Phase D1) ✅ **COMPLETE**

Low-level RAIL (Radio Abstraction Interface Layer) FFI bindings are now available:

```toml
[dependencies]
xiao-mg24-bsp = { version = "0.1", features = ["radio"] }
```

**Available**:
- ✅ **RAIL FFI bindings** (`efr32mg24-rail-sys`) - Production-ready
  - Platform-independent build system
  - SHA256 blob verification
  - 3,974 lines of auto-generated FFI bindings
  - Comprehensive documentation

**Next Steps** (Future Releases):
- **BLE (Bluetooth 5.3)**: Phase D2 (planned)
- **Zigbee 3.0**: Phase D3 (planned)
- **Thread 1.3**: Phase D4 (optional)
- **Matter**: Phase D5 (optional)

## Examples

Build and run examples:

```bash
# Blink LED
cargo run --example blink --release

# USART echo
cargo run --example usart_echo --release

# I2C sensor
cargo run --example i2c_imu --release
```

## Flashing

Use `probe-rs` for flashing:

```bash
cargo install probe-rs-tools

# Flash via CMSIS-DAP (built-in on XIAO)
probe-rs run --chip EFR32MG24B220F1536IM48 --release
```

## Resources

- [XIAO MG24 Sense Wiki](https://wiki.seeedstudio.com/xiao_mg24_getting_started/)
- [EFR32MG24 Reference Manual](https://www.silabs.com/documents/public/reference-manuals/efr32xg24-rm.pdf)
- [Schematic](https://files.seeedstudio.com/wiki/XIAO-MG24-Sense/XIAO-MG24-Sense-v1.0_SCH&PCB_PDF_20240606.zip)

## License

MIT OR Apache-2.0

## See Also

- [efr32mg24-rail-sys](../efr32mg24-rail-sys) - RAIL FFI bindings (Phase D1 complete)
- [efr32mg2x-hal](../efr32mg2x-hal) - Hardware Abstraction Layer
- [efr32mg2x-pac](../efr32mg2x-pac) - Peripheral Access Crate

---

**Version**: 0.1.0
**Target Board**: Seeed Studio XIAO MG24 Sense
**Status**: Phase D1 (RAIL Foundation Complete)
**Last Updated**: 2025-12-28

<!-- META: last_updated=2025-12-28 version=0.1.0 phase=D1-RAIL status=active -->
