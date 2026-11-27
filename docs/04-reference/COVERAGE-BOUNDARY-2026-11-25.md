# What this HAL drives, and what it does not

Written 2026-11-25, because "functional coverage is a B" had no target: a grade without a boundary is a grade nobody
can turn into an A. This file is the boundary, stated as a list, so that the 0.1.0 release can promise exactly what is
here and name exactly what is not.

## How the list was made, so it can be remade

The generated PAC is the device's own inventory. Every family it exposes is a line in
`crates/efr32mg2x-pac/src/*_ns.rs` — forty-eight of them on this part:

```
acmp0 acmp1 buram burtc cmu dcdc dmem dpll0 emu eusart0 eusart1 fsrco gpcrc gpio hfrco0 hfrcoem23
hfxo0 hostmailbox i2c0 i2c1 iadc0 icache0 keyscan ldma ldmaxbar letimer0 lfrco lfxo msc mvp pcnt0
prs radioaes scratchpad smu syscfg sysrtc0 timer0 timer1 timer2 timer3 timer4 ulfrco usart0 vdac0
vdac1 wdog0 wdog1
```

Each family was then searched for in `crates/efr32mg2x-hal/src/`. A family with no reference anywhere in the crate is
in the "absent" list below — not "absent as far as I remember", absent as far as `grep -i` can see.

**The first pass of this list was wrong, and how it was wrong is worth keeping.** It searched for the PAC's type names
with their capitalisation (`Prs`, `Msc`, `Letimer0`) and so reported "0" for `msc`, whose unlock key and `READCTRL` write
are in `board.rs`, and for `prs`, which `wdog`'s documentation already names as a TODO. A case-sensitive search over
prose does not find the prose. The tables below were rebuilt with `grep -i`, and every remaining zero was re-checked by
reading the hits it did find: at that time `letimer` and `burtc` appeared only inside `emu`'s comments (`letimer0` has
since been driven and moved to section 1), and `wdog1` only in a doc title.

Modes are listed separately, because a mode is not a family: `EUSART0` is driven, and `EUSART0` in slave mode is not.

## 1. Driven, and each with a record on the part

