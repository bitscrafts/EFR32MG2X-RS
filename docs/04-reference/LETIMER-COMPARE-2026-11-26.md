# LETIMER0's compares, measured: the flags are set by a write, and the counter runs down

2026-11-26, the same night as [`LETIMER-TOP-2026-11-26.md`](LETIMER-TOP-2026-11-26.md). That note settled which register
decides the top. This one is about the two compare channels: which one fires first, how a flag is cleared, and what a
`TOP` readback is worth. All three answers contradicted what the first version of the instrument assumed.

## The record, exactly as the board produced it

Two runs of the same image, both `PASS`:

```
letimer: clk=0x00000101 top0=1000 top=1000 comp0=500 comp1=250 refused=1 cnt0=1000 p1=501 qe=0 ql=1 pq=250 wrap=1 if1=0x00000001 ifc=0x00000000 if2=0x00000001 p2=499 cnt=499
letimer: clk=0x00000101 top0=0    top=1000 comp0=500 comp1=250 refused=1 cnt0=1000 p1=501 qe=0 ql=1 pq=250 wrap=1 if1=0x00000001 ifc=0x00000000 if2=0x00000001 p2=499 cnt=499
```

The instrument is `hal_letimer_compare`, now in `crates/efr32mg2x-hal/examples/`. The reference clock is the SYSRTC at
32.768 kHz, against which this crate measured the HFXO (`hal_hfxo`: a 1000 ms delay came back as 1006 ms). The
configuration: `TOP` = 1000 (a period of 1001 ticks), `COMP0` = 500, `COMP1` = 250, and one deliberate refusal — a
compare of 2000, above the top.

## What each field says

| field | value | what it means |
|---|---|---|
| `clk` | `0x101` | `CLKEN0.LETIMER0` = 1, `EM23GRPACLKCTRL.CLKSEL` = 1 (LFRCO) — the branch the driver selects |
| `top0` | `1000` / `0` | `TOP` read while the timer was stopped, after a settle and a double read: **not reliable** |
| `top` | `1000` | the same register read once the counter was running: the value that was written |
| `comp0`, `comp1` | `500`, `250` | both compares hold what was asked for |
| `refused` | `1` | the driver refused a compare above the top, as it must |
| `cnt0` | `1000` | the counter loaded from `TOP`, not from the 24-bit reset top |
| `p1` | `501` | `COMP0` fired 501 SYSRTC ticks after the reload, i.e. the programmed 500 plus the poll's own latency |
| `qe` | `0` | `COMP1` had **not** fired when `COMP0` did |
| `ql`, `pq` | `1`, `250` | `COMP1` fired 250 ticks after `COMP0` |
| `wrap` | `1` | the counter passed the wrap before the clear |
| `if1` | `0x1` | at the first match, only `COMP0`'s flag was set |
| `ifc` | `0x0` | after the clear, through the alias: zero |
| `if2`, `p2` | `0x1`, `499` | the next match, one period after the clear |
| `cnt` | `499` | the counter's own position at the second match, consistent with `p2` |

## The three findings

1. **The counter counts down, so the compare with the lower value fires last.** `COMP0` = 500 fired 501 ticks after the
   reload; `COMP1` = 250 fired 250 ticks *after that*. The first instrument asserted the opposite order and recorded
   `q1=0`, which reads as "the second channel is dead" but was true. A down-counter reaches 500 before 250, and an
   instrument that does not say which direction it counts will report a working channel as broken.

2. **Writing one to `IF` does not clear a flag on this family: it sets one.** After a run that left `IF = 0x1`, the
   driver wrote `COMP0|COMP1|UF` to `IF` and read back `0x7` — exactly what was written. The next wait then ended
   immediately (`p2=0`), because the flag it was waiting for was already set. The clear is the write-1-to-clear alias at
   `IF + 0x2000`, the same window the I2C driver was forced onto (`docs/04-reference/I2C0-READ-DEFECT.md`). With the
   write going there, `ifc=0x0` and the following match arrived 499 ticks later. This is worth stating plainly because
   the vendor's SVD has no `IF_CLR` entry for `LETIMER0`: the alias is the convention of the family, not a register the
   PAC will show you.

3. **A compare flag cleared while the counter is still below the compare value is set again immediately.** The flag is
   the comparison, not a latch: with the counter at ~500 and `COMP0` = 500, a clear reads back as set. The instrument
   therefore clears after the wrap, when the counter has been reloaded above both compares, and the clear is then
   observable (`ifc=0x0`). A driver that documents "clear the flag" without saying where the counter has to be is a
   driver whose users will see matches they cannot clear.

## `TOP` read back, while stopped

`TOP` written as 1000 and read after a settle returned **0 in two of the four runs** that read it while the module was
stopped, and **1000 in all three runs** that read it once the counter was running. The writes had landed in every case —
the counter loaded 1000 and the period was 1001 ticks. Two consequences, both now in the driver:

- validating a compare against a freshly-read `TOP` can refuse a compare that is perfectly reachable, because the read
  can return 0 — so `set_compare` validates against the value the driver programmed, and reports it separately
  (`programmed_top`);
- a `0` from a `TOP` readback on a stopped timer is not evidence that the write failed.

## What the driver now does, from these facts

- `CTRL.CNTTOPEN` is set at construction, so `TOP` always means the top.
- The counter is `u32`, 24 bits, and a compare at or above `TOP` is refused rather than programmed into a flag that can
  never be reached.
- Flags are cleared through `IF + 0x2000`, never by writing to `IF`.
- No `SYNCBUSY` poll anywhere: the caller settles with its own clock, and `SETTLE_US` is the constant that says how long.