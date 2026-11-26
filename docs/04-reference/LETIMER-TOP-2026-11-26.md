# LETIMER0: what decides the top, measured

2026-11-26. The question was narrow: `TOP` written as 1000 read back as 1000 and the counter still appeared to reload
from 65535, so either the write was being ignored or something else decided the top. It was the second, and the first
was an artefact of a 16-bit read of a 24-bit counter.

## The record, exactly as the board produced it

Two runs of the same image, identical output:

```
seq=1 status=PASS len=136
lt: clk=0x00000101 top=1000 comp0=4000 ctrl=0x00000200 A:cnt0=16777215 w1=0 w2=0 if=0x00000000 B:cnt0=1000 w1=1001 w2=2002 if=0x00000004
```

The probe is `hal_letimer_top_probe`, kept in `_trash/letimer-2026-11-26/` with the rest of this peripheral's work. One
image, two phases, so the treatment has a control measured in the same run:

| phase | `CTRL.CNTTOPEN` | `TOP` | `COMP0` | `CNT` after `START` | wrap 1 | wrap 2 | `IF` |
|---|---|---|---|---|---|---|---|
| A, control | 0 (reset) | 1000 | 4000 | 16777215 (0xFFFFFF) | none in 140000 RTC ticks | -- | 0 |
| B, treatment | 1 | 1000 | 4000 | 1000 | 1001 | 2002 | 0x4 (UF) |

`clk=0x101`: bit 0 is `CMU_CLKEN0.LETIMER0`, and bits 8..11 are `CMU_EM23GRPACLKCTRL.CLKSEL = 1` (LFRCO). The wrap
intervals are in SYSRTC ticks, 32.768 kHz, which this crate has measured against a host stopwatch.

## What the numbers say

1. **`CTRL.CNTTOPEN` is the switch.** At its reset value the top is the full counter width and both `TOP` and `COMP0`
   are ignored (phase A loads 0xFFFFFF and does not wrap). With the bit set, `TOP` decides: `CNT` loads 1000 and wraps
   every **`TOP`+1 = 1001** ticks, measured 1001 and 1001 apart in phase B. `COMP0` = 4000 was not the top -- if it had
   been, the period would have been 4001.
2. **The counter is 24 bits.** `CNT` loaded 16777215, not 65535. The manual's `CTRL` bit table says the top value is
   65535 when `CNTTOPEN` is clear; the silicon disagrees, and the register description of `CNT` ("bits 23:0") is the
   one that matches.
3. **A compare above the top never fires.** With `TOP` = 1000 and compares at 2000 and 4000, `IF` stayed at 0x4
   (underflow only) through both phases. `COMP0`/`COMP1` must sit below `TOP` or the flag is unreachable.
4. **The manual contradicts itself about `CNTTOPEN` and `TOP` wins.** The bit-table entry for `CNTTOPEN` says "the top
   value of the LETIMER is given by COMP0"; the `TOP` register page says "`TOP` will be used as Counter TOP Value if
   `CNTTOPEN` is set to 1", and section 18.12.1 shows `TOP` setting the period. The measurement agrees with the `TOP`
   page. The bit-table wording is the Series-1 description, where LETIMER had no `TOP` register and `COMP0` served as
   the top -- this part moved that job to `TOP` without rewriting the field text.

## The earlier reading, corrected

The record from the session before this one was

```
letimer: clk=00000401 want=500 rb=500 fired1=0 rtc1=1600 fired2=0 rtc2=1600 cnt_start=65525 cnt=62315
```

and it was read as "the counter reloads from 65535, so `TOP` is ignored". It does not. The driver at the time returned
`u16` from its counter read, and the numbers are the low 16 bits of a 24-bit count that had started at 0xFFFFFF:

```
0xFFFFFF - 10         = 16777205   & 0xFFFF = 65525   (0xFFF5)
0xFFFFFF - 10 - 3210  = 16773995   & 0xFFFF = 62315   (0xF36B)
```

Both match exactly. Nothing was ever ignored; the instrument had thrown away eight bits. The consequence of the real
top was also far worse than recorded: 0xFFFFFF ticks at 32.768 kHz is **512 s** per period, not the "about two seconds"
the earlier note assumed, which is why a compare at 500 never fired inside a 49 ms window.

## What the driver must do, from these facts

- Write `CTRL.CNTTOPEN = 1` before `TOP`, or the `TOP` write has no effect on the period.
- Expose the counter as `u32`/24-bit. A `u16` return silently truncates and produces readings that look like a
  different top -- the exact defect above.
- Keep both compares below `TOP`; a compare at or above it is unreachable. A `set_top` that accepts a value smaller
  than an already-programmed compare should be refused or stated, not silently accepted.
- The period is `TOP + 1` ticks, one tick per low-frequency clock with the prescaler at its reset value.

`SYNCBUSY` is still not polled anywhere: reading it on this peripheral stalls the bus, measured three times before. The
writes above were settled by the caller's own clock, and the measurement confirms they landed.