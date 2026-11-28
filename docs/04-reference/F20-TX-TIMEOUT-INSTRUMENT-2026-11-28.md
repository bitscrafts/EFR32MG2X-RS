# f20 — the transmit timeout, and four instruments that measured nothing

**Date:** 2026-11-28
**Status:** the fix is committed and green (`9621f31`); **the error path has never been entered.** No record from the
board shows `Error::Timeout` returning.
**Why this file exists:** so the next attempt starts from what was already paid for, instead of re-trying the four
designs below and losing another evening to the same three negatives.

---

## 1. What the defect was

`write_byte` on USART0 and on both EUSARTs, `write` on both, `flush` and `flush_receiver` spun on a status bit with no
limit:

| site | wait |
|---|---|
| `crates/efr32mg2x-hal/src/eusart/mod.rs:207` | `while !status.txfl()` — the transmit FIFO never has room |
| `crates/efr32mg2x-hal/src/usart/mod.rs:185` | `while !status.txbl()` — same, USART0 |
| `crates/efr32mg2x-hal/src/usart/mod.rs:334` | `while !status.txc()` — the last bit never leaves |
| `crates/efr32mg2x-hal/src/usart/mod.rs:289` | `flush_receiver` — same shape, receive side |

A clock nobody enabled, a pad nobody routed, or an unset enable bit left the program there for ever, and the caller
could not tell: `write_byte` returned `()`.

## 2. What the fix is (committed, `9621f31`)

`Error::Timeout`; `Result<(), Error>` from all four; a bound counted in `spin_loop` iterations (`TX_SPINS = 2_000_000`,
≈50–100 ms at 19–39 MHz against the 87 µs one byte takes at 115 200 — margin ≈600). Counted, not timed: a time limit
would need the SysTick, and `delay` now owns the only tick in the program. Sixteen example call sites carry the result
instead of dropping it.

**What the fix is worth without the measurement below:** the hang is now an error the type system can express. That is
not the same claim as "the error was seen", and the commit message says so.

## 3. The four instruments

Common setup unless stated:

```sh
cd /projects/efr32mg24-rs
export TMPDIR=/projects/_tmp/tmp CARGO_TARGET_DIR=/projects/_tmp/tmp/target-cold
cargo build --release --target thumbv8m.main-none-eabihf -p efr32mg2x-hal --features rt,test-hooks \
      --example hal_tx_timeout
FLASH_HOST=lnx FLASH_USER=mvcorrea bash scripts/flash.sh \
      $CARGO_TARGET_DIR/thumbv8m.main-none-eabihf/release/examples/hal_tx_timeout --confirm --run
FLASH_HOST=lnx FLASH_USER=mvcorrea bash scripts/read-mailbox.sh lnx mvcorrea
```

### 3.1 Loopback, receiver never drained — **measured, and it does not stall**

Idea: `CFG0.LOOPBK` closes the block on itself; with nobody reading, the receiver FIFO fills, the transmitter has
nowhere to put the next frame, `TXFL` stops clearing, write times out. No clock or enable bit needs touching.

Record (a real one, the only one obtained):

```
tx-timeout: control=240 written=240 refused_at=240 of=240      FAIL
```

240 bytes went through with no refusal, drained or not. **This part's loopback does not back-pressure — the receiver
overruns** (which is exactly what `hal_eusart_loopback` measures from the other side). Dead end, and cheap: one boot.

### 3.2 Block handed out before its clock is enabled — **no record**

`Board::eusart0_unclocked_for_test`: claim the slot, skip `CLKEN1.EUSART0`, build the driver.

```
no complete record after 12 attempts; last magic seen: 00000000
```

The constructor never returned: `Eusart0::new_at` waits on the block it is configuring, and a block with no clock never
reports. The hook reached the driver but not the write path.

### 3.3 Clock cleared **after** configuring — **no record**

Same idea, order inverted: build with the clock on, clear `CLKEN1.EUSART0` afterwards. This removes 3.2's objection.

```
no complete record after 12 attempts; last magic seen: 00000000
```

