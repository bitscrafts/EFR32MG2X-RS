# The compare channel works, and the interrupt belongs to RAIL

Measured 2026-11-13, on the board, with a probe that publishes read-backs rather than conclusions.

## What the measurements say

```
clr=0000  if=0022  d=1   cnt=302      compare channel 0, target = start + 300
a=0022  b=0044  d=1  e=1             channel 0 then channel 1, same test twice
cmp: start=0 deadline=3200 reached=3201 late=1 fired=1 en=1 run=1     the HAL, after the fix
```

- **The compare fires**, exactly one count after the programmed value. `late = 1`, twice, in two different
  programs. Everything a timer needs from this silicon exists.
- **The flag for compare channel 0 is bit 1**, and this is *not* a header defect: `efr32mg24_sysrtc.h` defines
  `GRP0_IF_OVF = bit 0`, `GRP0_IF_CMP0 = bit 1`, `GRP0_IF_CMP1 = bit 2`, `GRP0_IF_CAP0 = bit 3`. The code was
  testing bit 0 and reading "never fires" while the flag was set. **The header was right; the code was wrong.**
  An earlier version of this file claimed the header was off by one. It was not, and that claim is retracted
  here — the measurement (`0x22` set for channel 0, `0x44` for channel 1) is consistent with the header all
  along.
- **`GRP0_IF` is write-1-to-clear through the `+0x2000` alias.** A plain store to `0x60` leaves it set.
- **`CFG` cannot be written while the module runs.** SYSRTC lives in the always-on domain and keeps counting
  across a reset, so a warm restart finds `EN=1`; writing `CFG` then faults at exactly `0x500a800c`
  (`CFSR = 0x00008200`, BFAR naming the register, reproduced twice). The fix is the vendor's order: stop,
  `EN_CLR` (`0x2004`), wait for `EN` to read zero, then `CFG`.

## The interrupt belongs to the RAIL blob, not to us

Three measurements in a row, each narrowing the same question:

1. With the timer scheduled, the callback never ran and `irqs` stayed 0. Read-backs after scheduling:
   `NVIC ISER2 = 0x08` (IRQ 67 enabled), `PRIMASK = 0`, `GRP0_IEN = 0x02`, `GRP0_CTRL = 0x01`,
   `GRP0_CMP0 = 0x641`, `CNT = 0x7d7b2`. Everything an interrupt needs was in place — and nothing arrived.
2. `NVIC_ISPR2` read **`0x10`**: bit 4, **IRQ 68 pending**, never taken. We had enabled IRQ 67.
3. Pointing a handler at IRQ 68 as well made the link fail with **`duplicate symbol: SYSRTC_SEQ_IRQHandler`** —
   the RAIL blob defines it.

So: **the group block at `0x60` raises the sequencer line (IRQ 68), and that line belongs to RAIL.** RAIL uses
SYSRTC through PRS to wake at a deadline in EM2, which is what this project already recorded in
`SYSRTC-FOR-THREE-PROTOCOLS.md`; what was missing is that the *interrupt line* is shared with it, not just the
peripheral. An application timer that programs `0x60` is not merely interfering with a register — it is taking
RAIL's interrupt.

The consequence, and the next step: an application timer must use a **different group**, whose base offset is not
yet measured (the header defines two groups; the blob's 0x60 block is the one RAIL uses). Until that offset is
measured, the polled `poll()` is the honest mechanism, and the interrupt path is documented as unavailable rather
than quietly attempted.

## How to measure this again

`crates/efr32mg24-pac/examples/pac_sysrtc_compare_flag.rs` publishes, in one short line, the flags after a clear,
the flags when they change, and the distance between the counter and the programmed value at that moment. A
distance of 1 means the comparison is exact. Two things make it work:

- **Publish short.** An earlier message was cut at 64 bytes and the field that mattered — the counter at the end —
  was the part lost.
- **Publish read-backs, not conclusions.** `ctrl=00000001` says the write stuck; "the channel is enabled" is what
  I believed for two sessions while it was not.

## The four defects found in this peripheral, corrected

| # | What the code believed | Reality | Symptom |
|---|---|---|---|
| 1 | `CMU_SYSRTC0CLKCTRL` at `0x1DC` | `0x240` | clock select never took; `EN` never set; `CMD` faulted |
| 2 | `EN_SET` at `0x1000` | `0x1004` (`0x1000` is `IPVERSION_SET`, read-only) | writes vanished; `EN` stayed 0 |
| 3 | group block at `0x40` (header) | the block RAIL uses is at `0x60` | compare channel never enabled — and, once enabled, it was RAIL's group |
| 4 | `CMP0` flag = bit 0 | bit 1 (**and the header said so**: `GRP0_IF_CMP0`) | polling the wrong bit read "never fires" while the flag was set |

Only #4 is a case of the code ignoring documentation that existed. #1, #2 and #3 are offsets where the code and
the silicon disagreed, and the silicon won each time.
