# The delay bug that three clocks found

`emlib_clock` was written to be a check, and the first thing it caught was ours.

## The measurement

```
emlib: sys=19000k core=19000k meas=8268k     status = FAIL
```

- `SystemSYSCLKGet()` → 19 000 kHz
- `CMU_ClockFreqGet(cmuClock_CORE)` → 19 000 kHz
- an independent count: DWT cycles across `delay.delay_ms(10)`, divided by 10

The first two agree with each other and with everything measured on this board since the clocks
chapter. The third says the "10 ms" delay produced 82 680 core cycles. At 19 MHz that is **4.35 ms**,
not 10.

So `Delay` is fast by a factor of 2.3, and every example that uses `delay_ms` — every blink, every
sensor settle, every timeout — has been running on a delay that is 2.3 times shorter than it claims.

## Why the check mattered

The delay was **verified before**, by watching an LED blink. A blink at "500 ms" that is really 217 ms
still blinks. A person counting blinks does not notice a factor of 2.3, and the project had no reason
to look. The example that failed is the one that compared the delay against a clock that does not come
from the delay.

This is the general lesson, and it is HARD RULE 0b in the workspace: a check whose input comes from
the thing being checked will always pass. `delay_ms(10)` measured by `delay_ms` is a tautology; measured
by DWT it is a test.

## The arithmetic (the bug that was suspected first, and was not it)

`delay_us` converts with `(us as u64 * sysclk_hz as u64) / 1_000_000`, so there is no 32-bit overflow
there — that was my first hypothesis and the code disproved it. `DEFAULT_SYSCLK_HZ` is 19 000 000,
which matches what emlib reads back from the CMU. So the constant is right and the conversion is right;
the assumption that is wrong is **what rate SysTick counts at**.

SysTick is configured with `SystClkSource::Core`. The measurement implies it ticks at about 44 MHz while
the core runs at 19 MHz. That is not a rounding error, and it is not something to paper over by
changing `DEFAULT_SYSCLK_HZ` to 44 000 000: the number that matters is the ratio between the two
clocks, and it must be **measured on the part**, not tuned until the example goes green.

## What the fix has to be

Not a constant. The honest fix is to stop assuming and start measuring:

1. At `Board::delay()`, run SysTick for a known number of ticks and count DWT cycles across it. The
   ratio is the true SysTick frequency, whatever the clock tree was configured to.
2. Keep that measured value in `Delay`. A delay is then accurate without anyone knowing which clock
   feeds SysTick.
3. Re-run `emlib_clock` and require the third number to agree with the first two within a few percent.

That is the next commit. It is not in this one because a half-applied timing change across a workspace
whose examples already under-report their own duration is worse than a documented defect.

## Where this is referenced

- `crates/efr32mg24-hal/src/delay/mod.rs` — `with_sysclk`, `delay_us`, `DelayNs`
- `crates/efr32mg24-emlib-sys/examples/emlib_clock.rs` — the check that found it
- Book chapter 5 (`05.04-delay-without-lying.tex`) — written from this, now with a second case

## Addendum: the first fix attempt failed, and was reverted

An attempt was made to measure SysTick's rate against the DWT cycle counter at `Board::delay()` and
derive the conversion from it. Measured result of that change: the delay went from 2.3x short to
**3.0x short** — worse, and still wrong.

Cause of the failed attempt: the calibration used the same `set_reload` / `clear_current` /
`has_wrapped` sequence the delay itself uses. If that sequence is what is broken — writing `VAL = 0` on
a down-counter arms a wrap within a cycle or two rather than arming a full period — then the
calibration inherits the defect it is trying to measure, and its answer is garbage that looks precise.

The change was reverted, not shipped. A wrong mechanism dressed as a fix is worse than a documented
defect, because the next person trusts the number.

## The experiment that settles it

Do this before touching the conversion again, in one example, one flash:

1. `syst.set_reload(100_000); syst.clear_current(); syst.enable_counter();`
2. Count DWT cycles until `has_wrapped()`. Report the count.
3. Repeat with `syst.set_reload(10_000)`.

Two tick counts, one cycle count each, and the answer appears:

- If cycles(100 000) / cycles(10 000) ≈ 10, the counter is arming a real period and the rate is
  `ticks * 19e6 / cycles`. Then the conversion is where the error is.
- If both are a handful of cycles, `clear_current()` is arming an immediate wrap and the delay has been
  returning after one or two ticks all along. Then the fix is to *not* clear the counter — write the
  reload and let the count run down from it, or count elapsed cycles instead of waiting for a wrap.

Until one of those is measured, the honest statement about this board's delays is: they are shorter
than requested, by a factor that has been seen between 2.3 and 3.0, and nothing in this project should
assume a `delay_ms` is what it says.

## Addendum 2026-11-13: a fourth reading, from an example rather than an instrument

`ble_advertise_custom` publishes its uptime as a raw DWT cycle count (`cy=`), because no divisor for this
board's core clock has been proven. Read off the board after a flash that ends in `reset run`:

```
adv frame=9B cy=404172741 n=22
```

That is ~404 million cycles across the ~10 s the mailbox was left to settle, so **~40 MHz**, not the 19 MHz the
device configures and not the ~78 MHz a debugger-attached reading once suggested.

How much to trust it: the elapsed time is the harness's `sleep`, not a measurement on the board, so the divisor is
good to perhaps ±20% -- enough to rule out 19 MHz (which would need 190 million cycles in that window) and enough to
land near the 44 MHz that SysTick implies, but not enough to name a divider. The exact experiment remains the one
described above: measure a known number of SysTick ticks against the DWT counter in the same run, on the board, with
no debugger attached.
