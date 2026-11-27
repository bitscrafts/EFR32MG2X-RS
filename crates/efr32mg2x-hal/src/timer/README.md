# timer

Timer and PWM Driver

## What it is

Timer and PWM Driver

This module provides Timer and PWM functionality using TIMER0-4 peripherals
for the EFR32MG24 microcontroller.

# Hardware Registers

- **EN**: Enable register
- **CTRL**: Control register (mode, counting direction)
- **CFG**: Configuration register
- **CMD**: Command register (start, stop)
- **CNT**: Counter value
- **TOP**: Top value (period for PWM)
- **CC0/CC1/CC2_OC**: Output Compare registers (duty cycle)
- **CC0/CC1/CC2_CTRL**: Channel control registers
- **STATUS**: Status flags

# Features

- 5 Timer instances (Timer0-4)
- Each timer has 3 PWM channels
- Edge-aligned and center-aligned PWM modes
- Configurable frequency and duty cycle
- 16-bit or 32-bit resolution
- Hardware register access

# Example

```no_run
use efr32mg2x_hal::{
clock::{Clocks, ClockConfig, HfxoConfig},
timer::{Timer0, Config, PwmMode, PwmChannel},
pac,
};

let dp = pac::Peripherals::take().unwrap();

// Configure clocks
let clocks = Clocks::new(dp.cmu_s, ClockConfig {
hfxo: None,
lfxo: None,
}).freeze();

// Create Timer0 with PWM at 10 kHz
let mut timer = Timer0::new(
dp.timer0_s,
Config::new(10_000).with_pwm(PwmMode::EdgeAligned),
&clocks
);

// Set 50% duty cycle on channel 0
timer.set_duty_cycle(PwmChannel::Channel0, 50).unwrap();
timer.enable_channel(PwmChannel::Channel0);
```

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Config` | struct | — |
| `Error` | enum | — |
| `PwmChannel` | enum | — |
| `PwmMode` | enum | — |
| `clear_overflow` | fn | Clear overflow interrupt flag. |
| `disable_channel` | fn | Disable PWM output on a channel # Arguments * `channel` - PWM channel to disable. |
| `enable_channel` | fn | Enable PWM output on a channel # Arguments * `channel` - PWM channel to enable # PWM Output Configuration This configures the CC channel for PWM mode with: - **CMOA (Compare Match Output Action)**: SET - Output goes high on compare match - **COFOA (Counter Overflow Output Action)**: TOGGLE - Output toggles on overflow/underflow For edge-aligned PWM (up-counting): - Counter counts from 0 to TOP - Output goes HIGH when CNT == CCx_OC (compare match) - Output goes LOW when CNT reaches TOP (counter overflow) - Duty cycle = (CCx_OC / TOP) × 100% For center-aligned PWM (up-down counting): - Counter counts 0→TOP→0 - Output toggles on compare matches in both directions - Creates symmetric waveform **Note**: This configuration has been validated against EFR32MG24 reference manual section on Timer PWM generation. |
| `fn` | const | Create a new timer configuration # Arguments * `frequency` - Timer frequency in Hz # Example ```no_run let config = Config::new(1_000_000); // 1 MHz timer ```. |
| `get_counter` | fn | Get current counter value. |
| `get_frequency` | fn | Get the configured PWM frequency. |
| `get_top_value` | fn | Get the TOP value (period). |
| `is_overflow` | fn | Check if overflow interrupt flag is set. |
| `listen_overflow` | fn | Enable timer overflow interrupt This enables the overflow/underflow interrupt, useful for periodic tasks synchronized with the timer period. |
| `new` | fn | Creates a new timer instance # Arguments * `timer` - Timer peripheral instance * `config` - Timer configuration * `clocks` - Frozen clock configuration. |
| `reset_counter` | fn | Reset counter to zero. |
| `set_duty_cycle` | fn | Set duty cycle for a PWM channel # Arguments * `channel` - PWM channel (0, 1, or 2) * `duty_percent` - Duty cycle percentage (0-100) # Returns `Ok(())` on success, `Err(Error)` if duty cycle is invalid # Thread Safety This function uses critical sections for atomic register access, making it safe to call from interrupt contexts or RTOS tasks. |
| `set_duty_raw` | fn | Set raw duty cycle value for a PWM channel This provides direct access to the compare register for applications requiring precise control beyond percentage-based duty cycles. |
| `start` | fn | Start the timer. |
| `stop` | fn | Stop the timer. |
| `unlisten_overflow` | fn | Disable timer overflow interrupt. |

## Registers it touches

`Peripherals`, `Timer0S`

## What was measured

```
    app: windows=63 short=0 cycles=20735103 timer=1 bands=377810863 pct_of_realtime=108
    interleaved: windows=63 missing=0 cycles=19151407 pct=99 timer=1 bands=464915712   status=PASS
    ble: seconds=5 windows=315 published=5 notifies=0 timer=1 rms=68 b0=89423434   status=PASS
    letimer: clk=0x00000101 top0=1000 top=1000 comp0=500 comp1=250 refused=1 cnt0=1000 p1=501 qe=0 ql=1 pq=250 wrap=1 if1=0x00000001 ifc=0x00000000 if2=0x00000001 p2=499 cnt=499
```

Controls and the full story: [APP-LOOP-BUDGET.md](../../../../docs/04-reference/APP-LOOP-BUDGET.md), [APP-LOOP-WITH-BLE-OPEN.md](../../../../docs/04-reference/APP-LOOP-WITH-BLE-OPEN.md), [LETIMER-COMPARE-2026-11-26.md](../../../../docs/04-reference/LETIMER-COMPARE-2026-11-26.md), [LETIMER-TOP-2026-11-26.md](../../../../docs/04-reference/LETIMER-TOP-2026-11-26.md)

## What is not proven

- C -- advanced | complete to stage 2 | `adc`, `dma`, `emu`, `rtc`/`sysrtc`, `wdog`, `gpcrc`, `mvp`, `letimer`, `prs`; DMA stages 3 and 4 deferred |. (PLAN.md (`../../../../docs/03-development/PLAN`))
- `timer1`–`timer4` | `timer` | same driver, one instance each | compiled, not measured |. (COVERAGE-BOUNDARY-2026-11-25.md (`../../../../docs/04-reference/COVERAGE-BOUNDARY-2026-11-25`))
- `prs` | peripheral reflex system: routes an event from one peripheral to another without the CPU | ~250 lines, of which the crate now drives a channel: `prs::Source` names the producers it encodes (`Off`, `Timer0Overflow`, `Timer0Compare0`, `Timer1Overflow`, `GpioPin`), `Prs::connect` writes `ASYNC_CHn_CTRL` whole and `Prs::channel_ctrl` reads it back, and `Prs::pulse` is the software edge. (COVERAGE-BOUNDARY-2026-11-25.md (`../../../../docs/04-reference/COVERAGE-BOUNDARY-2026-11-25`))

## Using it

No example imports this module yet, so its API has never been exercised from `examples/`.
