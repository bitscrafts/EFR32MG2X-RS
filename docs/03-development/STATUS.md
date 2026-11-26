# Project Status Report

**Date**: 2026-11-26
**Project**: EFR32MG24 Rust support (PAC + HAL + radio + Bluetooth + BSP)
**Board**: Seeed XIAO MG24 Sense, EFR32MG24B220F1536IM48-B, on the bench at `lnx`
**Phase**: D2 verified on hardware -- a phone advertises, connects, reads and writes, and the BME680 answers as the
standard Environmental Sensing service. Open inside D2: **D2f pairing** and **D2g BME680 gas resistance**.

Every number below was counted from the tree or read back off the part. A number that came from a record names the
record. What has no hardware proof says so.

---

## What exists, counted from the tree

| crate | what it is | counted here |
|---|---|---|
| `efr32mg2x-pac` | register access, generated from the B220 SVD with svd2rust | 2,633 files, 103 peripheral instances in 46 families |
| `efr32mg2x-hal` | peripherals, buses and drivers, no board facts | 29 `pub mod`, 29 examples, 14,712 lines under `src/` |
| `efr32mg24-rail-sys` | RAIL FFI bindings | 7 examples, needs the vendor blobs at build time |
| `efr32mg24-bluetooth-sys` | BGAPI FFI bindings plus a safe blocking API | 1 example |
| `efr32mg24-radio` | Bluetooth API, RAIL setup, diagnostics | 7 examples |
| `xiao-mg24-bsp`, `xiao-expansion-base-bsp` | the board: which pin the LED, the IMU, the microphone and the OLED sit on | 28 + 10 examples |
| `libraries/` | `no_std` logic with no hardware: `bt-heap`, `aes-modes`, `aes-ctr-radio`, `motion-detection`, `record-core` | host-tested, never published |

The HAL counts are the ones `scripts/gate.sh` compares against the tree, so they cannot drift quietly. The HAL README
carries the per-example table of what came back off the part, generated from the example headers rather than typed by
hand.

## What this HAL drives, and what it does not

The boundary is a list, family by family, with the cost of each missing one:
[`COVERAGE-BOUNDARY-2026-11-25.md`](../04-reference/COVERAGE-BOUNDARY-2026-11-25.md). In short: `gpio`,
`cmu`/`emu`/`hfxo0`/`lfxo`/`lfrco`, `sysrtc0`, `timer0`-`timer4`, `usart0`, `eusart0`/`eusart1`, `i2c0`/`i2c1`, `iadc0`,
`ldma`/`ldmaxbar`, `mvp`, `wdog0`, `gpcrc` and `letimer0` are driven, each with a record on the part. `prs` is driven as
far as one channel with a named producer, and that producer has been a timer's overflow, a CPU pulse and a GPIO pad.
`pcnt0`, `burtc`, `acmp0`/`acmp1`, `vdac0`/`vdac1`, `keyscan`, `radioaes`, `dcdc`, `dpll0`, `ulfrco`, `fsrco` and
`hfrcoem23` are not touched at all.

## Measured on the part (2026-11-26)

