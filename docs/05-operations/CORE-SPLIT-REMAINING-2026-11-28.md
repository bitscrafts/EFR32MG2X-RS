# The core split: what the check was saying, and what is actually left

Read 2026-11-28, during the SDK-parity session.

## The check was wrong about two of five

`scripts/check-core-split.py` prints the ones that still hold "code of its own". It said five. It said
`libraries/i2c-core` was one, on the strength of this:

    pub mod sim;

and `libraries/aes-core`, on the strength of this:

    pub use spi::{
        answered, config_tag, data_tag, ...
    };

Both are declarations -- the material a shim is made of -- and the counter judged them line by line, so a
`pub mod` (not on its exclusion list) and the continuation lines of a wrapped `pub use` counted as code.

Corrected by counting *items* rather than lines: a `pub use` is tracked to its terminating semicolon, and
`pub mod` / `pub(crate) use` / `use` join the exclusions. The self-test gained three cases, and the third is
the important one: a shim whose declarations are followed by a real line of code must still FAIL. Loosening
a check is only safe when it keeps failing on the thing it exists to catch.

After the correction: **five became three**.

## What is actually left

| Crate | Lines of its own | What it includes from the HAL | State |
|---|---|---|---|
| `libraries/gpcrc-core` | 67 | `crates/efr32mg2x-hal/src/cores/gpcrc.rs` | **still to do** |
| `libraries/record-core` | 0 | `crates/efr32mg2x-hal/src/cores/record.rs` | closed: its 72 lines were its tests |
| `libraries/timer-core` | 0 | `crates/efr32mg2x-hal/src/cores/timer.rs` | closed 2026-11-28 |

Each already compiles the HAL's copy *and* keeps logic of its own beside it. That is the shape C1 exists to
remove, and the question to answer crate by crate is concrete:

* Is the remaining logic something the HAL **should** use (so it belongs in `crates/.../cores/`, with the
  library keeping only the shim)? or
* Is it logic the library kept from before the move, exercised by host tests that therefore prove something
  nobody ships?

## `timer-core`, answered and closed the same night

**Done 2026-11-28:** the planning logic moved into `crates/efr32mg2x-hal/src/cores/timer.rs`, `timer-core` is now a
shim plus its host tests, and `timer/mod.rs` calls `cores::timer::period_for`. The handle carries the truth it was
hiding: `get_actual_frequency()` and `get_frequency_error_ppm()`, beside `get_frequency()` which is documented as the
*request*. A request the fields cannot express uses `nearest_bound` and reports `None`, so "inexpressible" and
"inexact" are different answers.

Measured on the board, two boots, byte-identical (`len=115`):

    timers: ms=50 o1=48 o2=48 o3=48 o4=0 stopped=0 top=18999 want=1000 actual=1000 ppm=0 measured=960 clean=1 refused=1

`actual=1000 ppm=0` is arithmetic on `(prescaler 1, TOP 18999, 19 MHz)` and can be checked by hand. `measured=960` is
the polled witness reading 4% low -- the loop clears four flags and reads the SYSRTC between polls, which costs about
the 1 ms period, so an overflow is occasionally set and cleared unobserved. Two attempts to remove that bias are
recorded in the example: a ten-times-longer window turned the per-poll loss into 2.6% and broke the `SLACK` check
written for the short window; reading the counter at both ends failed outright, because the counter wraps once per
period (`counts=235`). The bias is the instrument's, it is now written down, and the cross-check that matters -- did
the configuration take effect -- passes.

The original finding, kept for the record:

The 153 lines are named now:

| `libraries/timer-core/src/lib.rs` | `crates/efr32mg2x-hal/src/cores/timer.rs` (included by it) |
|---|---|
| `PRESCALERS`, `MAX_TOP`, `TimerError`, `Period`, `period_for`, `may_configure` | `COUNTER_BITS`, `WRAP_TICKS`, `elapsed`, `elapsed_with_wraps`, `CaptureError`, `CaptureSlot` |

So the library holds the *period planning* and the state-machine refusal, and includes the HAL's
*counter arithmetic*. That is the intended shape -- except for one thing:

**`crates/efr32mg2x-hal/src/timer/mod.rs:182` has its own `calculate_prescaler_and_top`, and it is the one
that ships.** The two are not the same function:

* `period_for` (`timer-core:87`) returns `Result<(Period, u32), TimerError>`: it refuses a zero or
  impossible request, and it reports how far the chosen rate is from the wanted one -- the H12 shape.
* `calculate_prescaler_and_top` (`timer/mod.rs:182`) returns a bare `(u16, u32)`, walks the prescalers with
  `if timer_clk > target_freq` and `saturating_sub(1)`, and **reports nothing**: a frequency that cannot be
  expressed is silently approximated.

The host tests therefore exercise the version that is not shipped, and the version that is shipped is the
one the 2026-11-17 review asked to fix. This is exactly what the C1 item exists to prevent, found by
following the check instead of overriding it.

The fix is mechanical once stated: move `period_for` (and `Period`, `TimerError`, `PRESCALERS`, `MAX_TOP`,
`may_configure`) into `crates/efr32mg2x-hal/src/cores/timer.rs`, leave `timer-core` as a shim, and have
`timer/mod.rs` call it -- then measure the resulting frequency on the board, because a change to this path
without a measurement would be the same mistake in the other direction.

The same question is open, un-answered, for `gpcrc-core` (94 lines) and `record-core` (72).

## Not in the gate

`check-core-split.py` is not called by `scripts/gate.sh`, so a green gate says nothing about this. That is
a decision to take, not an oversight to assume: a check that is not run is a check nobody has.
