# Moving a core into the HAL: the procedure, and the traps that cost a session

**Why this exists.** `C1` (the SPI logic) took a full evening and four red gates, none of which were
about SPI. Everything below was paid for once; the next five cores are the same move, and this is
the page that stops the payment repeating. Written the night `C1` closed, commit `efedb81` +
`a06295b`.

## The shape

The HAL cannot be published while it depends on `libraries/` crates by path: cargo refuses a package
whose dependency has no version, and the `libraries/` crates must not go to crates.io — they exist
because they hold logic that runs on the host (109 tests across 15 crates, none touching hardware).

So the source moves into the crate that gets published, and the library keeps compiling **that same
file** through `#[path]`. One copy, two compilers: the host runs the tests, the target builds the
firmware. The library crate is the harness, not a duplicate — `scripts/check-core-split.py` fails if
any line of code appears beside the shim.

## The six steps

```
1  read the first line of the file you are moving       head -1 libraries/<x>-core/src/lib.rs
2  copy it to the HAL                                    crates/efr32mg2x-hal/src/cores/<x>.rs
3  strip crate-root attributes from the copy            #![no_std], #![deny(...)], extern crate
4  rewrite the library as the shim                      #[path], mod, pub use -- nothing else
5  redirect the callers                                 efr32mg2x_<x>_core:: -> crate::cores::<x>::
6  drop the path dependency                             Cargo.toml, one line, 6 -> 5 -> ... -> 0
```

The library shim keeps `#![no_std]` at the top: it is a crate root, and without it the thumb build
looks for `std`.

## The traps, each one measured

**`core` is a reserved name.** The module is `cores`, not `core`: `core` is the name of the Rust
crate, and a module with that name shadows it inside the crate, breaking `core::fmt` in `no_std`
code.

**Crate-root attributes are illegal inside a module.** Moving the file wholesale carried
`#![no_std]` into the HAL and the whole workspace stopped compiling with "the `#![no_std]` attribute
can only be used at the crate root". Step 1 of the procedure exists because of this: one `head -1`
would have saved two gate cycles.

**A red crate is a cascade, not a count.** The HAL not compiling turned two BSPs red as well ("1
warning" each). Read the gate from the bottom: one cause, three symptoms.

**The check must fail on the correct case too.** The first version of `check-core-split.py` excluded
`//`, `#[path`, `mod`, `pub use` when counting a shim's code lines — and so counted the shim's own
`#![no_std]` as code, failing a correct shim. A worker found it. A check that fails correct work is
as bad as one that passes broken work.

## Never two owners of one item

The evening's most expensive accident: a scheduler and a person worked `C1` at the same time. The
worker spent 24 minutes, the commit was refused because the tree had moved, and the item had been
marked done by hand in the meantime. Nothing was lost, but 24 minutes of paid work were.

The rule: **before starting an item, take a lock and write which item is in flight.** The runner
does `flock` on `_tmp/queue/lock`; a person starting by hand writes `_tmp/queue/current` with the id
and their name. If the file exists and is younger than the item budget, do not start — read the log
to see who has it.

## The proof that closes an item

```
python3 scripts/check-core-split.py                      -> OK: N core(s) moved, one copy each
cargo test -p efr32mg2x-<x>-core --target x86_64-...     -> the library's tests, on the host, through the shim
cargo clippy -p efr32mg2x-hal -- -D warnings             -> 0 warnings
bash scripts/gate.sh                                     -> VERDICT: PASS
```

The HAL's own tests do not run on the host: there is inline asm (`dsb`/`isb`) the x86 assembler
refuses. `cargo check` on the host is the most that can be asked of it.

Commit only the paths the spec's `Files` block names. `git add -A` has already committed one item's
half-work inside another item's commit.
