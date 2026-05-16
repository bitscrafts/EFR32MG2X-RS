# Board Support Package (BSP) Plan

**Date**: December 4, 2025
**Status**: Planning Phase

## Overview

This document outlines the plan for creating board-specific support packages separate from the generic EFR32MG24 HAL.

## Rationale

The HAL should remain **device-generic** for all EFR32MG24 variants, while BSPs provide board-specific:
- Pin definitions and aliases
- Crystal/oscillator frequencies
- Onboard peripheral drivers
- Board initialization
- Example code specific to the board

## Proposed Structure

```
EFR32MG2X-RS/
├── efr32mg24-pac/              # Device PAC (B220, A020, etc.)
├── efr32mg24-hal/              # Generic HAL for all EFR32MG24
│   ├── src/
│   │   ├── clock/              # Generic clock management
│   │   ├── gpio/               # Generic GPIO
│   │   ├── i2c/                # Generic I2C master
│   │   ├── usart/              # Generic USART
│   │   └── ...
│   └── examples/               # Generic HAL examples
│       ├── 01_clock.rs         # No board assumptions
│       ├── 02_delay.rs
│       ├── 03_gpio.rs
│       ├── 04_usart.rs
│       └── 05_i2c.rs
│
├── xiao-mg24-bsp/              # XIAO MG24 Sense BSP
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs              # Board init, pin definitions
│   │   ├── pins.rs             # Pin type aliases
│   │   ├── lsm6ds3.rs          # IMU driver
│   │   └── mic.rs              # Microphone driver
│   ├── examples/
│   │   ├── blinky.rs           # LED blink (knows LED is on PB2)
│   │   ├── button.rs           # Button read (knows button is PB3)
│   │   ├── imu.rs              # Read IMU via I2C1
│   │   ├── microphone.rs       # Read PDM microphone
│   │   └── radio.rs            # BLE/Matter example
│   └── README.md
│
└── other-board-bsp/            # Future boards
    └── ...
```

## XIAO MG24 Sense BSP

### Hardware Specifications

- **Chip**: EFR32MG24B220F1536IM48-B
- **Clock**: 39 MHz HFXO, 32.768 kHz LFXO
- **LED**: Single RGB LED (shared pin PB2)
- **Button**: User button on PB3
- **IMU**: LSM6DS3TR-C on I2C1 (addr 0x6B)
  - I2C1 SCL: PC6
  - I2C1 SDA: PC7
- **Microphone**: PDM microphone
- **Flash**: External QSPI flash
- **USB**: USB-C connector

### Pin Definitions

```rust
// xiao-mg24-bsp/src/pins.rs

use efr32mg24_hal::gpio::{self, Pin};
use efr32mg24_hal::pac;

pub struct Pins {
    pub led: Pin<gpio::PortB, 2, gpio::Output<gpio::PushPull>>,
    pub button: Pin<gpio::PortB, 3, gpio::Input<gpio::Pull>>,

    // I2C1 for IMU
    pub i2c1_scl: Pin<gpio::PortC, 6, gpio::Alternate>,
    pub i2c1_sda: Pin<gpio::PortC, 7, gpio::Alternate>,

    // Other pins...
}

impl Pins {
    pub fn new(gpio: pac::GpioS) -> Self {
        let gpio = gpio.split();

        Self {
            led: gpio.portb.pb2.into_push_pull_output(),
            button: gpio.portb.pb3.into_pull_up_input(),
            i2c1_scl: gpio.portc.pc6.into_alternate(),
            i2c1_sda: gpio.portc.pc7.into_alternate(),
        }
    }
}
```

### Board Initialization

```rust
// xiao-mg24-bsp/src/lib.rs

pub struct Board {
    pub pins: Pins,
    pub clocks: FrozenClocks,
    pub delay: Delay,
    pub imu: Option<Lsm6ds3>,
}

impl Board {
    pub fn new(cp: cortex_m::Peripherals, dp: pac::Peripherals) -> Self {
        // Configure clocks with XIAO MG24's 39 MHz crystal
        let clocks = Clocks::new(dp.cmu_s, ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)),
            lfxo: Some(LfxoConfig::default()),
        }).freeze();

        let delay = Delay::new(cp.SYST, &clocks);
        let pins = Pins::new(dp.gpio_s);

        // Initialize IMU if present
        let imu = Lsm6ds3::new(dp.i2c1_s, &clocks, &pins);

        Self { pins, clocks, delay, imu }
    }
}
```

