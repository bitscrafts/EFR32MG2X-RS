# adc

Analog-to-Digital Converter (IADC) Driver

## What it is

Analog-to-Digital Converter (IADC) Driver

This module provides ADC functionality using the IADC0 peripheral
for the EFR32MG24 microcontroller.

# Hardware Registers

- **EN**: Enable register
- **CTRL**: Control register
- **CMD**: Command register (start conversion)
- **STATUS**: Status flags
- **CFG0**: Configuration register for single conversion
- **SINGLE**: Single conversion configuration
- **SINGLEFIFODATA**: Single conversion result
- **IF/IEN**: Interrupt flags and enable

# Features

- Single-shot ADC conversion
- Configurable resolution (12-bit)
- Multiple reference voltage sources (VBGR, VDD)
- Timeout-protected conversion
- Critical section protection for RTOS safety

# Example

```no_run
use efr32mg2x_hal::{
clock::{Clocks, ClockConfig, HfxoConfig},
adc::{Adc, Config, Channel, Reference},
pac,
};

let dp = pac::Peripherals::take().unwrap();

// Configure clocks
let clocks = Clocks::new(dp.cmu_s, ClockConfig {
hfxo: None,
lfxo: None,
}).freeze();

// Create ADC with default configuration (12-bit, VBGR reference)
let mut adc = Adc::new(dp.iadc0_s, Config::default(), &dp.cmu_s);

// Read from channel 0
let value = adc.read(Channel::Ch0).unwrap();
```

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Adc` | struct | ADC instance. |
| `Channel` | enum | — |
| `Config` | struct | — |
| `Error` | enum | — |
| `Reference` | enum | — |
| `Resolution` | enum | — |
| `Trigger` | enum | — |
| `arm_single` | fn | Arm the single queue, so that the next trigger converts once. |
| `disable` | fn | Disable the ADC peripheral This disables the IADC hardware module to save power. |
| `new` | fn | Creates a new ADC instance # Arguments * `adc` - IADC peripheral instance * `config` - ADC configuration * `clocks` - Frozen clock configuration # Example ```no_run # use efr32mg2x_hal::{adc::{Adc, Config}, clock::{Clocks, ClockConfig, HfxoConfig}, pac}; # let dp = pac::Peripherals::take().unwrap(); # let clocks = Clocks::new(dp.cmu_s, ClockConfig { #     hfxo: None, #     lfxo: None, # }).freeze(); let adc = Adc::new(dp.iadc0_s, Config::default(), &dp.cmu_s); ```. |
| `read` | fn | Read one single-ended channel, always on **Port A**. |
| `read_on` | fn | Perform a single ADC conversion on the specified channel This function configures the channel, starts the conversion, waits for completion (with timeout), and returns the result. |
| `reference` | fn | Get the configured reference voltage source. |
| `select_input` | fn | Point the next single conversion at `pin` on `port`, without starting one. |
| `take_single` | fn | Pop one finished conversion, waiting at most `polls` reads of `STATUS` for it. |
| `trigger` | fn | What starts a conversion right now. |
| `use_prs_trigger` | fn | Take one conversion from PRS: a positive edge on asynchronous `channel` converts once. |
| `use_software_trigger` | fn | Go back to the CPU-started path: `CMD.SINGLESTART` converts once. |
| `with_reference` | fn | Set the reference voltage source. |
| `with_resolution` | fn | Set the ADC resolution. |

## Registers it touches

`CmuS`, `Iadc0S`, `Peripherals`

## What was measured

```
    image: reading `IADC0_S` (`0x4900_4000`) with *its* gate clear faults the same way (`bfar=0x49004000`), which is what
    adc: gnd=0 ch0=0 ref=VBGR                                              PASS   single-shot, ground channel
    adcprs: trig=prs0 refused=1 armA=1 imm=0 armB=0 prs=8 v0=0x0000026f v7=0x000002a7 quiet=0 noarm=0
    adcprs: trig=prs0 refused=1 armA=1 imm=0 armB=0 prs=8 v0=0x0000026c v7=0x000002a9 quiet=0 noarm=0
```

Controls and the full story: [ACMP-LADDER-2026-11-27.md](../../../../docs/04-reference/ACMP-LADDER-2026-11-27.md), [HAL-REVIEW-BRIEF-2026-11-25.md](../../../../docs/04-reference/HAL-REVIEW-BRIEF-2026-11-25.md), [IADC-PRS-TRIGGER-2026-11-26.md](../../../../docs/04-reference/IADC-PRS-TRIGGER-2026-11-26.md), [PRS-GPIO-PRODUCER-2026-11-26.md](../../../../docs/04-reference/PRS-GPIO-PRODUCER-2026-11-26.md)

## What is not proven

- C -- advanced | complete to stage 2 | `adc`, `dma`, `emu`, `rtc`/`sysrtc`, `wdog`, `gpcrc`, `mvp`, `letimer`, `prs`; DMA stages 3 and 4 deferred |. (PLAN.md (`../../../../docs/03-development/PLAN`))
- documentation | **B** | `deny(missing_docs)` now holds on all 27 modules (the five exemptions are gone, and the module that needed them -- `cores::i2c::sim` -- is test-only). (HAL-MATURITY-2026-11-25.md (`../../../../docs/04-reference/HAL-MATURITY-2026-11-25`))
- PRS routing | No hardware trigger path between timers, the IADC and the radio; needed for 802.15.4 timing. (HAL-RADIO-BLE-REVIEW-2026-11-17.md (`../../../../docs/04-reference/HAL-RADIO-BLE-REVIEW-2026-11-17`))

## Using it

See `examples/hal_adc_prs_gpio.rs`, `examples/hal_adc_prs_timer.rs`, `examples/hal_adc_prs_trigger.rs`, `examples/hal_adc_read.rs`, `examples/hal_one_of_each.rs`
