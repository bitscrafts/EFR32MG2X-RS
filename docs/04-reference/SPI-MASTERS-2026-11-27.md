# `hal_spi_masters` — the three SPI masters, measured on the board (2026-11-27)

## The record

```
spi3: n=8 eusart0=8 usart0=8 eusart1=8 cross=0 control=0 err=0      PASS
```

Three boots of the same image (`2aca49a` + this example), jumper **D9 (MISO, PA4) – D10 (MOSI, PA5)** fitted,
SCLK on D8 (PA3).

## What was being proved

`spi::Spi0` (USART0) and `spi::Spi2` (EUSART1) were public, documented and unbuildable: their only constructors took a
`&FrozenClocks`, and `Board` keeps the CMU private. `Board::spi0_on` and `Board::spi2_on` close that, and this instrument
is what turns "it builds" into a record. It drives one block at a time, through that block's own route registers.

## The control, and the one that could not fail

Two controls were tried and only the second is worth anything.

**The first** re-routed MISO to a pad the jumper does not reach, on the theory that a receiver with no MISO route reads
nothing. With the jumper fitted it read `control=8`. The measurement was correct and the control was worthless: the wire
ties D9 to D10 physically, so the signal arrives whether or not the internal route points at it. An instrument whose
control cannot fail certifies nothing.

**The second** rests on two properties:

1. each master sends eight bytes that share **no byte** with the other two patterns
   (`A5 5A 00 FF 0F F0 12 34`, `3C C3 81 18 24 42 99 66`, `01 02 04 08 10 20 40 80`), and the record counts the bytes each
   master received that belong to *another* master's pattern. A receiver silently reading a neighbour's line scores
   zero on its own pattern instead of eight;
2. the control sends `0xEE`, a byte no master ever sends. Whatever the receiver reads — the driven wire or a floating
   line — it matches none of the three patterns.

So `cross=0` and `control=0` are both falsifiable, and `eusart0=8 usart0=8 eusart1=8` is a claim about three different
drivers rather than a claim that something drove a wire.

## What this does not say

- Nothing about a second device: the "slave" is the same chip in loopback.
- Nothing above 1 MHz. The frequency is `1_000_000` Hz so that a breadboard jumper is not the limiting factor.
- Nothing about SPI slave mode. The drivers are master only, and §3 of the coverage boundary says why.

## The tell, for the next time

Before the jumper existed, the same image read `eusart0=1 usart0=1 eusart1=1 control=1`: one byte everywhere. That is
what a floating MISO looks like against a pattern that contains one `0xff` — and it was *also* what a broken driver
would have looked like. The two were told apart by re-measuring the **proven** instrument, `hal_spi_loopback`, which
regressed from its own 2026-11-25 record (`ok=8`) to `in=00..00` in the same session. A new instrument never gets to
declare an old one wrong; the old one is what said "the wire is missing".
