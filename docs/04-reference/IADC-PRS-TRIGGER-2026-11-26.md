# The IADC converts on a PRS edge, measured: eight pulses, eight conversions, and three controls

2026-11-26. Until tonight every conversion this crate produced came from a CPU write to `CMD.SINGLESTART`. The
converter's `TRIGGER.SINGLETRIGSEL` field also selects a PRS positive edge, and nothing in the tree had ever written
it — so "the ADC can be triggered by PRS" was a claim read off an SVD comment, not a fact.

## The record, exactly as the board produced it

The instrument is `hal_adc_prs_trigger`. Two runs of the same image, both `PASS`, differing only in the sampled values
(PC9's microphone bias, which is noise around mid-scale):

```
adcprs: trig=prs0 refused=1 armA=1 imm=0 armB=0 prs=8 v0=0x0000026f v7=0x000002a7 quiet=0 noarm=0
adcprs: trig=prs0 refused=1 armA=1 imm=0 armB=0 prs=8 v0=0x0000026c v7=0x000002a9 quiet=0 noarm=0
```

And the raw register words, from the bench probe that ran before the driver changed
(`_trash/prs-2026-11-26/hal_adc_prs_probe.rs`):

```
adcprs: ch0=0x000c0000 cons=0x00000000 tA=0x00000000 armA=1 imm=0 tB=0x00000300 armB=0 prs=8 v0=0x00000268 v7=0x000002a7 quiet=0 if=0x00000201
adcprs: ch0=0x000c0000 cons=0x00000000 tA=0x00000000 armA=1 imm=0 tB=0x00000300 armB=0 prs=8 v0=0x00000267 v7=0x000002a5 quiet=0 if=0x00000201
```

## The four phases, and why three of them are controls

| phase | trigger | armed | pulses | conversions |
|---|---|---|---|---|
| A | software (`TRIGGER = 0x0`) | yes | 8 | `armA=1`, `imm=0` |
| B | PRS positive edge, channel 0 (`TRIGGER = 0x300`) | yes | 8 | `armB=0`, `prs=8` |
| C | PRS positive edge, channel 0 | yes | 0 | `quiet=0` |
| D | PRS positive edge, channel 0 | **no** | 8 | `noarm=0` |

Phase A is the control that a count is not an artefact of arming a single queue: with the immediate trigger, the arm
converts exactly once and the eight pulses add nothing. Phase C is the control that the count is not an artefact of
writing to `ASYNC_SWPULSE` at all. Phase D is the control that the edges need an armed queue behind them: eight pulses
into an unarmed converter convert nothing. Only with all three does "one edge, one conversion" follow from phase B.

`refused=1` is the driver's own refusal, not a measurement: `Adc::read` returns `Error::Triggered` while PRS owns the
start, because arming and polling there would wait for an edge nobody sent and look like a dead channel.

## What each register says

| register | value | meaning |
|---|---|---|
| `TRIGGER` | `0x00000300` | `SINGLETRIGSEL` = 3 (`PRSPOS`, bits 8:10) and `SINGLETRIGACTION` = 0 (`ONCE`, bit 12) |
| `PRS.ASYNC_CH0_CTRL` | `0x000c0000` at reset | `FNSEL` = `A` (bits 16:19); `SOURCESEL` and `SIGSEL` are 0. **No write was needed**: a write to `ASYNC_SWPULSE` is XORed onto the channel output, so a channel that selects no source still carries the edge |
| `PRS.CONSUMER_IADC0_SINGLETRIGGER` | `0x00000000` | `PRSSEL` = 0: channel 0 feeds the single trigger. Written with the channel number, so a caller asking for another channel is not silently given channel 0 |
| `IADC.IF` at the end | `0x201` | `SINGLEFIFODV` (bit 0) and `SINGLEDONE` (bit 9) — set. No overflow (bit 16), no underflow (bit 18) |

## The fact that made the first run look wrong

The first version of the probe read **zero** for every conversion and reported `PASS`, because the count was the
measure. The register dump showed `IF = 0x2201`: bit 13, `PORTALLOCERR`. The input was PA0, and an odd/even pin only
reaches `ADC0` when `GPIO.CDBUSALLOC` gives it the matching CD bus. The conversion still "completed" and still counted —
it returned 0 and raised a flag nobody was reading.

Two consequences:

- the instrument now reads PC9 (the microphone), with the odd CD bus allocated, and the value is 0x271..0x2a7 rather
  than 0 — a conversion that really sampled the pin;
- `Adc::select_input` documents that the analogue bus is the caller's business and names `IF.PORTALLOCERR` as the flag
  that turns a silent wrong answer into a visible one.

## What the driver does now

- `adc::Trigger` — `Software` (the default, every existing caller) and `PrsPositiveEdge(channel)`.
- `Adc::use_prs_trigger(channel)` — writes `TRIGGER` and the consumer channel with `EN` down, then brings the converter
  back up. Channels above 15 are refused: `PRSSEL` is four bits.
- `Adc::use_software_trigger()` — the way back, so the default path stays reachable after a triggered run.
- `Board::prs()` and `prs::Prs::pulse(channel)` — the block, its bus clock, and one write to `ASYNC_SWPULSE`. No
  channel configuration, because the measurement showed none was needed; the `Prs` doc says so and says what is missing.
- `Board::allocate_odd_cd_bus()` — the line the microphone's path already wrote, now shared instead of duplicated.
- `Adc::arm_single()` and `Adc::take_single(polls)` — arming and popping separated, because under a trigger the CPU
  does the first and the hardware does the rest. `take_single` reads `SINGLEFIFODATA` (which pops), not `SINGLEDATA`.
- `Adc::read` / `read_on` return `Error::Triggered` while a PRS trigger is selected instead of timing out.
- `Adc::new` opens `CMU_CLKEN0.PRS` — the consumer word lives in the PRS block, and a block whose bus clock is off
  drops writes.

## What this does not prove

- **The PRS bus clock is necessary — measured only in the positive direction.** Every run had `CLKEN0.PRS` set before
  the first PRS write; nobody has run it with the gate closed to see the write disappear.
- **Producers other than the CPU's own pulse.** `ASYNC_SWPULSE` is the software producer. A timer or a GPIO edge
  arriving on the same channel is the next measurement, and it is what a PRS driver needs.
- **The other trigger modes.** `PRSCLKGRP` (a timer in the ADC's clock group), `PRSNEG` and `SINGLETRIGACTION =
  CONTINUOUS` are written nowhere and measured nowhere.
- **A rate.** This proves one edge, one conversion. What the highest sustainable edge rate is depends on the FIFO depth
  (one entry) and on how fast the reader pops it; that is a different instrument.