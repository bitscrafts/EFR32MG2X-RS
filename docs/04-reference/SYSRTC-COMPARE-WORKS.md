# The compare channel works, and three registers were wrong

Measured 2026-11-13, on the board, with a probe that publishes read-backs rather than conclusions. This file
exists because the SYSRTC story collected four separate lies from the headers, and each one cost a session.

## What the probe established

```
clr=0000  if=0022  d=1   cnt=302      compare channel 0, target = start + 300
a=0022  b=0044  d=1  e=1             channel 0 then channel 1, same test twice
cmp: start=0 deadline=3200 reached=3201 late=1 fired=1 en=1 run=1     the HAL, after the fix
```

Read literally:

- **The compare channel fires**, exactly one count after the programmed value, and `late = 1` in the HAL's own
  test. Everything the stack needs from a timer exists on this silicon.
- **`GRP0_IF`/`GRP0_IEN` bits are shifted by one against every header.** With only `CMP0EN` set, the flag appears
  in **bit 1**, never bit 0. Channel 1 appears in **bit 2**. Bits 4..7 mirror the channel flags at group level
  (`0x22` and `0x44` carry both halves). So the per-channel flag is bit `1 + n`, and the header's `CMP0 = bit 0`
  is wrong the same way its group base offset (`0x40`) was wrong.
- **`GRP0_IF` is write-1-to-clear through the `+0x2000` alias.** A plain store to `0x60` does not clear it; the
  probe cleared it and read `0000` back only after writing `0x2060`. A deadline that "looks already fired" is
  what a store-that-does-nothing looks like from the outside.
- **`CFG` cannot be written while the module runs.** SYSRTC lives in the always-on domain and keeps counting
  across a reset, so a warm restart finds `EN=1`. Writing `CFG` then faults at exactly `0x500a800c`, with
  `CFSR = 0x00008200` (PRECISERR + BFARVALID) and BFAR naming the register. Reproduced twice. The fix is the
  vendor's own order: stop, `EN_CLR` (write-1-to-clear at `0x2004`), wait for `EN` to read zero, then write `CFG`.

## The four defects, in the order they were found

| # | Believes | Reality | Symptom it caused |
|---|---|---|---|
| 1 | `CMU_SYSRTC0CLKCTRL` at `0x1DC` | `0x240` | clock select never took; `EN` never set; `CMD` faulted |
| 2 | `EN_SET` at `0x1000` | `0x1004` (`0x1000` is `IPVERSION_SET`, read-only) | writes vanished; `EN` stayed 0 |
| 3 | group block at `0x40` | `0x60` (`IF`), `0x64` (`IEN`), `0x68` (`CTRL`), `0x6C` (`CMP0`), `0x70` (`CMP1`) | the compare channel was never actually enabled |
| 4 | `CMP0` flag = bit 0 | bit 1 (and `CMP1` = bit 2) | polling the wrong bit read "never fires" while the flag was set and then cleared by the next event |

## How to measure this again, without a debugger

`crates/efr32mg24-pac/examples/pac_sysrtc_compare_flag.rs` publishes, in one short line: the flags after a clear,
the flags when they change, and the distance between the counter at that moment and the programmed value. A
distance of 1 means the comparison is exact; a distance of thousands means the flag was already set; no change
means the channel is not enabled.

Two things that make it work and are easy to get wrong:

- **Publish short.** An earlier version's message was cut at 64 bytes and the field that mattered — the counter
  at the end — was the part lost. A truncated record looks like a missing measurement.
- **Publish read-backs, not conclusions.** `ctrl=00000001` says the control write stuck; "the channel is enabled"
  is what I believed for two sessions while it was not.
