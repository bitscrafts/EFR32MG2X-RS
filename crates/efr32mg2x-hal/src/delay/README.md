# delay

Delay implementation using SysTick

## What it is

Delay implementation using SysTick

This module provides blocking delay functions using the ARM Cortex-M SysTick timer.

# Examples

```no_run
use efr32mg2x_hal::{clock::Clocks, delay::Delay, pac};
use embedded_hal::delay::DelayNs;

let cp = cortex_m::Peripherals::take().unwrap();
let dp = pac::Peripherals::take().unwrap();

let clocks = Clocks::new(dp.CMU_NS, Default::default()).freeze();
let mut delay = Delay::new(cp.SYST, &clocks);

// Delay for 1 second
delay.delay_ms(1000);

// Delay for 100 microseconds
delay.delay_us(100);
```

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Delay` | struct | Delay provider using SysTick Provides blocking delays using the Cortex-M SysTick timer. |
| `free` | fn | Release the SysTick peripheral This consumes the Delay and returns the SysTick peripheral so it can be used for other purposes. |
| `new` | fn | Create a new delay provider # Arguments * `syst` - The SysTick peripheral * `clocks` - The frozen clock configuration # Examples ```no_run # use efr32mg2x_hal::{clock::Clocks, delay::Delay, pac}; # let cp = cortex_m::Peripherals::take().unwrap(); # let dp = pac::Peripherals::take().unwrap(); # let clocks = Clocks::new(dp.CMU_NS, Default::default()).freeze(); let delay = Delay::new(cp.SYST, &clocks); ```. |
| `with_sysclk` | fn | Create a delay provider from a stated clock frequency. |

## Registers it touches

`Peripherals`

## What was measured

```
    delayclk: ok=1 hfxo=1 hz0=19000000 d0=200279 u0=99125 hz1=39000000 pw=10003 r1=38988303 em=200 d1=201124 e1=5 u1=100010 eu=0 x=401171 ex=2
    delayclk: ok=1 hfxo=1 hz0=19000000 d0=200279 u0=99125 hz1=39000000 pw=10003 r1=38988303 em=199 d1=201150 e1=5 u1=100010 eu=0 x=401145 ex=2
```

Controls and the full story: [DELAY-MEASURED-2026-11-27.md](../../../../docs/04-reference/DELAY-MEASURED-2026-11-27.md)

## What is not proven

Nothing in `docs/` records an unproven path for this module.

## Using it

See `examples/hal_delay_clock.rs`
