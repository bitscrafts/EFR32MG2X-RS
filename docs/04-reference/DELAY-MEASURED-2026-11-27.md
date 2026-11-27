# The delay, measured instead of inferred

2026-11-27. `delay_ms(10)` was reported at 4.35 ms on 2026-11-12, and that number had stood in `STATUS.md`,
`BACKLOG.md`, two crate documents and three research notes ever since. It was produced by counting DWT cycles across a
delay and dividing by a clock frequency that `emlib` reported. This page replaces it with a measurement of the delay
against a clock that is not the delay: **TIMER0 counting the 39 MHz crystal**, through a 2^10 prescaler, so that one
count is 26.3 us and a 200 ms interval fits in a single reading with no wrap to count.

Instrument: `crates/efr32mg2x-hal/examples/hal_delay_clock.rs`.

## The record

Three flashes, three boots, byte-identical except where noted, all `status=PASS`:

```
delayclk: ok=1 hfxo=1 hz0=19000000 d0=200279 u0=99125 hz1=39000000 pw=10003 r1=38988303 em=200 d1=201124 e1=5 u1=100010 eu=0 x=401171 ex=2
delayclk: ok=1 hfxo=1 hz0=19000000 d0=200279 u0=99125 hz1=39000000 pw=10003 r1=38988303 em=199 d1=201150 e1=5 u1=100010 eu=0 x=401145 ex=2
delayclk: ok=1 hfxo=1 hz0=19000000 d0=200279 u0=99178 hz1=39000000 pw=10003 r1=38988303 em=200 d1=201124 e1=5 u1=100010 eu=0 x=401171 ex=2
```

| field | meaning | measured |
|---|---|---|
| `hz0` | rate the HAL declares on the reset clock | 19 000 000 |
| `d0`, `u0` | us a `delay_ms(200)` / `delay_us(100_000)` took there | 200 279 / 99 125 |
| `hz1` | declared rate with the core on the crystal | 39 000 000 |
| `pw` | us of one SysTick period of 390 000 ticks, armed as the delay arms it | 10 003 |
| `r1` | the rate `pw` implies for SysTick | 38 988 303 |
| `em` | ms the delay's own algorithm takes for `delay_ms(200)`, same image | 199..200 |
| `d1`, `u1` | us a `delay_ms(200)` / `delay_us(100_000)` took at the crystal | 201 124..201 150 / 100 010 |
| `e1`, `eu` | their errors per mille against 200 000 / 100 000 | 5 / 0 |
| `x`, `ex` | the control: `delay_ms(200)` on a `Delay` told 78 MHz | 401 145..401 171, 2 per mille of 400 000 |

## What it settles

1. **The delay is right.** 200.3 ms for a requested 200 ms on the reset clock, 201.1 ms on the crystal, 100.010 ms for
   100 ms. The 2.3x-short figure does not reproduce, and neither does the "3.0x" of the reverted calibration attempt:
   `em=199..200` shows the code path that was blamed for both was doing its job in this image.
2. **SysTick counts the core clock, undivided.** 390 000 ticks took 10 003 us with the core on the crystal: 38.99 MHz,
   which is the crystal. There is no hidden divider to explain anything.
3. **The wrap-based arming is not a defect on this part.** `set_reload` + `clear_current` + wait-for-wrap produced a
   full 10 ms period, not the one-tick period that the Cortex-M documentation of `SYST_CVR` ("a write clears it to
   zero") would predict. An earlier attempt to fix this defect was made on the theory that the arming was wrong; the
   theory is testable here (`pw`) and it does not hold.
4. **The declared rate is what the conversion follows.** A `Delay` told the core runs at 78 MHz asks for twice the
   ticks and takes 401 ms for the same 200 ms request. That is the control that makes the rest falsifiable: a delay
   that ignored the declared rate would answer 201 ms there too.
5. **What the old claim actually measured.** `emlib_clock` counts DWT cycles across `delay_ms(10)` and divides by
   `emlib`'s clock reading. Re-run on 2026-11-27 it reports
   `emlib: sys=19000k core=19000k meas=2659k` -- `FAIL`. The delay in that same image is 10 ms by the crystal, so the
   number on trial is the counter, not the delay: DWT advanced 26 590 cycles across a measured 10 ms, i.e. 2.66 MHz,
   where the core is at 19. The 8.27 MHz of the 2026-11-12 run and today's 2.66 MHz are two readings of a counter that
   is not clocked at the core rate in a run started without a debugger attached
   ([`DWT-NEEDS-A-DEBUG-SESSION.md`](DWT-NEEDS-A-DEBUG-SESSION.md)); neither names a core clock. The clock the delay
   needs is the one `Board::delay()` already passes it, `clock_hz()`.

## What this run got wrong, and why it is written down

The first version of the instrument reported zero for every duration and passed its own arming check, because it
converted TIMER0 counts to **milliseconds** and printed them as **microseconds**. That is exactly the shape of the
defect being hunted -- a delay that returns immediately -- and it survived two flashes before the `p` and `r` fields
were compared. The record now prints the implied rate beside every duration, and the control carries its own expected
value, which is what caught it: an instrument whose two statements of the same interval cannot disagree is not an
instrument.

## What is not measured here

- `delay_ns` below 1 us: that path is a `cortex_m::asm::delay` loop and it was not timed.
- Any delay with an interrupt that sleeps longer than one 24-bit SysTick wrap (0.43 s at 39 MHz).
- EM2/EM3, where SysTick is not clocked, and temperature.
- The 5 per mille residual on the 200 ms call. It is not the instrument's 26 us quantisation (that is 0.03 per mille
  on the same interval) and it is not the conversion, because the 100 ms call on the same conversion is 0 per mille
  out. It is recorded as measured rather than explained.

## Where this is referenced

- `crates/efr32mg2x-hal/examples/hal_delay_clock.rs` -- the instrument
- `crates/efr32mg2x-hal/src/board.rs` -- `timer0_free_running_prescaled`, the prescaled counter this needs
- `docs/04-reference/DELAY-CLOCK-DEFECT.md` -- the earlier inference, now closed at the top
- `docs/03-development/STATUS.md`, `docs/03-development/BACKLOG.md` -- the claim removed from both