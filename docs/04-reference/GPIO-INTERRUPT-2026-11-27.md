# A GPIO edge as an interrupt, measured

2026-11-27. Board: XIAO MG24 Sense (EFR32MG24B220F1536IM48-B) on the bench, flashed with
`crates/efr32mg2x-hal/examples/hal_gpio_interrupt.rs` and read back through the mailbox.

## Why

`CLAUDE.md` carried one line for this: *GPIO: interrupts not yet implemented*. The edge detector was already measured
as a PRS producer (2026-11-26), where the pad's signal reached a converter without the CPU; what had never been done
was the other consumer of the same detector — `IEN`, the NVIC, and a handler that keeps the count.

## The chain, and which register at each link

| link | register / object | what it decides |
|---|---|---|
| the edge | `EXTIRISE` (0x410), `EXTIFALL` (0x414) | which transitions set the flag |
| the flag | `IF` (0x420), cleared through `IF + 0x2000` | that an edge happened, once, until acknowledged |
| the request | `IEN` (0x424), one bit per pin | whether the pin may request the vector |
| the vector | `GPIO_ODD_IRQn` = 25, `GPIO_EVEN_IRQn` = 26 | which handler runs: channel parity |
| the count | a static in the HAL, incremented by the handler | what the core actually served |

The two vectors are split by channel parity: channel 3 is delivered on `GPIO_ODD_IRQHandler`, channel 0 on
`GPIO_EVEN_IRQHandler`. This crate does not define either handler. It exposes `extint::service(parity)`, and the
application installs both — the same contract `SYSRTC_APP` already uses here, for the same reason: a library that takes
the part's vectors from an application is a library the application cannot re-arrange.

## What it measured

```text
gpioirq: n=8 if=0x00000008 cnt=8 off=0 dis=0 msk=0 odd=8 even=0 pin0=8 p0odd=0 p0even=8 a=0 up=4 sel=0x00002000 isel=0x00003000
gpioirq: n=8 if=0x00000008 cnt=8 off=0 dis=0 msk=0 odd=8 even=0 pin0=8 p0odd=0 p0even=8 a=0 up=4 sel=0x00002000 isel=0x00003000
```

Two flashes, two boots, `status=PASS`, byte-identical records. There is no sampled value in this record, so the board
has nothing to vary between runs.

| phase | configuration | result |
|---|---|---|
| A | channel 3 armed, pad quiet | `a=0` |
| B | channel 3, eight pad changes, both edges armed | `cnt=8`, `n=8`, `up=4`, `odd=8`, `even=0` |
| C | channel 3, both edge masks off, eight changes | `off=0` |
| D | channel 3, `IEN` bit cleared, eight changes | `dis=0` |
| E | channel 3, `IEN` set, vector masked in the NVIC | `msk=0`, `if=0x00000008` |
| F | channel 0 on PA0, eight changes | `pin0=8`, `p0even=8`, `p0odd=0` |

`n` is the number of pad changes **observed** by reading `DIN` after each write, not the number of writes the loop
asked for: a pad that never moved and a channel that counted nothing agree at zero, and that agreement is not a working
interrupt. `up=4` says the sequence started with a falling edge (the pin is configured with its pull-up, so the first
write pulls it down), which is why four of the eight changes are rising ones.

## The four facts worth carrying forward

1. **One pad change, one interrupt, one count.** `n=8 cnt=8` on both parities. The pull resistor slews the pad in
   microseconds and the writes are spaced 500 us, so this is not a rate measurement; it is a delivery measurement.

2. **`IEN` is a per-pin gate in front of the vector.** Phase D kept both edge masks armed and cleared only the pin's
   `IEN` bit: the count stayed at zero. So the masks alone do not reach the core — an implementation that configures
   `EXTIRISE`/`EXTIFALL` and unmasks the vector and stops there delivers nothing, silently.

3. **The flag is not the count.** Phase E left `IEN` set and masked the vector in the NVIC: `if=0x00000008`, channel
   3's bit alone, after eight changes. Eight edges produced one latched bit (the flag is a latch, not a counter), and
   the core counted nothing (`msk=0`) because it was never allowed to run. The piece that turns a latch into a count
   is the handler.

4. **The parity split is the vector's.** In B the odd vector entered eight times (`odd=8`) and the even vector not at
   all (`even=0`); in F, on channel 0, the reverse (`p0even=8`, `p0odd=0`). The entry counters are incremented by the
   handler that ran, so this is measured rather than assumed from the channel number.

## The rule the handler has to obey

The GPIO interrupt line is asserted while **any** flag of the parity is set. A handler that reads the flag, decides
the channel is not its business, and returns without acknowledging re-enters immediately, for ever — the same trap
`SYSRTC_APP` has with its overflow bit. `extint::service` therefore acknowledges every flag of its parity and counts
only the armed channels. `EdgeCounter::arm` clears the channel's flag again before it starts, so an acknowledgement
in a previous window cannot swallow the edge the next window is waiting for.

## Controls, and why they are the point

`a=0` (armed, pad quiet) refuses an interrupt that fires on its own. `off=0` (masks off) refuses an edge that never
reached the flag. `dis=0` (`IEN` cleared) refuses an edge that reached the flag but not the core. `msk=0` with a
latched flag refuses a count that came from anywhere other than the handler. Each one removes exactly one link of the
chain, and the good phase is the only one where all of them are in place.

## The delivery this produced

- `extint` module — `Parity`, `service`, `entries`, `EdgeCounter`.
- `pin::EdgeWatch::set_edges` — the edge masks rewritten in place, which is what the mask-off control needs.
- `Board::count_edges` — the handout, so an example never names the PAC.
- `examples/hal_gpio_interrupt.rs` — the probe above, with both handlers installed.

## Not measured

Latency from the edge to the first instruction of the handler, and rate: the pull resistors set the pace here, not the
vector. A pad driven by a wire or by another peripheral is not covered either. And whether `IEN` also gates the
`IF` latch is not separated from the vector: phase E keeps `IEN` set while masking the NVIC, so the two are tested
together and the flag is proven to latch with the core out of the path.