| family | HAL module | what it does | evidence |
|---|---|---|---|
| `gpio` | `pin`, `board` | output, input, interrupt, routing | `hal_led_blink` |
| `cmu`, `emu`, `hfxo0`, `lfxo`, `lfrco` | `clock`, `emu`, `sleep` | clock tree, energy modes, oscillator start | `hal_hfxo`, `hal_emu_voltage`, `hal_em2_sleep` |
| `sysrtc0` | `sysrtc` | 32.768 kHz counter, compare, 64-bit extended | `hal_sysrtc_wrap`, `hal_sysrtc_wide`, `hal_timer_ticks` |
| `timer0` | `timer`, `pwm`, `buzzer` | count, compare, PWM | `hal_timer_compare`, `hal_timer0_pwm` |
| `timer1`–`timer4` | `timer` | same driver, one instance each, one claim each | `hal_timer_instances` — each built at 1000 Hz and its **overflows** counted against a 50 ms SYSRTC window: `o1=48 o2=48 o3=48 o4=0 stopped=0 top=18999 clean=1 refused=1`, three boots identical; the stopped instance and the never-started one are the controls. Getting there found two defects in a constructor nothing had ever called (the `EN`/`CFG` order, and `clear_overflow` clearing `TIMER0` from every instance) — record in [`TIMER-INSTANCES-2026-11-27.md`](TIMER-INSTANCES-2026-11-27.md) |
| `usart0` | `usart` | asynchronous serial: echo, and a receiver that refuses a broken frame | `hal_usart_link` |
| `eusart1` | `eusart` | the same asynchronous driver on the second block, own `CLKEN1` bit, no per-peripheral clock selector on this part | `hal_eusart1_loopback` — eight distinct bytes out and back through `CFG0.LOOPBK`: `eusart1: n=8 matched=8 seen=8 cfg0=0x00000002 st=0x00003063 control=0 div=297 routed=1 refused=1`, three boots identical. Two controls: loopback off reads zero, and an **unrouted** block also reads zero, which is the finding — the loop closes through the receiver's pad path. Record: [`EUSART1-2026-11-27.md`](EUSART1-2026-11-27.md) |
| `eusart0` | `spi` | SPI **master**, 8-bit, blocking | `hal_spi_loopback` — with a jumper on D9–D10, and a control that reads `ff..ff` without it |
| `i2c0`, `i2c1` | `i2c` | I2C **master**: blocking, async, repeated start, NACK, recovery | `hal_i2c_hardware_imu`, `hal_i2c_async`, `hal_i2c_recovery` |
| `iadc0` | `adc`, `microphone` | single-ended conversion, started by the CPU, by a PRS positive edge, or by a timer through that edge; the microphone path | `hal_adc_read`, `hal_mic_level`, `hal_adc_prs_trigger`, `hal_adc_prs_timer`, `hal_adc_prs_gpio` — `TRIGGER = 0x300` with 8 pulses on PRS channel 0 gave 8 conversions (`prs=8 armB=0`), and the three controls held: arming alone converts once (`armA=1 imm=0`), no pulse converts nothing (`quiet=0`), an unarmed queue ignores edges (`noarm=0`); then TIMER0's overflow on the same channel gave `ev=8` for `ovf=8` with no CPU write in the loop, and four controls remained at zero (`off=0 idle=0 noarm=0 quiet=0`). Records: [`IADC-PRS-TRIGGER-2026-11-26.md`](IADC-PRS-TRIGGER-2026-11-26.md), [`PRS-TIMER-PRODUCER-2026-11-26.md`](PRS-TIMER-PRODUCER-2026-11-26.md) |
| `ldma`, `ldmaxbar` | `dma` | channel ownership, transfers, stop | `hal_dma_stop` |
| `mvp` | `mvp` | matrix/vector co-processor, `binary16` | `hal_mvp_bench` |
| `wdog0` | `wdog` | windowed watchdog, fed with a budget | `hal_wdog_feed` |
| `gpcrc` | `gpcrc` | CRC-32, reflected, the caller adds the final xor | `hal_gpcrc` |
| `letimer0` | `letimer` | 24-bit counter, two compare channels, `TOP`; runs in EM2 | `hal_letimer_compare` — a `TOP` of 1000 with compares at 500 and 250, both matches timed against the SYSRTC, `p1=501 pq=250 p2=499`; record in [`LETIMER-COMPARE-2026-11-26.md`](LETIMER-COMPARE-2026-11-26.md) |
| `prs` | `prs` | routes an event from one peripheral to another without the CPU: `Source` names the producers it encodes (`Off`, `Timer0Overflow`, `Timer0Compare0`, `Timer1Overflow`, `GpioPin`), `connect` writes `ASYNC_CHn_CTRL` whole, `channel_ctrl` reads it back, `pulse` is the software edge | `hal_adc_prs_timer` — TIMER0's overflow on channel 0 converts the IADC once per overflow (`ev=8` for `ovf=8`, four controls at zero), and `hal_adc_prs_gpio` — a pad moved by its own pull resistor paces one conversion per rising edge (`c=4 up=4`), with `EXTIRISE`/`EXTIFALL` proved *not* to be in that path (`fall=4` with the rising mask cleared). Records: [`PRS-TIMER-PRODUCER-2026-11-26.md`](PRS-TIMER-PRODUCER-2026-11-26.md), [`PRS-GPIO-PRODUCER-2026-11-26.md`](PRS-GPIO-PRODUCER-2026-11-26.md) |
| `pcnt0` | `pcnt` | pulse counter fed by a `PRS` producer; its ceiling is pulse **width**, not bandwidth, because the pins are sampled at 32.768 kHz | `hal_pcnt_gpio` — `pcnt: ch=3 … a=4 lv=4 b=2 c=3 … ar=4 br=2`, and `hal_pcnt_timer_rate` sweeps the rate until the count stops moving (`ceiling=0k`). Records: [`PCNT-GPIO-2026-11-26.md`](PCNT-GPIO-2026-11-26.md), [`PCNT-CEILING-2026-11-26.md`](PCNT-CEILING-2026-11-26.md) |
| `acmp0` | `acmp` | the comparator's input mux and its 63-tap reference ladder, read as a ruler | `hal_acmp_ladder` — the ladder swept against a fixed internal source, read three times: `acmp: ok=1 n=252 c1=43 c2=22 c3=43 … lo=833 hi=853 sw=01`, three boots identical; record in [`ACMP-LADDER-2026-11-27.md`](ACMP-LADDER-2026-11-27.md) |
| `radioaes` | `aes` | AES-128 ECB, one block per call, through scatter-gather descriptor chains; the cipher the radio stack uses, not the radio | `hal_aes_kat` — FIPS-197 C.1 and NIST SP 800-38A F.1.1 byte-exact plus a decrypt round trip: `aes: ok=1 n=4 ct=69c4e0d8…c55a c2=3ad77bb4…ef97 hw=0x05030127`; record in [`AES-KAT-2026-11-27.md`](AES-KAT-2026-11-27.md) |
| `burtc` | `burtc` | 32-bit backup counter in the always-on domain, power-of-two prescaler, `EM4GRPACLK` source; keeps counting across a core reset | `hal_burtc_backup` — two counters on the same 32.768 kHz branch over a known window (`d1=4113 s1=32901 r1=125`), four boots identical, and a control that takes the SYSRTC out of the comparison; record in [`BURTC-2026-11-27.md`](BURTC-2026-11-27.md). **Not proved**: surviving a real power cut — that needs the supply pulled, which no instrument here can do |

## 2. Absent: families the device has and this crate does not touch

