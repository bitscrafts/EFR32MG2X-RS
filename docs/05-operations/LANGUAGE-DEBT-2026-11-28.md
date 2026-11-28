# Code written in Portuguese: what is left, counted

The project's rule is that code is written in English -- identifiers, comments, commit messages. It is not a
style preference: `deny(missing_docs)` means a module's documentation *is* part of its public surface, and half
of this workspace's readers, reviewers and future models start from the code.

Measured 2026-11-28 by counting lines that contain a Portuguese accented character in `.rs` files:

| File | Lines |
|---|---|
| `crates/efr32mg2x-hal/src/i2c/async_impl.rs` | 68 |
| `crates/efr32mg2x-hal/src/sleep.rs` | 19 |
| `crates/efr32mg2x-hal/src/i2c/README.md` | generated from the above |
| `crates/efr32mg2x-hal/src/sleep/README.md` | generated from the above |

Two files, 87 lines, both of them dense technical reasoning (the async I2C's recovery path and the EM2/EM3
sleep sequence) where the measurements and the exact register names carry the argument. They are worth
translating carefully rather than quickly, and that is why they are listed here instead of being rewritten at
the end of a long session.

## Done in this pass

* `crates/efr32mg2x-hal/src/clock/clocks.rs` -- the whole `start_hfxo` block, including the HFXO clock-gate
  measurement (`Error: Exception` 0.16 s after a read without `CMU_CLKEN0.HFXO0`) and the ceiling rule, plus the
  correction to the LFXO statement.
* `crates/efr32mg2x-hal/src/cores/sysrtc.rs`, `src/sysrtc.rs`, `crates/embassy-efr32/src/alarm.rs` -- five lines,
  translated with their numbers intact (2^32 + 5, 36.4 h).

## Why the count, and not just the rule

A rule that lives only in a document gets broken, which this workspace has paid for more than once. A count with
a date is falsifiable: the next reader can re-run the one command and see whether the debt shrank.

    grep -rc 'ã\|õ\|ç\|ê\|á\|í\|ó\|ú\|à' crates/*/src/*.rs crates/*/src/*/*.rs | grep -v ':0$'
