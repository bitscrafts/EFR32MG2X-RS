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

| Crate | Lines of its own | What it includes from the HAL |
|---|---|---|
| `libraries/gpcrc-core` | 94 | `crates/efr32mg2x-hal/src/cores/gpcrc.rs` |
| `libraries/record-core` | 72 | `crates/efr32mg2x-hal/src/cores/record.rs` |
| `libraries/timer-core` | 153 | `crates/efr32mg2x-hal/src/cores/timer.rs` |

Each already compiles the HAL's copy *and* keeps logic of its own beside it. That is the shape C1 exists to
remove, and the question to answer crate by crate is concrete:

* Is the remaining logic something the HAL **should** use (so it belongs in `crates/.../cores/`, with the
  library keeping only the shim)? or
* Is it logic the library kept from before the move, exercised by host tests that therefore prove something
  nobody ships?

Measured while looking: `calculate_prescaler_and_top` exists **only** in
`crates/efr32mg2x-hal/src/timer/mod.rs:182` -- it is *not* in `cores/timer.rs` and not in the library -- so
the prescaler algorithm is not duplicated. The 153 lines are therefore something else, and naming them is
the first step of the job, not a guess to be written here.

## Not in the gate

`check-core-split.py` is not called by `scripts/gate.sh`, so a green gate says nothing about this. That is
a decision to take, not an oversight to assume: a check that is not run is a check nobody has.
