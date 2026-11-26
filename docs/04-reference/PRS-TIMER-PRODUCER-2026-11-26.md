# A timer's overflow converts, with no CPU write anywhere in the loop

2026-11-26. The PRS trigger measured earlier tonight (`IADC-PRS-TRIGGER-2026-11-26.md`) was driven by
`ASYNC_SWPULSE`, which is a register write: it proved the converter's edge path and nothing about the reflex system,
because the producer was still the CPU. A reflex system that only reflex to the core is not one.

This record is the next step: asynchronous channel 0 pointed at TIMER0's overflow, TIMER0 free-running from
HFRCOEM23, and the converter armed once. The core then does nothing but pop conversions.

## The record, exactly as the board produced it

The instrument is `hal_adc_prs_timer` (`crates/efr32mg2x-hal/examples/`). Two runs of the same image, both `PASS`,
differing only in the two sampled values (PC9's microphone bias, noise around mid-scale):

```
adcprst: trig=prs0 hz=500 period=38000 src=0x000c2101 off=0 idle=0 noarm=0 ev=8 quiet=0 v0=0x0000026d vl=0x00000238 ovf=8 ch0=0x000c0000
adcprst: trig=prs0 hz=500 period=38000 src=0x000c2101 off=0 idle=0 noarm=0 ev=8 quiet=0 v0=0x0000026b vl=0x00000230 ovf=8 ch0=0x000c0000
```

- `hz=500`, `period=38000`: TIMER0 was paced at 500 Hz from a 19 MHz clock, which is 38 000 timer clocks per period —
  the value `Board::pace_microphone` returns and the pace the microphone's DMA uses.
- `src=0x000c2101`: the readback of `PRS.ASYNC_CH0_CTRL` with TIMER0 connected (`FNSEL = 0xC`, `SOURCESEL = 0x21`,
  `SIGSEL = 1`).
- `ev=8`, `ovf=8`: eight timer overflows counted, eight conversions popped.
- `ch0=0x000c0000`: the same channel pointed back at nothing, which is also its reset value.

## The five phases, and why four of them are controls

| phase | channel source | armed | timer | conversions |
|---|---|---|---|---|
| A | none (`0x000c0000`) | yes | running | `off=0` |
| B | `TIMER1` overflow | yes | **not clocked** | `idle=0` |
| C | `TIMER0` overflow | **no** | running | `noarm=0` |
| D | `TIMER0` overflow | yes | running | `ev=8` |
| E | none | yes | still running | `quiet=0` |

A and E are the same configuration as D with one field changed, and they are the controls that the count belongs to
the channel's source rather than to the arm or to the timer running at all. B names a producer in the encoding whose
timer nobody started: same code path, same arming, no edges. C removes the arm with the real producer running — the
edges arrive at an unarmed queue and convert nothing.

## The word, from the vendor's signal header

`crates/efr32mg24-emlib-sys/vendor/device/efr32mg24_prs_signals.h`:

```
_PRS_ASYNC_CH_CTRL_SOURCESEL_TIMER0   0x21
_PRS_ASYNC_CH_CTRL_SOURCESEL_TIMER1   0x22
_PRS_ASYNC_CH_CTRL_SOURCESEL_GPIO     0x04
_PRS_ASYNC_CH_CTRL_SIGSEL_TIMER0UF    0x0
_PRS_ASYNC_CH_CTRL_SIGSEL_TIMER0OF    0x1
_PRS_ASYNC_CH_CTRL_SIGSEL_TIMER0CC0   0x2
_PRS_ASYNC_CH_CTRL_SIGSEL_GPIOPIN0    0x0   (.. GPIOPIN7 0x7)
```

`FNSEL` is bits 19:16 and its reset value is `0xC` — the mode that passes the selected signal through unchanged.
`SOURCESEL` is bits 14:8 and `SIGSEL` bits 2:0, which is the same layout the PAC gives every `ASYNC_CHn_CTRL`
(`FNSEL[19:16]`, `SOURCESEL[14:8]`, `SIGSEL[2:0]`, `AUXSEL[27:24]`).

`ASYNC_CH0_CTRL` sits at offset `0x018` in the block and the sixteen channel words follow it four bytes apart
(`ASYNC_CH4_CTRL` at `0x028` confirms the spacing). The driver writes the whole word rather than modifying it, so a
`FNSEL` left behind by another user cannot silently invert or gate the producer.

## What went wrong first, and what it says about the instrument

The first version bounded each window with `wrapping_sub` of two `u32` reads of `TIMER0.CNT` and broke out at
`8.5 * period`. On the board it closed 90 clocks into the run:

```
adcprst: trig=prs0 hz=500 period=38000 src=0x000c2101 off=0 idle=0 noarm=0 ev=0 quiet=1 v0=0x00000000 vl=0x00000000 win=4294967197 ch0=0x000c0000
```

`win=4294967197` is `0xFFFFFF9D`, i.e. `-99`: `CNT` is 16 bits and reloads at `TOP` (`38 000`), so a difference of two
reads goes negative once per period long before it reaches 323 000. The instrument, not the converter, was wrong —
and it was wrong in the direction that reports a working path as dead, which is at least the honest direction.

The window is now counted in the producer's **overflows** (a decrease of `CNT` is a wrap), which is immune to the
counter's width: `ovf=8` in the record is that counter.

A second defect fell out of the same run: with the window closing early, the conversion triggered by the last edge
landed in the FIFO after phase D had ended and was popped by phase E (`quiet=1`). Each window now drains the FIFO
before it arms and allows one bounded drain after its last wrap, so a conversion cannot be attributed to the phase
after the one that caused it.

## What is not proven

- **A GPIO producer.** `Source::GpioPin` is encoded from the header above, and nothing in this crate routes a pin to
  an external-interrupt channel or configures its edge, so no record exists for it.
- **A compare event** (`SIGSEL = TIMER0CC0`) is encoded and not measured; only the overflow is.
- **The fastest rate.** One-deep FIFO plus software polling: this measures eight edges two milliseconds apart, not the
  ceiling of the converter. `SINGLETRIGACTION = CONTINUOUS` and a DMA-fed FIFO would be a different measurement.

## Consequence for the delivery

`prs::Source` now names what it can connect (`Off`, `Timer0Overflow`, `Timer0Compare0`, `Timer1Overflow`,
`GpioPin`), `Prs::connect` writes the channel word whole, and `Prs::channel_ctrl` reads it back, so a probe can report
the word it configured instead of the word it meant to configure. The producer itself (a timer, a GPIO edge) stays the
producer's driver's business: `Prs::connect` starts nothing, which is exactly what makes phase B above possible.