# clock

Clock system implementation with hardware register access

## What it is

Clock system implementation with hardware register access

This module implements actual hardware configuration for the EFR32MG24 clock system.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `ClockConfig` | struct | — |
| `ClockError` | enum | — |
| `Clocks` | struct | — |
| `DEFAULT_SYSCLK_HZ` | const | The SYSCLK this HAL configures when no other clock source is asked for. |
| `Em01Source` | enum | — |
| `Error` | enum | — |
| `FSRCO_MAX_HZ` | const | `FSRCO` maximum, datasheet Rev 1.2 Table 4.28. |
| `FSRCO_MIN_HZ` | const | `FSRCO` minimum, datasheet Rev 1.2 Table 4.28. |
| `FSRCO_TYP_HZ` | const | `FSRCO` typical, datasheet Rev 1.2 Table 4.28. |
| `FrozenClocks` | struct | Frozen clock configuration Once frozen, the clock configuration cannot be changed. |
| `Hertz` | struct | — |
| `HfxoConfig` | struct | — |
| `LFRCO_TOLERANCE_PER_CENT` | const | `LFRCO` accuracy in normal mode, percent, datasheet Rev 1.2 Table 4.29. |
| `LFRCO_TYP_HZ` | const | `LFRCO` nominal, datasheet Rev 1.2 Table 4.29. |
| `LfxoConfig` | struct | — |
| `ULFRCO_MAX_HZ` | const | `ULFRCO` maximum, datasheet Rev 1.2 Table 4.30. |
| `ULFRCO_MIN_HZ` | const | `ULFRCO` minimum, datasheet Rev 1.2 Table 4.30. |
| `ULFRCO_TYP_HZ` | const | `ULFRCO` typical, datasheet Rev 1.2 Table 4.30. |
| `em01_readback` | fn | — |
| `em01_select` | fn | Point `EM01GRPACLK` -- the clock group TIMER0-4 and USART0 run from -- at `source`, and check it took. |
| `enable_peripheral_clock` | fn | — |
| `fn` | const | — |
| `freeze` | fn | Freeze the clock configuration This consumes the Clocks struct and CMU peripheral, returning a FrozenClocks which can be used by other peripherals but cannot be modified. |
| `hfclk` | fn | — |
| `hfrcoem23_band` | fn | Force `HFRCOEM23` on, wait for `STATUS.RDY`, and return the band `CAL.FREQRANGE` reports. |
| `hfrcoem23_cal` | fn | — |
| `hfrcoem23_status` | fn | — |
| `lfclk` | fn | — |
| `new` | fn | Configure the clock system with hardware register access # Arguments * `cmu` - CMU peripheral (consumed to ensure exclusive access) * `config` - Clock configuration # Returns A `Result` containing either the configured `Clocks` or a `ClockError` # Errors Returns `ClockError::HfxoTimeout` if HFXO fails to stabilize within timeout Returns `ClockError::LfxoTimeout` if LFXO fails to stabilize within timeout # Hardware Configuration This method configures the CMU registers to: - Enable and configure HFXO if requested - Enable and configure LFXO if requested - Select SYSCLK source - Wait for oscillator stabilization with timeout # Example ```no_run use efr32mg2x_hal::{clock::{Clocks, ClockConfig, HfxoConfig}, pac}; let dp = pac::Peripherals::take().unwrap(); let clocks = Clocks::new( dp.cmu_s, ClockConfig { hfxo: None, lfxo: None, // Use internal LFRCO } )?; # Ok::<(), efr32mg2x_hal::clock::ClockError>(()) ```. |
| `pclk` | fn | — |
| `sysclk` | fn | — |

## Registers it touches

`CmuS`, `Hfrcoem23S`, `Hfxo0Ns`, `Peripherals`

## What was measured

```
    tx: radioclk=39000kHz      (the clock RAIL now sees)
    tx: pdu=20B clock=39000
```

Controls and the full story: [RAIL-BLE-ON-XIAO.md](../../../../docs/04-reference/RAIL-BLE-ON-XIAO.md)

## What is not proven

- ⏳ Peripheral clock individual enable/disable (deferred). (PLAN.md (`../../../../docs/03-development/PLAN`))

## Using it

See `examples/hal_cmu_sources.rs`, `examples/hal_delay_clock.rs`
