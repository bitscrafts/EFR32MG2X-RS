# PCNT0's ceiling: it is a pulse-width limit, not a counting limit

2026-11-26. The queue item was "find the exact ceiling of `PCNT0` between 25 and 50 kHz, with a producer whose rate
does not depend on `delay_us`". Two producers were built and measured. Neither reached the counter, and the reason is
worth more than the number would have been.

## What was tried, and what the board said

### Producer 1: TIMER0's overflow, through PRS

TIMER0 paced by `Board::pace_microphone` overflows at a rate the board computed; PRS channel 0 carried
`Source::Timer0Overflow` into `CONSUMER_PCNT0_S0IN`. Rates from 1 kHz to 50 kHz, window 450 SYSRTC ticks:

```
pcntrate: r1k=0/13 r5k=0/68 r10k=0/137 r16k=0/219 r20k=0/274 r50k=0/686 w=450 followed=0 ceiling=0k
```

Zero at **1 kHz**, where 13 edges were produced. A rate that low cannot be a bandwidth problem, so the counter was not
seeing the edges at all.

### Why: the pulse is 53 ns and the sampler runs at 30.5 us

The overflow event is one clock wide -- about 53 ns at 19 MHz. `PCNT0` samples S0IN with the `EM23GRPACLK` branch, which
this board runs from LFRCO at 32.768 kHz (`CFG.MODE = OVSSINGLE`). The probability of a sample landing inside a 53 ns
pulse is 53 ns / 30.5 us, about 1 in 575, so 13 edges give an expected count of 0.02. Zero is the correct answer to the
question as posed.

**`EM23GRPACLK` has only three sources in this part -- LFRCO, LFXO, ULFRCO** -- so there is no fast clock to move the
sampler onto: the fastest is 32.768 kHz. The only other clock for the block is `CLKSEL = 2`, the dedicated `PCNT_S0`
pin, and pointing it at a routed GPIO was already measured to leave the block with no clock at all.

### Producer 2: the timer driving a pad, read back through the PRS producer

If the sampler cannot see a narrow pulse, give it a wide one: TIMER0 drives PA0 as a 50 % square wave (500 us high at
1 kHz, 31 us at 16 kHz) and PRS reads `Source::GpioPin(0)` from the same pad.

```
pcntpwm: r1k=0/13 r5k=0/68 r10k=0/137 r16k=0/219 r25k=0/343 w=450 followed=0 ceiling=0k
```

Zero again, at every rate, including 1 kHz with a 500 us high time -- seventeen sampling periods long. So the pad is
not on the producer path once the timer owns it as an output. The GPIO producer reads the pad's **input** level, and
the routing that makes a pad a producer (`pin::EdgeWatch`) is the routing a measurement has to use; a pin driven by its
own timer as an alternate-function output is not the same thing.

## What this establishes

- **`PCNT0`'s limit in this configuration is the sampling clock, not the counter**: 32.768 kHz, so a pulse must be wide
  relative to 30.5 us. It is not a "25 to 50 kHz" bandwidth question and the number the plan asked for does not exist.
- **A timer's overflow event is not a usable producer for it** -- one clock wide is invisible.
- **A pad driven by a timer output is not a usable producer either**, measured: the PRS GPIO producer is the pad's input.
- The pull-resistor producer measured on 2026-11-26 does work (`PRS-GPIO-PRODUCER-2026-11-26.md`), because the pad's
  level is held while it moves -- which is exactly the property a product's external signal has and a timer pulse does
  not.

## What would be needed to finish the item

One of these, and each is external hardware, which is why this is parked rather than guessed:

1. a signal generator or another MCU on a free pad, producing pulses at a known rate with a known width -- this measures
   the ceiling as a function of width, which is the honest form of the question;
2. the dedicated `PCNT_S0` pin brought out to a jumper, so `CLKSEL = 2` can clock the block from the signal itself;
3. a pulse extender (a one-shot or a flip-flop) between the timer and the pad, if only the timer is available.

Probes kept: `_trash/pcnt-2026-11-26/hal_pcnt_timer_rate.rs` and `hal_pcnt_pwm_rate.rs`. They compile and they produce
the two records above; neither carries a verdict, because neither measured what it set out to measure.