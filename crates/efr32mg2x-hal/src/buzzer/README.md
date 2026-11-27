# buzzer

Piezo buzzer on TIMER0 compare output — the Expansion Base's `D3` (PC03).

## What it is

Piezo buzzer on TIMER0 compare output — the Expansion Base's `D3` (PC03).

## Why a timer and not a delay loop

A passive buzzer needs a square wave at the wanted pitch. Producing it by
toggling a pin in a NOP loop ties the CPU up and makes the pitch a function of
whatever else the program is doing. TIMER0's compare channel generates it in
hardware: the CPU sets a frequency and a duty, and is free.

## The sequence this die demands

Measured, and in this order — every deviation is a bus fault or a silent timer:

```text
LOCK = 0xCE80            unlock the timer's configuration registers
EN   = 0                 configuration is only writable with the timer disabled
CFG  = up, prescem01grpaclk, div1
CC0_CFG = output compare  (also needs EN = 0)
EN   = 1
TOP  = period            (this one needs EN = 1)
CC0_OC = half period     (so the output is a 50 % square)
CC0_CTRL = toggle on compare
```

## Why the frequency is calibrated instead of assumed

`HFRCOEM23` does not start in the band the datasheet suggests: measured on this
board it runs at roughly 5 MHz at reset and settles near 19 MHz, so a `TOP`
computed from 20 MHz gave a tone 5 % flat. The driver therefore counts timer
ticks against a known CPU delay once, then computes `TOP` from what it measured.
See `../../../../docs/../../../../docs/04-reference/PAC-CLOCK-AND-TIMER-NOTES.md`.

## Silence

Stopping the timer stops `CNT` too, which breaks any timing that waits on it.
Silence is `CC0_CTRL.CMOA = none` with the timer still counting — measured
failure of the obvious approach.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Buzzer` | struct | A calibrated buzzer: TIMER0 in toggle mode on one pin. |
| `DEFAULT_TONE_HZ` | const | The piezo on this board is loudest near 2.7 kHz; the base's own demo uses it. |
| `period_ticks` | fn | — |
| `set_sysclk_hz` | fn | Override the assumed SYSCLK, keeping the measured tick/cycle ratio. |
| `set_tone` | fn | Set the pitch. |
| `silence` | fn | Silence without stopping the counter, so timings that wait on `CNT` keep working. |
| `sound` | fn | Sound the tone. |
| `sysclk_hz` | fn | — |
| `tick_hz` | fn | — |

## Registers it touches

`Peripherals`

## What was measured

No mailbox record in `docs/` names this module, so treat its API as unmeasured on hardware. The first measurement belongs here, with the line the board produced and the controls around it.

## What is not proven

Nothing in `docs/` records an unproven path for this module.

## Using it

See `examples/hal_timer0_pwm.rs`
