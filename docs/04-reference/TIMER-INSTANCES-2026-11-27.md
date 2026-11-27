# TIMER1–TIMER4, and the constructor nobody could call — 2026-11-27

The coverage boundary carried four rows reading "compiled, not measured" for `TIMER1`–`TIMER4`. The honest reading of
that line was never "nobody got round to it"; it was that **nothing in the crate could build one**. `timer::TimerN::new`
takes a `&FrozenClocks`, and `FrozenClocks` owns the CMU — which the `Board` owns. Every `Board` hand-out builds its
driver from a clock in hertz for exactly this reason (`usart::Usart0::new_at` says so, and `i2c` before it). `timer` was
the last driver still tied to the clock handle, and because of that the four instances beyond `TIMER0` were documented,
compiled, and dead.

## What was done

- `TimerN::new_at(timer, config, hfclk_hz)` — the same constructor without the handle, `new` delegating to it.
- `Board::timer1()` … `Board::timer4()` — one claim each (`Slot::Timer1` … `Slot::Timer4`), the `CLKEN0` gate, and the
  branch clock brought up before any register is touched.
- `Board::open_timer_branch()` — `CLKEN0.HFRCOEM23`, `FORCEEN`, wait for `STATUS.RDY`, `EM01GRPACLK.CLKSEL = HFRCOEM23`.
- `examples/hal_timer_instances.rs` — the instrument, and the record below.

## Two defects the measurement found, before it recorded anything

Both were in code that had never run, and neither could have been found by any single-timer instrument.

1. **The order of `EN` and `CFG` was inverted.** `CFG` faults while the timer is enabled — `pwm::Timer0::take` has
   carried that measurement since 2026-11-17. The unused constructor enabled the block first and configured it second,
   so every call was a bus fault. First run:

   ```
   Frame 1: HardFault <Cause: Escalated BusFault (Precise data access error) at location: 0x4004c004>
   Frame 2: Timer1::new_at   Frame 3: Board::timer1
   ```

   `0x4004c000` is `TIMER1`'s base and `+0x04` is `CFG`.

   Fixing the order exposed the mirror image: **`TOP` faults with the block disabled** (`... at 0x4004c01c`). `CFG`
   wants the timer stopped, `TOP` wants it running. The vendor reference does not say this in one place; the two
   records now do, in the driver's comments.

2. **`clear_overflow()` cleared `TIMER0`'s flag from every instance.** The base address was hard-coded —
   `alias::clear_window(pac::Timer0S::ptr())` — inside the macro that generates all five drivers. A crate with one
   measured timer cannot see this. The instrument's `clean=1` field is what now proves the fix: each instance's flag is
   cleared and read back on that instance.

## The record

Three flashes of the same image, three boots, `status=PASS` in all three, byte-identical:

```
timers: ms=50 o1=48 o2=48 o3=48 o4=0 stopped=0 top=18999 want=1000 clean=1 refused=1
```

Read it as:

| field | meaning |
|---|---|
| `ms=50` | the window, from the `SYSRTC`: 1600 ticks at 32.768 kHz |
| `o1`, `o2`, `o3` | overflows of `TIMER1`, `TIMER2`, `TIMER3` in that window, each built at 1000 Hz — 48, i.e. 960 Hz measured |
| `o4=0` | **control**: `TIMER4` was constructed and never started, and never overflowed |
| `stopped=0` | **control**: `TIMER3` was stopped after the window and did not overflow once in the next 10 ms |
| `top=18999` | the `TOP` the driver computed for 1000 Hz from 19 MHz |
| `clean=1` | **control on defect 2**: each instance's overflow flag was cleared and read back on itself |
| `refused=1` | **control on the claim table**: a second `board.timer1(...)` was refused by name |

48 rather than 50 is not a defect and is not rounded away: the timer counts `HFRCOEM23` while the window comes from the
`LFXO` crystal, the two oscillators are not the same frequency to 1 %, and the flags are polled rather than interrupted,
so the two edges of the window are soft. The tolerance in the instrument is ±3 in 50, which refuses a stopped counter
(0) and a prescaler off by two (about 100).

## What this does not prove

- **Interrupt delivery.** `listen_overflow` and a vector are not exercised here; nothing in this instrument takes an
  interrupt. `hal_dma_irq` is where that was measured.
- **`TIMER0` itself**, which has its own records (`hal_timer_ticks`, `hal_timer0_repacing`, `hal_timer0_pwm`).
- **Input capture**, which remains absent as a driver.

## Why this entry exists

The boundary file's whole purpose is that "not measured" is a decision, not a shrug. Here the shrug was hiding a
constructor that had never been called, and the constructor was wrong twice over. Measuring four instances that looked
identical on paper is what turned that up.
