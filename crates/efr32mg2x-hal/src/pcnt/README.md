# pcnt

`PCNT0`: pulses counted by a peripheral, with the core doing nothing per event.

## What it is

`PCNT0`: pulses counted by a peripheral, with the core doing nothing per event.

## Why this module exists

The PRS producer measured on 2026-11-26 carries a pad's level, and an IADC converts once per rising edge of it
(`../../../../docs/../../../../docs/04-reference/PRS-GPIO-PRODUCER-2026-11-26.md`). That proves the signal reaches a consumer; it does not prove
anything **counts** it, and the converter is a poor counter: each conversion costs a FIFO slot, a pop and a
microsecond of settling. `PCNT0` is the block built for it — a pulse counter whose S0IN is one of the PRS consumers
(`CONSUMER_PCNT0_S0IN`), so the same edge that converted a sample also increments a register.

## What was measured, and what follows from it

- **The PRS path counts.** Eight pull moves of a pad (four rising edges) gave four, an odd number of moves split
between the two polarities gave the split exactly, and the four controls gave zero
(`../../../../docs/../../../../docs/04-reference/PCNT-GPIO-2026-11-26.md`).
- **The ceiling is a pulse-width limit, not a bandwidth.** `EM23GRPACLK` has only LFRCO, LFXO and ULFRCO in this
part, so the sampler cannot be moved faster than 32.768 kHz: a pulse narrower than 30.5 us is seen with probability
`width / 30.5 us` and usually not at all. A timer's overflow event is one clock wide (53 ns at 19 MHz) and gave
**zero** counts at 1 kHz, where 13 edges were produced -- measured twice, with two producers
(`../../../../docs/../../../../docs/04-reference/PCNT-CEILING-2026-11-26.md`).
- **The GPIO producer is the pad's input.** A pad the timer drives as an output is not on the path: driving PA0 at
50 % duty and reading `GpioPin(0)` back gave zero at every rate from 1 kHz to 25 kHz, with a high time seventeen
sampling periods long. A producer has to be a signal presented *to* the pin.
- **It is sampled, so it has a ceiling.** `CFG.MODE = OVSSINGLE` samples S0IN with the `EM23GRPACLK` branch, which
this board runs at 32.768 kHz. The ladder in the example finds the rate at which edges start being lost.
- **`CFG` is write-protected while `EN` is set**: writing it there raises a precise bus error at `PCNT0 + 0x0C`
(`CFSR = 0x00008200`, `BFAR = 0x5903_000C`) and halts the core. Every configuration write below therefore happens
with `EN` low, and the module waits for `EN.DISABLING` to clear both before and after.
- **`CMD` is synchronised and needs the wait too**: a second command inside one synchroniser period faults at
`PCNT0 + 0x14`. `SYNCBUSY.CMD` is the handshake.
- **`CFG.MODE = EXTCLKSINGLE` with `CLKSEL = 1` does not count S0IN**: it counts the `EM23GRPACLK` branch itself
(measured: 88 and 139 counts in windows sized at 2.6 and 4 ms, both ≈ 33 kHz, whatever the pad did). The
external-clock mode needs the dedicated `PCNT_S0` pin as the block's clock (`CLKSEL = 2`), and pointing that at a
routed GPIO left the block with no clock at all: `EN.DISABLING` stayed set and the next `CFG` write faulted. That
path is not offered here, because this crate only ships what the board demonstrated.
- **There is no capture command.** The plan for this item said `CNT` is read through `CMD.CAPTURE`; `CMD` has reset,
load, start and stop bits only and `CNT` has no writer. A running read returned the same value as the read after
`CMD.STOPCNT`, so the read is live.
- **`TOP` resets to `0x00FF`, not `0xFFFF`**: a counter left at reset wraps after 255 events, which on a fast
producer looks like a counter that cannot keep up. [`Pcnt::set_top`] is called before any count is believed.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Edge` | enum | — |
| `Error` | enum | — |
| `Pcnt` | struct | `PCNT0`, handed out once by `Board::pcnt`. |
| `count` | fn | The current count. |
| `count_edges` | fn | Count `edge`s of S0IN, from zero, starting now. |
| `flags` | fn | The raw `IF` word, for a record that has to show it rather than summarise it. |
| `init` | fn | Bring the block up clean: a core reset, and the module left **disabled**. |
| `overflowed` | fn | Whether the counter has wrapped past `TOP` since the last clear (`IF.OF`). |
| `reset` | fn | Zero the count without touching the running state or the configuration. |
| `running` | fn | Whether the counter is running right now (`STATUS.CNTRUNNING`). |
| `set_top` | fn | The counter's wrap value. |
| `start` | fn | Resume counting with the configuration already in place. |
| `stop` | fn | Stop counting. |
| `use_prs_input` | fn | Point S0IN at a PRS channel instead of the pad, and select `channel` as its source. |
| `words` | fn | `(CFG, CTRL, STATUS)`, so a probe can report the words it configured instead of the ones it meant to configure. |

## Registers it touches

`Pcnt0Ns`

## What was measured

```
    pcntrate: r1k=0/13 r5k=0/68 r10k=0/137 r16k=0/219 r20k=0/274 r50k=0/686 w=450 followed=0 ceiling=0k
    pcntpwm: r1k=0/13 r5k=0/68 r10k=0/137 r16k=0/219 r25k=0/343 w=450 followed=0 ceiling=0k
    pcnt: ch=3 cfg=0x00000100 ctrl=0x00000010 st=0x00000000 a=4 lv=4 b=2 c=3 off=0 stop=0 q=0 mm=0 p0=4 ok=25000 bad=50000 g=77 e=200 of=0 ar=4 br=2 cf=2 cm=4 hr=4 em=8 if=0x00000000
```

Controls and the full story: [PCNT-CEILING-2026-11-26.md](../../../../docs/04-reference/PCNT-CEILING-2026-11-26.md), [PCNT-GPIO-2026-11-26.md](../../../../docs/04-reference/PCNT-GPIO-2026-11-26.md)

## What is not proven

- PCNT0's ceiling: parked on external hardware.** The item asked for the ceiling between 25 and 50 kHz with a. (BACKLOG.md (`../../../../docs/03-development/BACKLOG`))

## Using it

See `examples/hal_pcnt_gpio.rs`
