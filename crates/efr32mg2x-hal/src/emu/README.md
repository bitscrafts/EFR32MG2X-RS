# emu

Energy Management Unit (EMU) Module

## What it is

Energy Management Unit (EMU) Module

This module provides a Hardware Abstraction Layer (HAL) for the EFR32MG24's
Energy Management Unit (EMU), enabling efficient power management through
energy mode transitions and voltage scaling.

# Features

- **Energy Mode Transitions**: EM0 → EM1 → EM2 → EM3 → EM4
- **Voltage Scaling**: 0.9V, 1.0V, 1.1V for power optimization
- **EM2 Debug Support**: Keep debugger active in deep sleep
- **EM4 IO Retention**: Preserve GPIO states through hibernate
- **Thread-Safe**: Critical sections for RTOS environments

# Current implementation (C3-EMU Stage 1)

- EM1 (sleep): `enter_em1`.
- EM2 (deep sleep): `enter_em2` with an `Em2Config`, and `em2_entered` to read back where the part actually is.
- Voltage scaling for EM0/EM1 and EM2/EM3: `set_em01_voltage_scale` and `voltage_scale`.
- EM3 and EM4 are **not offered**: no door in this module reaches them, so there is nothing here to measure. The
register work for them exists in the PAC; the sequence that returns from each without losing the debug session is
what is missing, and until it is measured a method that claims it would be a claim.

# Energy Modes

```text
┌─────────────────────────────────────────────────────────┐
│                    Energy Modes                          │
├─────────────────────────────────────────────────────────┤
│ EM0 │ Active      │ Full CPU, all peripherals           │
│ EM1 │ Sleep       │ CPU stopped, peripherals running    │
│ EM2 │ Deep Sleep  │ Major power savings                 │
│ EM3 │ Stop        │ Minimal power                       │
│ EM4 │ Hibernate   │ Lowest power (~100nA)               │
└─────────────────────────────────────────────────────────┘
```

# Usage

```rust,no_run
use efr32mg2x_hal::{emu::{Emu, Em2Config, VoltageScale}, pac};

let dp = pac::Peripherals::take().unwrap();

// Initialize EMU
let emu = Emu::new(dp.emu_s);

// Enter EM1 (Sleep Mode)
emu.enter_em1();  // Wakes on any interrupt

// Enter EM2 (Deep Sleep Mode) with custom config
let config = Em2Config {
voltage_scale: VoltageScale::Vscale1,  // 1.0V for power savings
debug_enable: false,  // Disable debug in production
};
emu.enter_em2(config).expect("Failed to enter EM2");
```

# Safety

This module uses `unsafe` for direct hardware register access. All unsafe
operations are documented with SAFETY comments.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Em2Config` | struct | — |
| `Em4Config` | struct | — |
| `Emu` | struct | EMU (Energy Management Unit) Owns the EMU peripheral and provides methods for energy mode transitions and voltage scaling. |
| `EnergyMode` | enum | — |
| `Error` | enum | — |
| `IoRetentionMode` | enum | — |
| `VoltageScale` | enum | — |
| `em2_entered` | fn | Check if EM2 was entered Returns true if the device has entered EM2 since the last reset. |
| `enter_em1` | fn | Enter EM1 (Sleep Mode) Enters Energy Mode 1 where the CPU is stopped but peripherals continue to run. |
| `enter_em2` | fn | Enter EM2 (Deep Sleep Mode) Enters Energy Mode 2 where major power savings are achieved by stopping the CPU and most peripherals. |
| `is_locked` | fn | Check if EMU registers are locked # Returns `true` if registers are locked, `false` if unlocked. |
| `new` | fn | Creates a new EMU instance Takes ownership of the EMU peripheral. |
| `set_em01_voltage_scale` | fn | Set voltage scaling for EM0/EM1 Configures the voltage regulator output level for EM0 (Active) and EM1 (Sleep) modes. |
| `voltage_scale` | fn | Get current voltage scaling level Reads the current voltage scaling setting from the STATUS register. |

## Registers it touches

`EmuS`, `Peripherals`

## What was measured

No mailbox record in `docs/` names this module, so treat its API as unmeasured on hardware. The first measurement belongs here, with the line the board produced and the controls around it.

## What is not proven

Nothing in `docs/` records an unproven path for this module.

## Using it

See `examples/hal_emu_voltage.rs`
