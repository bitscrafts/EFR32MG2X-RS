# prs

The peripheral reflex system (PRS), as far as this crate drives it: one asynchronous channel, a source for it, and

## What it is

The peripheral reflex system (PRS), as far as this crate drives it: one asynchronous channel, a source for it, and
a software pulse.

## What is measured, and what this module is for

Measured 2026-11-26: a write to `ASYNC_SWPULSE` is XORed onto the channel's output, so a channel whose
`ASYNC_CHn_CTRL` selects no source still carries the edge. `ASYNC_CH0_CTRL` read back `0x000c0000` at reset
(`FNSEL = A`, `SOURCESEL` and `SIGSEL` zero) and **no write was needed** for eight pulses to become eight IADC
conversions (record: `../../../../docs/../../../../docs/04-reference/IADC-PRS-TRIGGER-2026-11-26.md`).

Measured 2026-11-26 as well, and the reason [`Source`] exists: with `SOURCESEL = TIMER0` and `SIGSEL = TIMER0UF`
(`0x0c2101` in `ASYNC_CH0_CTRL`) and TIMER0 free-running, eight timer periods produced eight IADC conversions with
no CPU pulse at all; the same channel with no source, and with a source whose timer is not clocked, produced none
(record: `../../../../docs/../../../../docs/04-reference/PRS-TIMER-PRODUCER-2026-11-26.md`).

## The clock

`CMU.CLKEN0.PRS` is a gate: the block's registers are behind a bus clock, and [`Board::prs`] opens it before handing
the block out. (The positive direction is measured — every run had the gate open. Nobody has run the negative one.)

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Error` | enum | — |
| `Prs` | struct | The PRS block, handed out once by `Board::prs`. |
| `Source` | enum | — |
| `channel_ctrl` | fn | What asynchronous `channel` is pointed at right now, as the raw word. |
| `connect` | fn | Point asynchronous `channel` at `source`, so the channel carries that producer's edges. |
| `pulse` | fn | One pulse on asynchronous `channel`: a single write, and the hardware makes the edge one clock wide. |

## Registers it touches

`PrsS`

## What was measured

```
    adcprs: trig=prs0 refused=1 armA=1 imm=0 armB=0 prs=8 v0=0x0000026f v7=0x000002a7 quiet=0 noarm=0
    adcprs: trig=prs0 refused=1 armA=1 imm=0 armB=0 prs=8 v0=0x0000026c v7=0x000002a9 quiet=0 noarm=0
    adcprst: trig=prs0 hz=500 period=38000 src=0x000c2101 off=0 idle=0 noarm=0 ev=8 quiet=0 v0=0x0000026d vl=0x00000238 ovf=8 ch0=0x000c0000
    adcprst: trig=prs0 hz=500 period=38000 src=0x000c2101 off=0 idle=0 noarm=0 ev=8 quiet=0 v0=0x0000026b vl=0x00000230 ovf=8 ch0=0x000c0000
```

Controls and the full story: [IADC-PRS-TRIGGER-2026-11-26.md](../../../../docs/04-reference/IADC-PRS-TRIGGER-2026-11-26.md), [PRS-TIMER-PRODUCER-2026-11-26.md](../../../../docs/04-reference/PRS-TIMER-PRODUCER-2026-11-26.md)

## What is not proven

- C -- advanced | complete to stage 2 | `adc`, `dma`, `emu`, `rtc`/`sysrtc`, `wdog`, `gpcrc`, `mvp`, `letimer`, `prs`; DMA stages 3 and 4 deferred |. (PLAN.md (`../../../../docs/03-development/PLAN`))
- `prs` | peripheral reflex system: routes an event from one peripheral to another without the CPU | ~250 lines, of which the crate now drives a channel: `prs::Source` names the producers it encodes (`Off`, `Timer0Overflow`, `Timer0Compare0`, `Timer1Overflow`, `GpioPin`), `Prs::connect` writes `ASYNC_CHn_CTRL` whole and `Prs::channel_ctrl` reads it back, and `Prs::pulse` is the software edge. (COVERAGE-BOUNDARY-2026-11-25.md (`../../../../docs/04-reference/COVERAGE-BOUNDARY-2026-11-25`))
- PRS routing | No hardware trigger path between timers, the IADC and the radio; needed for 802.15.4 timing. (HAL-RADIO-BLE-REVIEW-2026-11-17.md (`../../../../docs/04-reference/HAL-RADIO-BLE-REVIEW-2026-11-17`))

## Using it

See `examples/hal_adc_prs_gpio.rs`, `examples/hal_adc_prs_timer.rs`, `examples/hal_adc_prs_trigger.rs`, `examples/hal_pcnt_gpio.rs`
