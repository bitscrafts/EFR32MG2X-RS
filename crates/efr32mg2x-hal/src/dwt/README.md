# dwt

The cycle counter, as a time base that cannot lie — and that says so when it cannot count.

## What it is

The cycle counter, as a time base that cannot lie — and that says so when it cannot count.

Every timing defect in this project was a *declared* interval that nobody measured: a loop that said 400 Hz and ran
at 158, an `asm::delay(1_000_000)` that was not a million cycles, a millisecond counter built by dividing
microseconds by 1000 and losing the remainder. The DWT cycle counter is the antidote: it counts CPU cycles, so the
interval between two samples can be *measured* rather than assumed.

## The counter only runs while a debug session is attached

Measured 2026-11-24, twice, with the host's clock as the referee: with the probe detached, two readings 10.8 s apart
returned the same value while the SYSRTC advanced 346 091 ticks; under `probe-rs run` the same firmware gave a
DWT/SYSRTC ratio of 578.4 against the 579.8 that 19 MHz / 32.768 kHz predicts. `DEMCR.TRCENA` and `CYCCNTENA` read
back set while the counter stands still: the writes land, and the counter is simply not clocked once the debug
domain powers down. Record: `../../../../docs/../../../../docs/04-reference/DWT-NEEDS-A-DEBUG-SESSION.md`.

That makes a bare reading a trap — it returns a frozen number, which reads as "the interval was zero" and looks like
a very fast loop. So the entry point is [`start`], which **proves the counter moves** before handing anything back,
and returns [`DwtError::NotRunning`] when it does not. A program that has to measure itself without a probe should
use SysTick or a peripheral counter instead; this module is an instrument, not a time source.

Only the board knows when this is safe: enabling the counter costs one write and nothing else in this crate touches
DWT.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `DwtError` | enum | — |
| `cycles` | fn | — |
| `cycles_to_us` | fn | — |
| `since` | fn | — |

## Registers it touches

None directly: this module composes other drivers and reaches no `pac::` peripheral of its own.

## What was measured

```
    probe: t=1865376137 c=29605      SYSRTC and DWT at the first read
    probe: t=1865722228 c=29605      same DWT value 10.8 s later
```

Controls and the full story: [DWT-NEEDS-A-DEBUG-SESSION.md](../../../../docs/04-reference/DWT-NEEDS-A-DEBUG-SESSION.md)

## What is not proven

Nothing in `docs/` records an unproven path for this module.

## Using it

See `examples/hal_sysrtc_wide.rs`