| what | record, read back off the board | document |
|---|---|---|
| `LETIMER0`'s top is `CTRL.CNTTOPEN` | control `A:cnt0=16777215 w1=0 w2=0 if=0x0`; treatment `B:cnt0=1000 w1=1001 w2=2002 if=0x4` | [`LETIMER-TOP-2026-11-26.md`](../04-reference/LETIMER-TOP-2026-11-26.md) |
| `LETIMER0` delivery: 24-bit counter, two compares below the top | `clk=0x00000101 top=1000 comp0=500 comp1=250 refused=1 cnt0=1000 p1=501 qe=0 ql=1 pq=250 wrap=1 if1=0x1 ifc=0x0 if2=0x1 p2=499 cnt=499` | [`LETIMER-COMPARE-2026-11-26.md`](../04-reference/LETIMER-COMPARE-2026-11-26.md) |
| an IADC conversion started by a PRS edge, with the controls that make it mean something | `trig=prs0 refused=1 armA=1 imm=0 armB=0 prs=8 v0=0x26f v7=0x2a7 quiet=0 noarm=0` | [`IADC-PRS-TRIGGER-2026-11-26.md`](../04-reference/IADC-PRS-TRIGGER-2026-11-26.md) |
| a timer's overflow converted once per overflow, with no CPU write in the loop | `trig=prs0 hz=500 period=38000 src=0x000c2101 off=0 idle=0 noarm=0 ev=8 quiet=0 v0=0x0000026d vl=0x00000238 ovf=8 ch0=0x000c0000` | [`PRS-TIMER-PRODUCER-2026-11-26.md`](../04-reference/PRS-TIMER-PRODUCER-2026-11-26.md) |
| a GPIO pad pacing conversions through the PRS, with a quiet-pad, an `Off`-channel and a wrong-pin control | `ch=3 sel=0x00002000 isel=0x00003000 din=1 a=0 b=4 c=4 mv=8 up=4 off=0 other=0 pin0=4 up0=4 long=8 upg=8 fall=4 ifb=0x00000009 ifc=0x00000001` | [`PRS-GPIO-PRODUCER-2026-11-26.md`](../04-reference/PRS-GPIO-PRODUCER-2026-11-26.md) |

The rest of the hardware records -- I2C read and recovery, the USART receiver refusing a broken frame, SPI loopback with
a jumper and a control without it, the WDOG, the CRC, the MVP benchmark, the microphone, the IMU -- are in the HAL
README's measured section and in `docs/04-reference/`, one document per measurement.

## Open

| item | state | where |
|---|---|---|
| D2f pairing and bonding | the PSA layer is written and verified on the part; the security manager fails at a DHKey check with no PSA call made (`pg=0 pe=0 pa=0`), so the fault is before the key exchange, and the component stays out of the product build | [`PAIRING-D2F-STATE.md`](../04-reference/PAIRING-D2F-STATE.md), `BACKLOG.md` |
| D2g BME680 gas resistance | the heater, `res_heat` and the `par_g*` curves are not implemented, so the reading that separates a 680 from a 688 is missing | `BACKLOG.md` |
| the delay runs 2.3x short | measured against two clocks and an independent DWT count: a "10 ms" delay produced 82 680 core cycles at 19 MHz | [`DELAY-CLOCK-DEFECT.md`](../04-reference/DELAY-CLOCK-DEFECT.md) |
| DMA stages 3 and 4 | linked descriptors and interrupt-driven async are deferred, not broken | `BACKLOG.md` |
| the root README's project metrics | still say "12 peripheral modules" and "83 files, 26 checked"; the HAL alone has 29 public modules and 29 examples, and the tree has 130 `.rs` files under `crates/*/examples/` | queued as a documentation item |
| absent families | `pcnt0`, `burtc`, `acmp`, `vdac`, `keyscan`, `radioaes`, `dcdc`, `dpll0`, `ulfrco`, `fsrco`, `hfrcoem23` | `COVERAGE-BOUNDARY-2026-11-25.md` |

## Where the state lives

- `README.md` -- the phase table and the Gantt, and what the project promises.
- `BACKLOG.md` -- the open work, in order.
- this file -- the state, counted from the tree and the part.
- `docs/04-reference/COVERAGE-BOUNDARY-2026-11-25.md` -- peripheral by peripheral, what is driven and what is not.
- the crate READMEs -- per-example evidence, generated and gate-checked.
- `LOG.md` -- the milestones, with what each one cost.

## History

This file used to be a phase-by-phase report written in January 2026 and last touched in June, still announcing 12
modules, 27 examples and files named `08_adc.rs`. Those sections were false by the time the HAL had 29 modules, and a
status page that is false is worse than a short one. The old text is in git history, not lost:
`git show 043c7cc:docs/03-development/STATUS.md` (and `BACKLOG.md`, `PLAN.md` beside it). A timestamped copy also sits
in `.archive/` on the machine that made this change, but that directory is gitignored, so the commit is the copy that
survives a clone. `LOG.md` keeps what actually happened, in order.