### BSP Examples

Board-specific examples that use the BSP:

```rust
// xiao-mg24-bsp/examples/blinky.rs

use xiao_mg24_bsp::Board;

fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut board = Board::new(cp, dp);

    loop {
        board.pins.led.set_high();
        board.delay.delay_ms(500);
        board.pins.led.set_low();
        board.delay.delay_ms(500);
    }
}
```

```rust
// xiao-mg24-bsp/examples/imu.rs

use xiao_mg24_bsp::Board;

fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut board = Board::new(cp, dp);

    if let Some(imu) = &mut board.imu {
        let who_am_i = imu.who_am_i().unwrap();
        // Should be 0x69 for LSM6DS3

        loop {
            let (accel, gyro) = imu.read_all().unwrap();
            // Process IMU data
            board.delay.delay_ms(10);
        }
    }

    loop {}
}
```

## Other BSP Candidates

### BRD4186C Radio Board

- **Chip**: EFR32MG24B220F1536GM48
- **Clock**: 38.4 MHz HFXO
- **LEDs**: LED0 (PA4), LED1 (PA5)
- **Buttons**: BTN0 (PB1), BTN1 (PB3)
- **VCOM**: USART0 for serial console
- **Extension**: Compatibility with EXP headers

### SparkFun Thing Plus MGM240P

- **Chip**: EFR32MG24 module
- **Form factor**: Feather-compatible
- **Qwiic connector**: I2C for sensors
- **microSD**: SPI storage
- **LiPo charger**: Battery management

## HAL vs BSP Responsibilities

| Feature | HAL | BSP |
|---------|-----|-----|
| Peripheral drivers | ✅ Generic | ❌ |
| Pin configuration | ✅ Generic GPIO API | ✅ Board pin aliases |
| Clock frequencies | ✅ Configurable | ✅ Board defaults |
| embedded-hal traits | ✅ | ❌ (uses HAL) |
| Board init | ❌ | ✅ |
| Sensor drivers | ❌ | ✅ Board-specific |
| Examples | ✅ Generic | ✅ Board-specific |

## Migration Path

### Current (HAL with board assumptions)

```rust
// Assumes XIAO MG24 (39 MHz)
let clocks = Clocks::new(dp.cmu_s, ClockConfig {
    hfxo: Some(HfxoConfig::new(39_000_000)),
    ...
}).freeze();
```

### Future (Generic HAL + BSP)

**Using HAL directly** (for custom boards):
```rust
// User must know their board's crystal frequency
let clocks = Clocks::new(dp.cmu_s, ClockConfig {
    hfxo: Some(HfxoConfig::new(MY_CRYSTAL_FREQ)),
    ...
}).freeze();
```

**Using BSP** (for supported boards):
```rust
// BSP handles all board-specific details
let board = xiao_mg24_bsp::Board::new(cp, dp);
// board.clocks, board.pins, board.delay, board.imu all ready to use
```

## Implementation Priority

1. **Phase 1** (Current): Keep HAL development generic
   - ✅ I2C module is already generic
   - Document board requirements in examples
   - Use generic examples only

2. **Phase 2** (After HAL Tier 2 complete): Create XIAO MG24 BSP
   - Separate `xiao-mg24-bsp` crate
   - Board initialization
   - Pin definitions
   - LSM6DS3 IMU driver

3. **Phase 3** (Future): Additional BSPs
   - BRD4186C Radio Board BSP
   - SparkFun Thing Plus BSP
   - Community-contributed BSPs

## Benefits

- **HAL Reusability**: Works with any EFR32MG24 board
- **Clear Separation**: Device drivers vs board support
- **Easy Testing**: Generic HAL can be tested independently
- **Community**: Others can create BSPs for their boards
- **Maintainability**: Changes to one board don't affect others

## References

- [Rust Embedded Book - Board Support Crates](https://docs.rust-embedded.org/book/start/hardware.html#board-support-crate)
- [STM32 BSP examples](https://github.com/stm32-rs/stm32f4xx-hal/tree/master/examples)
- [nRF BSPs](https://github.com/nrf-rs/nrf-hal/tree/master/boards)
- [RP2040 BSPs](https://github.com/rp-rs/rp-hal-boards)
