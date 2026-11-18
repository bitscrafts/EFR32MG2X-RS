# Spec 09: one core moves into the HAL and the pattern proves itself (H-pub 1/6)

## Overview

`efr32mg2x-hal` cannot be published: six of its dependencies are `libraries/` crates declared by
`path` only, and cargo refuses a package whose dependency has no version. The `libraries/` crates
must not go to crates.io, and they exist because they hold logic that runs on the host: 109 tests
across 15 crates, none of which touches the hardware stack.

The fix is not to copy and not to publish: the source of a core moves into the HAL, and the
library crate keeps compiling **that same file** through `#[path]`. One file, one set of tests,
two places that compile it -- host for the tests, target for the firmware. This item moves one
core (`spi`) to prove the pattern before the other five follow, one commit each, `sysrtc` last
because `efr32mg24-bluetooth-sys` also calls it.

## Requirements

**R1 -- the source lives in the HAL, and the library compiles it.**
`libraries/spi-core/src/lib.rs` contains no domain code: a `#[path]` attribute pointing at
`crates/efr32mg2x-hal/src/core/spi.rs`, one `mod`, one `pub use`.
Example: `cargo test -p efr32mg2x-spi-core --target x86_64-unknown-linux-gnu` runs the same six
tests it runs today, and they pass.

**R2 -- nothing is duplicated, and the repository can prove it.**
`scripts/check-core-split.py` compares the file the library names in its `#[path]` against the
file the HAL contains: same bytes, and the library file holds at most the shim.
Example: `python3 scripts/check-core-split.py` prints `OK: 1 core, one copy each` on success and
`FAIL: libraries/spi-core/src/lib.rs holds code of its own (N lines)` when a copy is introduced.

**R3 -- a library that reaches the hardware stack fails the gate.**
Already true for the other crates, and this item must not weaken it: the moved file is compiled
by a crate that builds for the host, so a `pac::` or `cortex_m` reference in it stops that crate
from compiling.
Example: adding `use efr32mg2x_pac::spi0_ns as _;` to `crates/efr32mg2x-hal/src/core/spi.rs` makes
`cargo test -p efr32mg2x-spi-core` fail before the gate is even asked.

## Files

- `libraries/spi-core/src/lib.rs`
- `crates/efr32mg2x-hal/src/core/mod.rs`
- `crates/efr32mg2x-hal/src/core/spi.rs`
- `crates/efr32mg2x-hal/src/spi.rs`
- `crates/efr32mg2x-hal/Cargo.toml`
- `scripts/check-core-split.py`
- `scripts/gate.sh`

## TDD Contract

| Test Name | Input | Expected Output |
|---|---|---|
| the_shim_holds_no_code_of_its_own | a library whose `lib.rs` has the `#[path]` shim | `OK`, and the shim is under 10 lines |
| a_copy_is_caught | the same `lib.rs` with the function bodies pasted back | `FAIL`, naming the file and the line count |
| a_missing_target_is_caught | `#[path]` pointing at a file that does not exist | `FAIL`, naming the path |
| the_moved_file_is_the_same_bytes | the HAL file and the file the shim names | identical sha256 |

## Exit Criteria

- [ ] `python3 scripts/check-core-split.py`
- [ ] `cargo test -q -p efr32mg2x-spi-core --target x86_64-unknown-linux-gnu`
- [ ] `cargo check -q -p efr32mg2x-hal --target x86_64-unknown-linux-gnu --no-default-features --features mg24`
- [ ] `bash scripts/gate.sh`

## Guardrails

- Do NOT copy the file. If two copies exist, the item is wrong even when the tests pass.
- Do NOT publish anything. Publication needs an express order.
- Do NOT touch the other five cores in this item: one commit, one core.
- Do NOT remove the library crate: it is the only place the host tests run from.
