# sleep

Entering EM2 -- and leaving it.

## What it is

Entering EM2 -- and leaving it.

## Why this is an API and not a `wfi` in an example

Entering EM2 is not sleeping: it turns the high-frequency clock off and leaves the part dependent on an interrupt
that is already armed. A loose `wfi` with `SLEEPDEEP` set and no wake-up source leaves the device asleep until
someone touches its reset -- and the symptom, seen from outside, is a board that died.

On this series the `EMU` has `CTRL.EM2DBGEN` and does **not** have an EM2 lock: what prevents entry is the state of
the rest of the system, not a bit. The two this module deals with:

- **the debugger**: with `EM2DBGEN = 0` the debug connection dies in EM2 and does not come back with the board
running -- [`Sleep::keep_debugger`] sets it before sleeping;
- **`SLEEPDEEP`**: it is left set during the `wfi` and cleared afterwards, so that the next `wfi` someone writes by
hand does not enter EM2 without knowing it.

## What this module does not do

It does not arm wake-up sources. The caller has to have the interrupt enabled in the peripheral **and** in the
`NVIC` before entering, or the `wfi` does not return. That is deliberate: choosing the wake-up source is the
product's decision, and a `sleep_ms` that decided for us would be the same class of assumption the `19_000_000`
was hiding in the timer driver.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Sleep` | struct | Entering EM2 and leaving it. |
| `enter` | fn | Sleeps in EM2 until an armed interrupt wakes the core. |
| `keep_debugger` | fn | Keeps the debug connection alive while the core is in EM2. |

## Registers it touches

None directly: this module composes other drivers and reaches no `pac::` peripheral of its own.

## What was measured

```
    pending: the interrupt that ended the sleep was the controller's. The handler cleared the flag (`af=0`), and the
```

Controls and the full story: [DMA-IRQ-2026-11-27.md](../../../../docs/04-reference/DMA-IRQ-2026-11-27.md)

## What is not proven

- ~~H28~~ | EM2 entry and exit sequence | **CLOSED 2026-11-21** (`B3`): ten ~1 s naps in EM2 woken by the SYSRTC comparator, intervals 0.92-1.03 s, mean 0.99, total 9.92 s, eleven interrupts. (HAL-GAP-ANALYSIS-2026-11-18.md (`../../../../docs/04-reference/HAL-GAP-ANALYSIS-2026-11-18`))

## Using it

No example imports this module yet, so its API has never been exercised from `examples/`.
