# sysrtc

A tick timer over SYSRTC.

## What it is

A tick timer over SYSRTC.

A HAL service, not a vendor glue layer: one peripheral, three protocols (Bluetooth, Zigbee, Thread), one
driver. The vendor stack's entry point is a thin adapter over this.

# The two offsets that cost forty flash cycles

Measured 2026-11-13 with the vendor headers themselves (`offsetof` against efr32mg24_cmu.h and
efr32mg24_sysrtc.h), because both were wrong in the code paths this module first used:

```text
CMU.SYSRTC0CLKCTRL   0x240    the PAC's accessor writes 0x1DC -- a different register entirely
SYSRTC.EN_SET        0x1004   the alias page mirrors the layout; 0x1000 is IPVERSION_SET (read-only)
```

The consequences were not obvious from the symptoms. Writing the clock select to `0x1DC` left
`SYSRTC0CLKCTRL` at its disabled value, so the counter's clock never ran; a peripheral with no clock
cannot accept a synchronised write, and `CMD = START` came back as `CFSR = PRECISERR` with
`BFAR = SYSRTC0->CMD` — a bus error that looked like a fault in the driver's logic. Writing `1` to
`0x1000` hit `IPVERSION_SET`, which is read-only, so `EN` stayed 0 and the module never enabled.

Both are reached here through raw pointers with the address written next to the write, because the PAC
cannot be trusted for these two registers and a comment is cheaper than another forty cycles.

# Order

The vendor's (`sl_hal_sysrtc_init` + `sleeptimer_hal_init_timer`, Gecko SDK v4.4.5):
bus clocks, clock select, wait, `CFG`, group config, interrupts off and cleared, `EN_SET`, `CMD = START`.
Every synchronised register is read or written across a `SYNCBUSY` wait.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `CLKEN0_LFRCO` | const | `CMU_CLKEN0` bits: LFRCO (22) and SYSRTC0 (30). |
| `CLKEN0_SYSRTC0` | const | — |
| `CLKSEL_LFRCO` | const | `CMU_SYSRTC0CLKCTRL.CLKSEL`: 0 = disabled, 1 = LFRCO. |
| `CLKSEL_MASK` | const | — |
| `CMD_START` | const | `SYSRTC_CMD.START` (bit 0) and `.STOP` (bit 1). |
| `CMD_STOP` | const | — |
| `CMU_CLKEN0` | const | `CMU_CLKEN0`, relative to the CMU base. |
| `CMU_SYSRTC0CLKCTRL` | const | `CMU_SYSRTC0CLKCTRL`, relative to the CMU base. |
| `CTRL_CMP0EN` | const | `SYSRTC_GRP0_CTRL.CMP0EN`. |
| `EN_CLR` | const | `SYSRTC_EN_CLR`, the write-1-to-clear alias. |
| `EN_EN` | const | `SYSRTC_EN.EN`. |
| `EN_SET` | const | `SYSRTC_EN_SET`, the write-1-to-set alias of `EN`. |
| `GRP0_ALL` | const | Bits 4..7 mirror the channel flags at group level. |
| `GRP0_CMP0` | const | `SYSRTC_GRP0_IF` bits, from efr32mg24_sysrtc.h: `OVF` = bit 0, `CMP0` = **bit 1**, `CMP1` = bit 2, `CAP0` = bit 3. |
| `GRP0_CMP0VALUE` | const | — |
| `GRP0_CMP1` | const | Compare channel 1's flag, same shift. |
| `GRP0_CTRL` | const | — |
| `GRP0_IEN` | const | — |
| `GRP0_IF` | const | **Group 0, at the offsets the vendor header gives.** Group 1 (`0x60`) is what the RAIL blob drives, and its events arrive on IRQ 68, whose handler RAIL defines — an application timer in that group takes RAIL's interrupt. |
| `GRP0_OVF` | const | The counter wrapped past 2^32 (GRP0_IF bit 0). |
| `STATUS_RUNNING` | const | `SYSRTC_STATUS.RUNNING`. |
| `Sysrtc` | struct | The SYSRTC peripheral, once. |
| `acknowledge_compare_interrupt` | fn | Acknowledge the compare interrupt from a handler, where no `Sysrtc` instance is reachable. |
| `enable_clocks` | fn | Gate the SYSRTC's bus clocks and select its reference clock, **without touching the module itself**. |
| `enable_overflow_interrupt` | fn | Count the counter's wraps with the overflow interrupt instead of polling for them. |
| `ensure_running` | fn | Make sure the counter is **running** before the Bluetooth stack starts. |
| `fn` | const | Ticks per millisecond at LFRCO, rounded down: 32.768 kHz is not a multiple of 1000, so a thousand milliseconds lose 768 ticks (about 23 ms/s). |
| `is_enabled` | fn | — |
| `is_running` | fn | — |
| `now` | fn | — |
| `now_wide` | fn | The monotonic **64-bit** tick, widened from the 32-bit counter by counting its wraps. |
| `now_wide_global` | fn | The monotonic 64-bit tick **without a handle**. |
| `schedule_at` | fn | — |
| `schedule_at_global` | fn | Arm the comparator **without a handle**, the companion of [`Sysrtc::now_wide_global`]. |
| `schedule_in` | fn | — |
| `service_interrupts` | fn | Clear whichever of the two events are pending, and count an overflow. |
| `set_counter` | fn | Move the counter, for a bench test that cannot wait 36.4 hours for a wrap. |
| `take` | fn | Take the peripheral and put it in a known state: LFRCO, compare channel 0 armed, counter running. |
| `take_compare_event` | fn | — |
| `unmask_interrupt` | fn | Unmask the module's interrupt line. |
| `wraps` | fn | How many wraps have been counted, by whichever mechanism is armed. |

## Registers it touches

`Interrupt`, `Sysrtc0Ns`

## What was measured

```
    probe: t=1865376137 c=29605      SYSRTC and DWT at the first read
    cmp: start=…57 deadline=…77 reached=…78 late=1 fired=1 en=1 run=1      PASS   SYSRTC compare fires once, late=1
```

Controls and the full story: [DWT-NEEDS-A-DEBUG-SESSION.md](../../../../docs/04-reference/DWT-NEEDS-A-DEBUG-SESSION.md), [HAL-REVIEW-BRIEF-2026-11-25.md](../../../../docs/04-reference/HAL-REVIEW-BRIEF-2026-11-25.md)

## What is not proven

- C -- advanced | complete to stage 2 | `adc`, `dma`, `emu`, `rtc`/`sysrtc`, `wdog`, `gpcrc`, `mvp`, `letimer`, `prs`; DMA stages 3 and 4 deferred |. (PLAN.md (`../../../../docs/03-development/PLAN`))
- ~~H28~~ | EM2 entry and exit sequence | **CLOSED 2026-11-21** (`B3`): ten ~1 s naps in EM2 woken by the SYSRTC comparator, intervals 0.92-1.03 s, mean 0.99, total 9.92 s, eleven interrupts. (HAL-GAP-ANALYSIS-2026-11-18.md (`../../../../docs/04-reference/HAL-GAP-ANALYSIS-2026-11-18`))

## Using it

See `examples/hal_burtc_backup.rs`, `examples/hal_cmu_sources.rs`, `examples/hal_em2_sleep.rs`, `examples/hal_eusart1_loopback.rs`, `examples/hal_letimer_compare.rs`, `examples/hal_sysrtc_wide.rs`, `examples/hal_sysrtc_wrap.rs`, `examples/hal_timer_compare.rs`, `examples/hal_timer_instances.rs`
