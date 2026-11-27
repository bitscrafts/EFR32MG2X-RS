# wdog

Watchdog Timer (WDOG0/WDOG1) Module

## What it is

Watchdog Timer (WDOG0/WDOG1) Module

This module provides a Hardware Abstraction Layer (HAL) for the EFR32MG24's
Watchdog Timer peripheral, which monitors system operation and resets the device
if software fails to periodically "feed" (clear) the watchdog.

# Features

- **Configurable Timeout**: 16 timeout periods from 9ms to 4.3 minutes
- **Energy Mode Support**: Can run in EM0-EM3 (configurable)
- **Lock Mechanism**: Configuration can be locked to prevent malicious changes
- **Debug Mode**: Can be frozen during debugging (development-friendly)
- **Warning Mode**: Can generate interrupts without reset (for testing)

# Current Implementation (C5-WDOG Stage 1)

- ✅ Watchdog enable with configuration
- ✅ Feed (clear) operation
- ✅ Lock/unlock configuration
- ✅ Timeout period selection (16 options)
- ✅ Energy mode support (EM1/EM2/EM3)
- ✅ Debug mode control
- ✅ Status queries (enabled, locked)

# Not Implemented (Future)

- ⏳ Interrupt handling
- ⏳ PRS (Peripheral Reflex System) integration
- ⏳ Window mode (minimum feed interval)

# Usage

```rust,no_run
use efr32mg2x_hal::{wdog::{Wdog, WdogConfig, TimeoutPeriod}, pac};

let dp = pac::Peripherals::take().unwrap();

// Create configuration: 2 second timeout, runs in all modes
let config = WdogConfig::new(TimeoutPeriod::Cycles2k)
.run_in_all_modes();

// Enable watchdog
let mut wdog = Wdog::new(dp.wdog0_s, config)
.expect("Failed to enable watchdog");

loop {
// Do work
perform_task();

// Feed watchdog to prevent reset
wdog.feed();

// If we don't feed within 2 seconds, device will reset
}
```

# Safety Considerations

**Critical for Production**:
- Once locked, watchdog configuration CANNOT be changed without reset
- Ensure feed interval is shorter than timeout period
- Test thoroughly before locking in production
- Consider using warning mode first for validation

**Lock Key**: 0xABE8 (44008) - required to unlock configuration

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Error` | enum | — |
| `State` | struct | — |
| `TimeoutPeriod` | enum | — |
| `Wdog` | struct | Watchdog Timer Owns the WDOG peripheral and provides watchdog functionality. |
| `WdogConfig` | struct | — |
| `clear_timeout_flag` | fn | Clears the watchdog timeout warning flag # Example ```rust,no_run if wdog.timeout_flag() { wdog.clear_timeout_flag(); } ```. |
| `cmd_sync_busy` | fn | Whether a `CMD` write is still crossing into the watchdog's clock domain. |
| `debug_run` | fn | Enable debug mode (watchdog runs even when debugger is attached). |
| `feed` | fn | Feeds (clears) the watchdog timer This resets the watchdog counter to 0, preventing a timeout reset. |
| `is_enabled` | fn | Checks if the watchdog is enabled # Returns `true` if watchdog is enabled and running, `false` otherwise # Example ```rust,no_run if wdog.is_enabled() { println!("Watchdog is active"); } ```. |
| `is_locked` | fn | Checks if the watchdog configuration is locked # Returns `true` if configuration is locked, `false` if unlocked # Example ```rust,no_run if wdog.is_locked() { println!("Watchdog is locked - cannot modify config"); } ```. |
| `lock` | fn | Locks the watchdog configuration Once locked, the watchdog configuration CANNOT be changed without a device reset. |
| `new` | fn | Creates a new Watchdog instance and enables it with the given configuration This function: 1. |
| `run_in_all_modes` | fn | Enable watchdog to run in all energy modes. |
| `state` | fn | Every register of the block in one read, for instruments and for fault diagnosis. |
| `timeout_flag` | fn | Checks if a watchdog timeout warning occurred When `reset_disable` is true (warning-only mode), this flag is set when timeout occurs instead of resetting the device. |
| `timeout_ms` | fn | Get timeout in milliseconds (assuming 1 kHz watchdog clock). |
| `warning_only` | fn | Disable reset on timeout (warning-only mode for testing). |

## Registers it touches

`CmuS`, `Peripherals`, `Wdog0Ns`

## What was measured

No mailbox record in `docs/` names this module, so treat its API as unmeasured on hardware. The first measurement belongs here, with the line the board produced and the controls around it.

## What is not proven

- C -- advanced | complete to stage 2 | `adc`, `dma`, `emu`, `rtc`/`sysrtc`, `wdog`, `gpcrc`, `mvp`, `letimer`, `prs`; DMA stages 3 and 4 deferred |. (PLAN.md (`../../../../docs/03-development/PLAN`))

## Using it

See `examples/hal_one_of_each.rs`, `examples/hal_wdog_feed.rs`
