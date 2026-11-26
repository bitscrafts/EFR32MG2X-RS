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
| `timer1`–`timer4` | `timer` | same driver, one instance each | compiled, not measured |
| `usart0` | `usart` | asynchronous serial: echo, and a receiver that refuses a broken frame | `hal_usart_link` |
| `eusart1` | `usart` | the same driver, second instance | compiled, **not measured** — no example routes it |
| `eusart0` | `spi` | SPI **master**, 8-bit, blocking | `hal_spi_loopback` — with a jumper on D9–D10, and a control that reads `ff..ff` without it |
| `i2c0`, `i2c1` | `i2c` | I2C **master**: blocking, async, repeated start, NACK, recovery | `hal_i2c_hardware_imu`, `hal_i2c_async`, `hal_i2c_recovery` |
| `iadc0` | `adc`, `microphone` | single-ended conversion, started by the CPU, by a PRS positive edge, or by a timer through that edge; the microphone path | `hal_adc_read`, `hal_mic_level`, `hal_adc_prs_trigger`, `hal_adc_prs_timer` — `TRIGGER = 0x300` with 8 pulses on PRS channel 0 gave 8 conversions (`prs=8 armB=0`), and the three controls held: arming alone converts once (`armA=1 imm=0`), no pulse converts nothing (`quiet=0`), an unarmed queue ignores edges (`noarm=0`); then TIMER0's overflow on the same channel gave `ev=8` for `ovf=8` with no CPU write in the loop, and four controls remained at zero (`off=0 idle=0 noarm=0 quiet=0`). Records: [`IADC-PRS-TRIGGER-2026-11-26.md`](IADC-PRS-TRIGGER-2026-11-26.md), [`PRS-TIMER-PRODUCER-2026-11-26.md`](PRS-TIMER-PRODUCER-2026-11-26.md) |
| `ldma`, `ldmaxbar` | `dma` | channel ownership, transfers, stop | `hal_dma_stop` |
| `mvp` | `mvp` | matrix/vector co-processor, `binary16` | `hal_mvp_bench` |
| `wdog0` | `wdog` | windowed watchdog, fed with a budget | `hal_wdog_feed` |
| `gpcrc` | `gpcrc` | CRC-32, reflected, the caller adds the final xor | `hal_gpcrc` |
| `letimer0` | `letimer` | 24-bit counter, two compare channels, `TOP`; runs in EM2 | `hal_letimer_compare` — a `TOP` of 1000 with compares at 500 and 250, both matches timed against the SYSRTC, `p1=501 pq=250 p2=499`; record in [`LETIMER-COMPARE-2026-11-26.md`](LETIMER-COMPARE-2026-11-26.md) |

## 2. Absent: families the device has and this crate does not touch

None of these appears in `crates/efr32mg2x-hal/src/` at all. Each line says what the peripheral is and what driving it
would cost, so that the boundary is a decision and not an omission.

| family | what it is | cost to drive it |
|---|---|---|
| `prs` | peripheral reflex system: routes an event from one peripheral to another without the CPU | ~250 lines, of which the crate now drives a channel: `prs::Source` names the producers it encodes (`Off`, `Timer0Overflow`, `Timer0Compare0`, `Timer1Overflow`, `GpioPin`), `Prs::connect` writes `ASYNC_CHn_CTRL` whole and `Prs::channel_ctrl` reads it back, and `Prs::pulse` is the software edge. Measured: TIMER0's overflow through channel 0 converts the IADC once per overflow (`ev=8` for `ovf=8`), with controls that keep the channel's source the only thing that changed ([`PRS-TIMER-PRODUCER-2026-11-26.md`](PRS-TIMER-PRODUCER-2026-11-26.md)). **Not driven**: no consumer word other than the IADC's single trigger is written by this crate, the synchronous channels are untouched, and a GPIO producer is encoded but unmeasured |
| `pcnt0` | pulse counter with external inputs, direction and S0/S1 | ~200 lines + a signal source to test it, which is bench work |
| `burtc` | backup RTC in the always-on domain | ~150 lines; hard to prove, because "it kept time with the main supply off" needs a power cut |
| `vdac0`, `vdac1` | 12-bit DACs | ~200 lines each + a meter to read the output |
| `acmp0`, `acmp1` | analogue comparators | ~150 lines each + a reference voltage to compare against |
| `keyscan` | keyboard matrix scanner | ~200 lines + a keypad |
| `dcdc` | the internal DC-DC converter | small, but a power-mode decision, not a driver: the reset default is the LDO |
| `radioaes` | AES used by the radio stack | belongs with the radio, not with this crate |
| `wdog1` | a second watchdog instance | trivial once `wdog0` exists; nothing needs two |
| `smu` | safety management unit | configuration, not a driver |
| `syscfg`, `dmem`, `buram`, `scratchpad`, `hostmailbox`, `icache0` | infrastructure: memory and bus configuration | not touched. (`msc` is **not** here: `Board::take` unlocks it with `0x1B71` and writes `READCTRL` for the flash wait states — one privileged write, not a driver) |
| `hfrco0`, `hfrcoem23`, `fsrco`, `ulfrco`, `dpll0` | clock sources | owned by `clock` through `CMU`; no separate driver is correct here |

## 3. Absent: modes of peripherals that are already driven

This is the list that matters most for honesty, because a reader sees "SPI supported" and assumes the mode they need.

| peripheral | mode | status |
|---|---|---|
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