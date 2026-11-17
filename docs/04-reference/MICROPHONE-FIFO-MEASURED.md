# The single FIFO is slower than the plain read — measured, 2026-11-17

## What was tried

`Microphone::sample()` issues `CMD.SINGLESTART` and waits on `STATUS.SINGLEFIFODV`, then reads `SINGLEDATA`
— the register the PAC describes as "latest single queue conversion data". A review suggested that reading
`SINGLEFIFODATA` instead ("read the OLDEST valid data from the single FIFO and pop the FIFO") would be both
more correct and cheaper: more correct because `SINGLEDATA` does not consume, cheaper because one status check
could serve a whole burst.

So a `capture(&mut [i16])` was written that pops the FIFO in bursts, and both paths were measured on the same
256-sample window, on the board, with the cycle counter.

## What was measured

```
sample() per element, 256 samples      10 043 cycles    39,2 cycles per sample
capture() popping the FIFO, 256        31 105 cycles   121,5 cycles per sample
data quality, both paths               distinct 41 and 64 values, first 2639 — real signal, not a constant
```

**The FIFO path is 3,1x slower.** It is not a measurement artefact: the FIFO is not queueing. With
`SINGLESTART` and no `SINGLEFIFOCFG` set up, one start produces one entry, so the "burst" path issues the same
number of starts as the plain path *plus* two status polls per sample. The extra work is exactly what the
numbers show.

## What this means

- The review's *mechanism* was right — `SINGLEDATA` does not pop, and `SINGLEFIFODATA` does — but the
  optimisation that followed from it does not hold without configuring the FIFO itself.
- `capture()` was removed from the driver rather than kept as a slower alternative with a nice name. A method
  called "capture in bursts" that costs three times what the simple call costs is a trap, and the next person
  would use it because of the name.
- Making the FIFO genuinely useful needs `SINGLEFIFOCFG`: FIFO enable, depth and a watermark, so one start
  fills several entries. That is a separate change, and it must arrive with its own before/after numbers —
  the same cycle counter, the same window, the same board.

## The cost that actually matters

39,2 cycles per sample is what the dosimeter has to budget for today:

```
at 303 148 Hz   11,9 M cycles/s = 62 % of a 19 MHz core    <- unusable
at  16 000 Hz    0,63 M cycles/s =  3,3 %                  <- the rate the features need
```

So the fix for the CPU is not a smarter read — it is **not oversampling**. Pacing the converter at the rate the
features use (16 kHz) is the change that takes sampling from 62 % of the core to 3 %.

## How to re-run this

The instrument was an example named `module_mic_burst` in `crates/xiao-mg24-bsp/examples/`. It measured both
paths, counted distinct values in each window, and passed only when both delivered real data. It was removed
together with `capture()`; recreate it from this document if the FIFO is configured properly later — the
numbers above are the baseline it has to beat.
