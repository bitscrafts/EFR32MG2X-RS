# Does `stop()` disable the DMA channel? Closed, with the instrument that could answer

**Status: closed, 2026-11-23.** `Channel::stop()` disables the channel, and a blocking `transfer()` no longer leaves it
armed. The instrument is `crates/efr32mg2x-hal/examples/hal_dma_stop.rs`; the record, read back off the part:

```
dma-stop: st0=0 armed=1 stA=1 busyA=0 stS=0 ctl=1 stRe=0 stS2=0      status=PASS
```

`st0` the channel was not enabled before arming · `armed` the peripheral-to-memory channel took the descriptor ·
`stA` it was **enabled** while the guard held the buffer · `busyA` it was **not busy** (see below) · `stS` it was
disabled after the guard was dropped, which is `stop()` · `ctl` a blocking transfer copied the right bytes ·
`stRe` it left the channel disabled · `stS2` a second `stop()` was idempotent.

## Why the first instrument could not answer, and it was not a race

The earlier version read **`CHBUSY`** and got `busy_armed=0`, every time. That is not the defect showing and not a
timing artefact: `CHBUSY` (RM 25.7.13) reads 1 while the channel is *doing a transfer*, and a channel can be armed,
enabled and waiting without transferring anything. Measured in the same boot, on the same firmware that passes:
`busyA=0` with `stA=1`.

The register that answers the question is **`CHSTATUS`** (RM 25.7.12, offset 0x02C): one bit per channel, "true if the
channel is enabled". It reads 1 from the moment `CHEN` is set until `CHDIS` takes effect, and it can be read at any
time, including while a `Transfer` guard borrows the channel.

## The defect, and the register it should have used

`CHEN` (offset 0x024) is write-1-to-set and the manual says writing zeros to it has no effect; the disable register is
`CHDIS` (offset 0x028), also one bit per channel (RM 25.7.10-25.7.11). `stop()` wrote `chen().bits(0)`, which disabled
nothing, so every path that dropped a `Transfer` handed back a buffer the DMA still owned -- from safe code, with the
host tests green because their fake channel records the call, not the register.

## A second defect the instrument found: a blocking transfer left the channel armed

With `stop()` correct, the blocking `transfer()` still returned with `CHSTATUS` set for its channel: the transfer was
complete, the borrow was over, and the channel was still enabled with a spent descriptor. A later request -- a stray
`SWREQ`, a peripheral event routed to that channel -- could run it again over memory the caller had taken back.
`transfer()` now calls `stop()` before returning `Ok`, and the record shows `stRe=0`.

## The control, in both directions

| firmware | record | verdict |
|---|---|---|
| `stop()` with `CHDIS` | `st0=0 armed=1 stA=1 busyA=0 stS=0 ctl=1 stRe=0 stS2=0` | PASS |
| **control**: `stop()` back to `chen().bits(0)` | `st0=0 armed=1 stA=1 busyA=0 stS=1 ctl=1 stRe=1 stS2=1` | **FAIL** |
| control reverted | as the first row | PASS |

Same instrument, same board, one binary apart. The control flips exactly the three readings that depend on `stop()`,
and `stA=1` sits in both firmwares as the proof that the bit can read 1 in that boot -- without it, `stS=0` would only
show that the bit can read zero.

## What is still open

- Nothing about `stop()` itself. The device-side question this example sidesteps is the *peripheral* path: a channel
  fed by a real request source (`PeripheralRequest::IadcScan`) exercised end to end, which is the microphone
  driver's territory and is measured there, not here.

## Files

- `crates/efr32mg2x-hal/src/dma/mod.rs` -- `stop()` through `CHDIS`; `is_enabled` and the free `channel_enabled`
  (reading `CHSTATUS`); `transfer()` disables the channel before it returns.
- `crates/efr32mg2x-hal/examples/hal_dma_stop.rs` -- the instrument, with the control built into its own record.