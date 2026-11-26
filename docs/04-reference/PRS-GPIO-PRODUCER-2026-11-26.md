# A GPIO pin as a PRS producer, measured

2026-11-26. Board: XIAO MG24 Sense (EFR32MG24B220F1536IM48-B) on the bench, flashed with
`crates/efr32mg2x-hal/examples/hal_adc_prs_gpio.rs` and read back through the mailbox.

## Why

`prs::Source::GpioPin(n)` was encoded from the vendor's signal table (`SOURCESEL = 0x04`, `SIGSEL = n`) and had
never been driven. The two producers measured before it were the CPU's own pulse (`ASYNC_SWPULSE`) and TIMER0's
overflow. A pin is the producer a product actually has: a button, a sensor's interrupt line, a reed switch.

There is no hand on the bench and no jumper wire, so the pad had to be moved by the chip itself. In the pull mode the
output latch selects the resistor instead of a driver, so writing it high ties the pad towards VDD and writing it low
pulls it towards ground. On the two pads used here that works: eight toggles, eight observed changes of `DIN`.

## The register encoding, confirmed by readback

An external interrupt is not (port, pin) in a single field. Both the 2025.6.2 SVD and the vendor header
(`efr32mg24_gpio.h`) encode it as two nibbles:

| register | field | width | meaning |
|---|---|---|---|
| `EXTIPSELL` (0x400) | `EXTIPSELn`, bits `4n+1:4n`, n = 0..7 | 2 | port of channel n: A=0, B=1, C=2, D=3 |
| `EXTIPSELH` (0x404) | same fields | 2 | the same for channels 8..15 |
| `EXTIPINSELL` (0x408) | `EXTIPINSELn` | 2 | pin of channel n, as its index inside its group of four |
| `EXTIPINSELH` (0x40C) | same fields | 2 | the same for channels 8..15 |
| `EXTIRISE` (0x410) | one bit per channel | 12 | rising edge arms the interrupt flag |
| `EXTIFALL` (0x414) | one bit per channel | 12 | falling edge arms the interrupt flag |

The constraint this puts on a caller is the one emlib spells out in `GPIO_INTNO_PIN_VALID(intNo, pin)`:
`(intNo & ~3) == (pin & ~3)`. A channel can only watch a pin in its own group of four, so binding `channel = pin` is
the choice that is always legal, and that is what `Board::watch_edges` does.

The probe used channel 3 on port C, pin 3 — one of each field non-zero — and read back

```text
sel=0x00002000 isel=0x00003000
```

`0x2000` is port field 2 (PORTC) at bits 12:13, `0x3000` is pin field 3 at bits 12:13 of `EXTIPINSELL`. The second
pad, PA0 on channel 0, is the all-zero case of the same encoding and behaved identically.

## What it measured

```text
adcgpio: ch=3 sel=0x00002000 isel=0x00003000 din=1 a=0 b=4 c=4 mv=8 up=4 off=0 other=0 pin0=4 up0=4 long=8 upg=8 fall=4 ifb=0x00000009 ifc=0x00000001 v0=0x00000231 vl=0x0000023a
adcgpio: ch=3 sel=0x00002000 isel=0x00003000 din=1 a=0 b=4 c=4 mv=8 up=4 off=0 other=0 pin0=4 up0=4 long=8 upg=8 fall=4 ifb=0x00000009 ifc=0x00000001 v0=0x00000233 vl=0x0000023a
```

Two runs, `status=PASS`, differing only in the two sampled values (PC9's microphone bias and its noise).

| phase | configuration | result |
|---|---|---|
| A | channel 3 bound to PC3, armed, pad quiet | `a=0` conversions |
| B | eight pad changes, flag never cleared | `b=4` |
| C | eight pad changes, flag cleared after each | `c=4`, `mv=8`, `up=4` |
| D | channel `Off`, eight changes on the same pad | `off=0` |
| E | channel 3 armed, eight changes on the *other* pad | `other=0` |
| F | channel 0 on PA0, eight changes on PA0 | `pin0=4`, `up0=4` |
| G | channel 3, sixteen pad changes | `long=8`, `upg=8` |
| H | channel 3 with `EXTIRISE` cleared, only `EXTIFALL` armed, eight changes | `fall=4` |

## The three facts worth carrying forward

1. **The producer is a level, not an edge.** Eight pad changes are four rising edges and four falling ones, and the
   converter fired four times (`c=4 up=4`). Sixteen changes gave eight (`long=8 upg=8`), so the count follows one
   direction of movement rather than losing a fixed share to a FIFO. The IADC's `TRIGGER.SINGLETRIGSEL = PRSPOS` takes
   the rising edge of that level.

2. **`EXTIRISE`/`EXTIFALL` are not in the PRS path.** Phase H cleared the rising mask for the channel and armed only
   falling, and the count did not move (`fall=4`). The masks configure the GPIO interrupt flag, a separate consumer of
   the same edge detector. A caller who wants an interrupt and a PRS producer on one pin configures both, and neither
   depends on the other.

3. **The flag is not the signal, but the clear works.** B (no clearing) and C (clear after each edge) agree at four,
   so the asynchronous signal does not carry the latch. The clear itself is real: `ifb=0x9` — channels 0 and 3, the
   channel-0 bit set by the other pad's initial pull-up write — became `ifc=0x1` after `clear()` on channel 3, so the
   alias at `IF + 0x2000` clears the channel named and no other. That is the same `+0x2000` clear window LETIMER
   needed, now measured on a second block.

## Controls, and why they are the point

`a=0` (armed, pad quiet), `off=0` (channel pointed at nothing, pad moving) and `other=0` (channel armed, the *other*
pad moving) are three different ways for this experiment to produce nothing, and it produced nothing in all three.
Without them a count of four could have been an artefact of arming, of the pad moving at all, or of the converter
free-running.

## The delivery this produced

- `pin::EdgeWatch` — binds channel = pin, both edges, clears the flag, reports `IF` and the select words.
- `pin::Input::set_pull_up` / `is_high` — the pull resistor as a pad mover, and the readback that proves the pad moved.
- `Board::watch_edges` — the handout, so an example never names the PAC.
- `prs::Source::GpioPin(n)` now refuses `n > 7`: the vendor signal table names `PRS_ASYNC_GPIO_PIN0 .. PIN7` and
  nothing above, and `SIGSEL` is three bits wide. An external interrupt on a pin in 8..15 exists, but there is no PRS
  signal to carry it.
- `examples/hal_adc_prs_gpio.rs` — the probe above.

## Not measured

A pad driven by a wire, by a button or by another peripheral; the fastest rate a GPIO producer can pace the converter
(the pull resistors are the limit here, not the path); and channels 8..15, which have no PRS signal.