The constructor now returns. Hypothesis: a write to `TXDATA` on an unclocked block is a barred access that faults the
core, and nothing writes the mailbox from a fault handler (`panic_halt` leaves no trace).

### 3.4 `CMD.TXEN` never asserted — **no record**

The defect as it actually happens: clock on, block configured, receiver enabled, only the transmitter's command bit
left out, so the FIFO fills and `TXFL` stops clearing. 32 bytes written.

```
no complete record after 12 attempts; last magic seen: 00000000
```

## 4. What is established, and what is not

**Established by measurement**

* the loopback does not back-pressure the transmitter on this part;
* an unclocked block at construction time hangs the constructor;
* an unclocked block after construction still produces no record.

**Not established**

* that `Error::Timeout` is ever returned. No record shows it.
* why 3.4 produced nothing. Its design has no fault mechanism (the clock is on), so either it hung somewhere before the
  mailbox write, or the image did not run as intended, or the read raced a reset.

## 5. The cheapest discriminating test for the next session

The failure is invisible because the mailbox says nothing until the end. Write it in stages, before anything can hang:

```
magic + stage 1   after Board::take()
magic + stage 2   after eusart0_*() returns a driver
magic + stage 3   before the first write_byte
magic + stage 4   after the loop (with written / refused_at / timeout)
```

Whichever stage is the last one present is where the program stops. That costs one boot and turns "no record" into a
line number, which is the only thing four blind iterations could not buy.

If stage 3 is the last: the hang is inside `write_byte`, i.e. `TXFL` never sets and **`TX_SPINS` is not being reached** —
look at whether the loop can be optimised or whether `spin_loop` is elided, and whether `TXFL` of a disabled transmitter
reads as *set* (FIFO empty) rather than clear. If stage 4 is present but the record is a FAIL, the numbers will say
which half refused.

## 6. Constraints that still hold

* The `test-hooks` feature is off by default and must never be enabled in a product build; the hook's doc comment says
  why. `hal_tx_timeout` declares `required-features = ["rt", "test-hooks"]`, so a plain examples build skips it and the
  gate's group builder (`scripts/example-groups.py`) compiles it with those features.
* Nothing in this file justifies removing or weakening a gate check. If a check disagrees with a measurement, the file
  to change is this one, and the disagreement gets written down.
* Work in the tree at the time of writing: `HANDOFF.md`, `crates/efr32mg2x-hal/{Cargo.toml,README.md}`,
  `examples/hal_tx_timeout.rs`, `src/board.rs`, three module READMEs, and `scripts/run-hal-tx-timeout-sonnet.sh`
  (untracked). HEAD `9fdbc24`.

## 7. The dispatch, and why it came back empty-handed

`scripts/run-hal-tx-timeout-sonnet.sh` dispatched `claude-sonnet-5-5` with `_tmp/f20/brief.md`. It returned a plan and no
edits: the session ran read-only (`IS_SANDBOX=1` without `--permission-mode acceptEdits`), which is a fault in the
script, not in the worker. **Fix before re-dispatching:** add the permission mode, and read the session id from the log
(`grep -oE '"session_id":"[^"]+"'`) because the resume handle is recorded only when the run writes one.

---

# Addendum, same evening: the timeout was finally seen once, and why one boot cannot show both halves

After four blind negatives the mailbox was instrumented in stages ("stage N: ..." written before and after each step).
That turned `magic 0` into a line in one boot, and the line moved the work forward by three facts.

## What the stages found

| stage | meaning | appeared? |
|---|---|---|
| 1 | `Board::take()` returned | yes |
| 2 | about to call the constructor | yes |
| 3 | driver built | **no, on the hook that cleared `CLKEN1`** |
| 3 | driver built, clearing `CMD.TXEN` | yes |
| 4 | about to write | yes |
| 5 | the wait returned | yes, until the flush design replaced it |

So the earlier "constructor hangs" was not the constructor: it was the **write to `CMD` from `board.rs`** that never
returned. Moving the same line into a driver method (`disable_transmitter_for_test`) made it pass -- the difference
being that the driver's copy is the block's own handle, while the board's was a second `steal()`.