None of these appears in `crates/efr32mg2x-hal/src/` at all. Families that were on this list and are now driven (`prs`, `pcnt0`, `acmp0`, `radioaes`, `burtc`) moved to section 1 with their records; `acmp1`, which is an instance rather than a family, moved to section 3.
would cost, so that the boundary is a decision and not an omission.

| family | what it is | cost to drive it |
|---|---|---|
| `vdac0`, `vdac1` | 12-bit DACs | ~200 lines each + a meter to read the output |
| `keyscan` | keyboard matrix scanner | ~200 lines + a keypad |
| `dcdc` | the internal DC-DC converter | small, but a power-mode decision, not a driver: the reset default is the LDO |
| `wdog1` | a second watchdog instance | trivial once `wdog0` exists; nothing needs two |
| `smu` | safety management unit | configuration, not a driver |
| `syscfg`, `dmem`, `buram`, `scratchpad`, `hostmailbox`, `icache0` | infrastructure: memory and bus configuration | not touched. (`msc` is **not** here: `Board::take` unlocks it with `0x1B71` and writes `READCTRL` for the flash wait states — one privileged write, not a driver) |
| `hfrco0`, `hfrcoem23`, `fsrco`, `ulfrco`, `dpll0` | clock sources | owned by `clock` through `CMU`; no separate driver is correct here |

## 3. Absent: modes of peripherals that are already driven

This is the list that matters most for honesty, because a reader sees "SPI supported" and assumes the mode they need.

| peripheral | mode | status |
|---|---|---|
| `acmp` | **ACMP1**, and any comparator input arriving on a pin | absent, and the module says so: `acmp/mod.rs` exposes the internal input mux and the 63-tap ladder only. A comparator fed from a pad needs a reference on another pad and two instruments to watch it; neither is on the bench |
| `eusart0` / `usart0` | **SPI slave** | absent as a driver. `spi::Spi0`/`Spi1` are master only. The decision layer exists and is host-tested (`cores::spi::Mode`, `cores::spi::SlaveRx` with its overrun rule), but the register path is not written: it cannot be proven without a master on the other side, and this crate does not ship unproven drivers |
| `i2c0`, `i2c1` | **I2C slave** (address match) | absent as a driver. The state machine is host-tested (`cores::i2c::slave`: seven-bit address match, NACK on a full receive buffer, stretch on an empty transmit one), but 10-bit addressing, general call and the stretch's timing are hardware this crate has not measured |
| `usart0`, `eusart1` | 9-bit / multiprocessor address mode | absent |
| `usart0`, `eusart1` | IrDA, ISO7816 | absent |
| `eusart0` | **I2S** (`I2SCTRL`) | absent. The audio mode of the same peripheral |
| `iadc0` | differential input | deliberately absent, and refused rather than misread: `cores::adc::parse` returns an error for a differential word instead of reading it as single-ended |
| `iadc0` | scan / multiple channels per trigger | absent; one conversion at a time |
| `iadc0` | trigger modes other than software and `PRSPOS` | absent and measured nowhere: `PRSCLKGRP`, `PRSNEG` and `SINGLETRIGACTION = CONTINUOUS` are not written by this crate. Two timer-paced paths do exist and are distinct: `SINGLETRIGSEL = Timer` (the ADC clock group's own timer) is driven by `microphone::Converter`, and a timer routed through PRS to `PRSPOS` is measured in [`PRS-TIMER-PRODUCER-2026-11-26.md`](PRS-TIMER-PRODUCER-2026-11-26.md) |
| `timer0`–`timer4` | input capture | absent as a driver; the decision layer is host-tested (`cores::timer::CaptureSlot` and the wrapping interval arithmetic). Compare and PWM are driven and proven |
| `eusart0` | SPI with a device on the pads | the loopback proof has a jumper on D9–D10; a real device has never spoken on that bus |

## 4. Not this crate

The radio (BLE, 802.15.4) lives in `crates/efr32mg24-radio`, and the PAC below this crate is generated from the SVD.
Neither is "missing" — they are elsewhere, and a reader looking for the radio should not conclude from this file that it
does not exist.

## 5. What this means for 0.1.0

A first release does not have to be complete. It has to be **specific**: the README already says what is proven, what is
compiled and unmeasured, and what is not implemented. The proposal, for the operator to accept or change:

- **Claims**: every row of section 1 that carries a dated record. The four rows whose evidence is "compiled, not
  measured" (`timer1`–`timer4`) are named as such.
- **Names as not implemented**: sections 2 and 3, in the README, by family and by mode. A sentence like "SPI is
  implemented" is the sentence that shipped a false claim in the previous release attempt; "SPI master is implemented,
  SPI slave is not" cannot.
- **Not claimed at all**: anything in section 4.

The reason this belongs in the release decision rather than in a backlog: the previous 0.1.0 went to the registry
advertising "Phase 5 Tier 2 – USART Complete, I2C/SPI/Timers Next" while those were already implemented, and pointing at
a plan file that did not exist. Nothing in the tree was wrong; the *claim* was. A boundary written down is what makes the
next claim checkable.