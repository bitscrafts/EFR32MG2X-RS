# pwm

TIMER0 as a PWM source on any pin.

## What it is

TIMER0 as a PWM source on any pin.

This is the module behind "the louder you shout, the brighter the light" — the demo Seeed
ships in the flash of every XIAO MG24 **Sense**. In Arduino that is `analogWrite(pin, 0..255)`;
here it is a compare channel with a duty cycle, because a GPIO has exactly two brightnesses
and a PWM has 1024.

## What this module owns, and what it does not

It owns the timer: the clock gate, the oscillator choice, the route to one pin, and the
calibration of the tick rate against the CPU cycle counter. It does **not** know which pin
the caller wants, nor what is attached to it — an LED and a piezo are the same waveform at
different frequencies.

## The two facts that cost time here

- **HFRCOEM23, not FSRCO.** The IADC and TIMER0 share the requirement; the oscillator is
forced on and waited for, because a timer clocked by a stopped source reports a running
counter and counts nothing.
- **`asm::delay(n)` is not `n` cycles.** It is ~3 cycles per iteration on this part, which
turned a 19 MHz measurement into 57 MHz and a 2.7 kHz tone into something else entirely.
The rate below is measured with `DWT->CYCCNT`, and the tick/cycle ratio is what is stored.

## One owner per timer

`TIMER0` has one counter and one route per channel, so a Pwm and a Buzzer cannot be alive at
the same time. [`Timer0::take`] returns `None` the second time, which makes that a compile-time
discipline rather than a runtime surprise.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Pwm` | struct | A PWM channel: a frequency, and a duty cycle between 0 % and 100 %. |
| `Timer0` | struct | The timer, configured and calibrated, before it is given a meaning. |
| `free` | fn | — |
| `frequency_hz` | fn | — |
| `new` | fn | Start a PWM at `freq_hz`, dark (0 % duty). |
| `off` | fn | Turn the output off. |
| `period_ticks` | fn | — |
| `set_duty` | fn | Set the duty cycle, 0..=100. |
| `set_frequency` | fn | Change the PWM frequency, keeping the current duty. |
| `set_sysclk_hz` | fn | Tell the timer what SYSCLK actually is, keeping the measured tick/cycle ratio. |
| `silence` | fn | Stop driving the pin without stopping the counter. |
| `sound` | fn | Drive the pin with the square wave `square_wave` configured. |
| `square_wave` | fn | Drive channel 0 as a square wave at `hz`, half period high and half low. |
| `sysclk_hz` | fn | — |
| `tick_hz` | fn | — |

## Registers it touches

`Peripherals`

## What was measured

```
    pcntpwm: r1k=0/13 r5k=0/68 r10k=0/137 r16k=0/219 r25k=0/343 w=450 followed=0 ceiling=0k
```

Controls and the full story: [PCNT-CEILING-2026-11-26.md](../../../../docs/04-reference/PCNT-CEILING-2026-11-26.md)

## What is not proven

Nothing in `docs/` records an unproven path for this module.

## Using it

No example imports this module yet, so its API has never been exercised from `examples/`.