## RETRACTION, same evening: the three "facts" below were measured through a broken hook

An outside review (`claude-opus-5-5`, `OPUS-REVIEW-F20-2026-11-28.md`) read the PAC and found the defect that makes
this section worthless as written:

* **`CMD` acts only on bits written as one.** `txen().clear_bit()` writes a **zero** into that register and therefore
  does nothing at all. The bit that disables a transmitter is `CMD.TXDIS` (bit 3, `usart0_s/cmd.rs:43`). Every
  measurement taken "with the transmitter disabled" was taken with the transmitter **running**.
* **`STATUS.TXC` is clear after reset** (`STATUS` reads `0x2040`), so a `flush` that looked only at it reported
  `Timeout` on a port that had never transmitted. That, and not the bound, is what `flush_timeout=1` measured -- and
  what `control_flush=0` measured, from the other side. The defect is fixed in `c344432` (`tx_pending`).
* None of the records carried `STATUS.TXENS`, the bit that would have shown the transmitter was still enabled. A
  record that cannot distinguish "disabled" from "enabled" cannot settle this question at all.

So: facts 1 and 3 below are **not established** -- they describe a transmitter that was never disabled. Fact 2's
observation (a `Timeout` appeared) is real, but its cause was the `flush` bug, not a stalled transmitter. What remains
true is only what does not depend on the hook: the loopback experiment (the receiver overruns, it does not
back-pressure) and the fact that a `CMD` write from a second `steal()` in `board.rs` never returned.

The corrected design, and the record that would settle it, is at the end of `OPUS-REVIEW-F20-2026-11-28.md`: one boot
with `flush->Ok`, `TXDIS`, `TXENS` recorded as zero, writes until a refusal, `flush->Timeout`, `TXEN`, `flush->Ok`.

## Three facts the board settled

1. **`write_byte` cannot be made to block.** With `CMD.TXEN` clear the block accepts every byte and discards it: `TXFL`
   ("the FIFO has room") stays set. Measured at 32 bytes and at 1024 bytes: `refused_at=32`, `refused_at=1024`, no
   `Timeout`. The bound on the FIFO wait is therefore a safety net that a healthy block never exercises.
2. **`flush` is the wait that describes the defect.** It asks for `STATUS.TXC`, the transmitter's own "I have
   finished", and a transmitter that was never enabled never says it. With `CMD.TXEN` cleared *before any
   transmission*: `flush_timeout=1` -- **`Error::Timeout` returned by the crate, seen in a record**, once.
3. **`TXC` is latched, and a write to a disabled transmitter does not clear it.** Trying to put both halves in one boot
   failed for that reason: the control's successful transmission left `TXC` set, and the disabled transmitter's `flush`
   then returned `Ok` too (`control_flush=1 flush_timeout=0`). A `flush` on a block that never transmitted also returns
   `Ok` only by accident of the flag being clear -- the first control attempt read `control_flush=0` because `TXC` was
   *not* set, i.e. the control was wrong, not the block.

## What that means for the instrument

The two halves cannot live in one boot, because the state they need is mutually exclusive and the hardware gives no way
to clear `TXC` from a disabled transmitter. The shape that works, and that this project already uses elsewhere
(`hal_spi_masters`), is **two boots, two records, compared**:

* boot A -- transmitter enabled: write eight bytes, `flush()` -> `Ok`, record `control: write=ok flush=ok`;
* boot B -- `CMD.TXEN` cleared before the first write: write eight bytes, `flush()` -> `Err(Timeout)`, record
  `measure: write=ok flush=timeout`.

Both records then sit in the example's header, and the claim is "the bound returns `Timeout` on a transmitter that was
never enabled, and does not on one that was", which is exactly what the fix promised.

## State at the end of this session

* `9621f31` -- the fix, green.
* `e40d319` -- the documentation of the four negatives, green.
* the staged-mailbox instrument and the two hooks (`disable_transmitter_for_test` on `Usart0` and on the EUSARTs) are in
  the tree, **uncommitted**, with the seventh attempt's record still FAIL because it tries to hold both halves at once.
