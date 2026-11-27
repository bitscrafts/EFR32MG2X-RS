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
| `new_at` | fn | The same driver, built from the timer clock in hertz instead of a `FrozenClocks`. |
| `reset_counter` | fn | Reset counter to zero. |
| `set_duty_cycle` | fn | Set duty cycle for a PWM channel # Arguments * `channel` - PWM channel (0, 1, or 2) * `duty_percent` - Duty cycle percentage (0-100) # Returns `Ok(())` on success, `Err(Error)` if duty cycle is invalid # Thread Safety This function uses critical sections for atomic register access, making it safe to call from interrupt contexts or RTOS tasks. |
| `set_duty_raw` | fn | Set raw duty cycle value for a PWM channel This provides direct access to the compare register for applications requiring precise control beyond percentage-based duty cycles. |
| `start` | fn | Start the timer. |
| `stop` | fn | Stop the timer. |
| `unlisten_overflow` | fn | Disable timer overflow interrupt. |

## Registers it touches

`Peripherals`

## What was measured

```
    app: windows=63 short=0 cycles=20735103 timer=1 bands=377810863 pct_of_realtime=108
    interleaved: windows=63 missing=0 cycles=19151407 pct=99 timer=1 bands=464915712   status=PASS
    ble: seconds=5 windows=315 published=5 notifies=0 timer=1 rms=68 b0=89423434   status=PASS
    letimer: clk=0x00000101 top0=1000 top=1000 comp0=500 comp1=250 refused=1 cnt0=1000 p1=501 qe=0 ql=1 pq=250 wrap=1 if1=0x00000001 ifc=0x00000000 if2=0x00000001 p2=499 cnt=499
```

Controls and the full story: [APP-LOOP-BUDGET.md](../../../../docs/04-reference/APP-LOOP-BUDGET.md), [APP-LOOP-WITH-BLE-OPEN.md](../../../../docs/04-reference/APP-LOOP-WITH-BLE-OPEN.md), [LETIMER-COMPARE-2026-11-26.md](../../../../docs/04-reference/LETIMER-COMPARE-2026-11-26.md), [LETIMER-TOP-2026-11-26.md](../../../../docs/04-reference/LETIMER-TOP-2026-11-26.md)

## What is not proven

- `EUSART1` was the last row of the coverage boundary that read "compiled, not measured". (EUSART1-2026-11-27.md (`../../../../docs/04-reference/EUSART1-2026-11-27`))
- Wake latency:** if the MCU timer paces each sample, every sample costs an EM2→EM0 wake and an I2C transfer. (HAL-OPUS-ANALYSIS-2026-11-22.md (`../../../../docs/04-reference/HAL-OPUS-ANALYSIS-2026-11-22`))
- A compare event** (`SIGSEL = TIMER0CC0`) is encoded and not measured; only the overflow is. (PRS-TIMER-PRODUCER-2026-11-26.md (`../../../../docs/04-reference/PRS-TIMER-PRODUCER-2026-11-26`))

## Using it

See `examples/hal_timer_instances.rs`
