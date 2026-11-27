# letimer

`LETIMER0`: the timer that keeps counting while the core is asleep.

## What it is

`LETIMER0`: the timer that keeps counting while the core is asleep.

Every other timer in this crate needs the bus clock, which means the core is awake or the count stops. `LETIMER0` runs
from the low-frequency domain and keeps counting in EM2 and EM3, so it is the thing a low-power design wakes on: a
compare match every second, every minute, or once an hour, with the core asleep in between.

# What this driver encodes, and why

Eight facts were measured on the part (2026-11-26, records in `../../../../docs/../../../../docs/04-reference/LETIMER-TOP-2026-11-26.md` and
`../../../../docs/../../../../docs/04-reference/LETIMER-COMPARE-2026-11-26.md`) and each one is a line of the code below:

1. **`EN` is the gate.** `CLKEN0` alone leaves the module answering on the bus and never counting.
2. **`CTRL.CNTTOPEN` is the switch that makes `TOP` mean anything.** At its reset value the top is the whole 24-bit
counter (`0xFFFFFF`, not the 65535 the manual's bit table claims) and `TOP` is ignored; set, the period is
`TOP`+1 ticks and `TOP` is the top. This driver sets it, always, before returning.
3. **The counter is 24 bits.** It is exposed here as [`Letimer::counter`] returning `u32`; the `u16` this crate used
before truncated it and made a real top of `0xFFFFFF` look like a top of 65535 -- the whole defect that hid the
switch in the first place.
4. **A compare at or above `TOP` never fires.** Both `set_top` and `set_compare` refuse that combination instead of
programming a flag that cannot be reached.
5. **`SYNCBUSY` is never polled.** Reading it on this peripheral stalls the bus -- measured three times, with the
branch on LFXO, on LFRCO, and before a branch was selected at all. The registers still cross into the low-frequency
domain, so the caller settles with its own clock: [`SETTLE_US`] is how long that takes.
6. **Writing to `IF` sets a flag; clearing goes through `IF + 0x2000`.** A write of `COMP0|COMP1|UF` to `IF` left
`0x7` behind, and the run's flags never cleared until the write went to the alias. [`Letimer::clear_compares`] and
[`Letimer::clear_flags`] use the alias.
7. **The counter counts down, so `COMP1` fires after `COMP0`** when its value is lower. Measured: `COMP0` = 500 fired
501 ticks after the reload and `COMP1` = 250 fired 250 ticks after that. And a compare flag set while the counter
is still below the compare value cannot be cleared -- it is the comparison, not a latch -- so a caller that wants
to see a clear takes it after the wrap.
8. **`TOP` read back is not trustworthy while the timer is stopped.** Two of four runs returned `0` for a `TOP` that
had been written as 1000 and settled; the same register read once the counter was running returned 1000 every
time. That is why the compare validation uses [`Letimer::programmed_top`] and the register is reported separately.

# What is deliberately absent

The prescaler stays at its reset value (divide by one) and only the free-running count and the two compare channels
are exposed. Repetition (`REP0`/`REP1`), the PWM outputs and the PRS trigger are in the PAC and are not driven here
yet: none of them has an instrument in this crate, and a driver written without one is a claim.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Channel` | enum | — |
| `Error` | enum | — |
| `Letimer` | struct | `LETIMER0`, owned by one caller. |
| `SETTLE_US` | const | How long a caller must let pass after a write that crosses into the low-frequency domain: two clock cycles at 32.768 kHz, rounded up, and nothing more. |
| `TICKS_PER_SECOND` | const | How many ticks a second holds with the prescaler at its reset value and LFRCO (32.768 kHz) as the source. |
| `clear` | fn | Reload the counter from `TOP`. |
| `clear_compares` | fn | Clear the compare flags. |
| `clear_flags` | fn | Clear every flag, including the underflow that a wrap sets. |
| `clock_readback` | fn | — |
| `compare` | fn | — |
| `compare_fired` | fn | — |
| `counter` | fn | The counter's value, 24 bits, read through the low-frequency boundary. |
| `flags` | fn | — |
| `programmed_top` | fn | — |
| `set_compare` | fn | Set a compare value, in ticks from the reload. |
| `set_top` | fn | Set the top, which is where the counter reloads and how long one period lasts: `TOP`+1 ticks. |
| `start` | fn | Start counting from `TOP` downwards. |
| `stop` | fn | Stop counting. |
| `top` | fn | — |
| `wrapped` | fn | — |

## Registers it touches

`CmuS`, `Letimer0Ns`

## What was measured

```
    letimer: clk=0x00000101 top0=1000 top=1000 comp0=500 comp1=250 refused=1 cnt0=1000 p1=501 qe=0 ql=1 pq=250 wrap=1 if1=0x00000001 ifc=0x00000000 if2=0x00000001 p2=499 cnt=499
    letimer: clk=0x00000101 top0=0    top=1000 comp0=500 comp1=250 refused=1 cnt0=1000 p1=501 qe=0 ql=1 pq=250 wrap=1 if1=0x00000001 ifc=0x00000000 if2=0x00000001 p2=499 cnt=499
    letimer: clk=00000401 want=500 rb=500 fired1=0 rtc1=1600 fired2=0 rtc2=1600 cnt_start=65525 cnt=62315
```

Controls and the full story: [LETIMER-COMPARE-2026-11-26.md](../../../../docs/04-reference/LETIMER-COMPARE-2026-11-26.md), [LETIMER-TOP-2026-11-26.md](../../../../docs/04-reference/LETIMER-TOP-2026-11-26.md)

## What is not proven

- C -- advanced | complete to stage 2 | `adc`, `dma`, `emu`, `rtc`/`sysrtc`, `wdog`, `gpcrc`, `mvp`, `letimer`, `prs`; DMA stages 3 and 4 deferred |. (PLAN.md (`../../../../docs/03-development/PLAN`))

## Using it

See `examples/hal_letimer_compare.rs`
