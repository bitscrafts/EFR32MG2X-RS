# efr32mg2x-hal — HAL for the EFR32MG24 (XIAO MG24 Sense)

High-level, type-safe Hardware Abstraction Layer for EFR32MG24 microcontrollers.

> **Status**: 20 peripheral drivers in the tree. What is measured on the board, and what merely compiles, is listed
> below rather than summarised in a phase number that has been stale since June 2026.

## Overview

This crate provides high-level, safe abstractions over EFR32MG24 peripherals with embedded-hal v1.0 trait implementations.

### Where this crate sits

```
efr32mg2x-pac          the register access layer: one type per peripheral, offsets from the SVD
      |
efr32mg2x-hal   <--    this crate: peripherals, buses and drivers, no board facts
      |
efr32mg2x-xiao-bsp     the board: which pin the LED is on, which pins power the IMU and the microphone
```

### What this crate uses from `efr32mg2x-pac`

Read from the source, so it cannot drift:

| what it takes | where it is used |
|---|---|
| `Acmp0S` | `src/acmp/mod.rs`, `src/board.rs` |
| `BurtcNs` | `src/board.rs`, `src/burtc/mod.rs` |
| `CmuNs` | `src/mvp/mod.rs` |
| `CmuS` | `src/acmp/mod.rs`, `src/adc/mod.rs`, `src/burtc/mod.rs` (+7 more) |
| `EmuS` | `src/board.rs`, `src/emu/mod.rs` |
| `Eusart0S` | `src/board.rs`, `src/spi/mod.rs` |
| `Eusart1S` | `src/board.rs`, `src/spi/mod.rs` |
| `GpcrcNs` | `src/board.rs`, `src/gpcrc.rs` |
| `GpioNs` | `src/board.rs` |
| `GpioS` | `src/board.rs` |
| `Hfrcoem23S` | `src/clock/sources.rs` |
| `Hfxo0Ns` | `src/clock/clocks.rs` |
| `I2c0Ns` | `src/board.rs`, `src/i2c/async_impl.rs`, `src/i2c/mod.rs` |
| `I2c0S` | `src/i2c/async_impl.rs`, `src/i2c/mod.rs` |
| `I2c1S` | `src/board.rs`, `src/i2c/mod.rs` |
| `Iadc0S` | `src/adc/mod.rs`, `src/board.rs`, `src/microphone.rs` |
| `Interrupt` | `src/dma/irq.rs`, `src/extint.rs`, `src/i2c/async_impl.rs` (+1 more) |
| `LdmaS` | `src/dma/irq.rs`, `src/dma/linked.rs`, `src/dma/mod.rs` |
| `LdmaxbarS` | `src/dma/mod.rs` |
| `Letimer0Ns` | `src/board.rs`, `src/letimer.rs` |
| `MvpNs` | `src/board.rs`, `src/mvp/mod.rs` |
| `Pcnt0Ns` | `src/board.rs`, `src/pcnt.rs` |
| `Peripherals` | `src/adc/mod.rs`, `src/board.rs`, `src/buzzer.rs` (+13 more) |
| `PrsS` | `src/board.rs`, `src/prs.rs` |
| `RadioaesS` | `src/aes/mod.rs`, `src/board.rs` |
| `Sysrtc0Ns` | `src/rtc/mod.rs`, `src/sysrtc.rs` |
| `Timer1S` | `src/board.rs` |
| `Timer2S` | `src/board.rs` |
| `Timer3S` | `src/board.rs` |
| `Timer4S` | `src/board.rs` |
| `Usart0S` | `src/board.rs`, `src/spi/mod.rs`, `src/usart/mod.rs` |
| `Wdog0Ns` | `src/board.rs`, `src/wdog/mod.rs` |
| `X` | `src/cores/slots.rs` |
| `acmp0_s` | `src/acmp/mod.rs` |
| `cmu_s` | `src/board.rs`, `src/pcnt.rs` |
| `emu_ns` | `src/sleep.rs` |
| `gpio_s` | `src/board.rs`, `src/bus/bitbang.rs`, `src/pin.rs` |
| `i2c0_ns` | `src/i2c/async_impl.rs` |
| `iadc0_s` | `src/board.rs`, `src/microphone.rs` |
| `letimer0_ns` | `src/letimer.rs` |
| `pcnt0_ns` | `src/pcnt.rs` |
| `prs_s` | `src/adc/mod.rs`, `src/pcnt.rs` |
| `timer0_s` | `src/board.rs`, `src/pwm.rs` |


**Target Hardware**: Seeed Studio XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
- **CPU**: ARM Cortex-M33 @ 78 MHz with FPU and TrustZone-M
- **Flash**: 1536 KB
- **RAM**: 256 KB
- **Radio**: 2.4 GHz (Bluetooth 5.3, Zigbee, Thread, Matter)

## What is in the tree

Every peripheral family the crate drives, one module each -- a peripheral, a bus or a driver (plus `prelude`, which
only re-exports):

```
acmp  adc  aes  board  burtc  bus  buzzer  clock  delay  display  dma  dwt  emu  eusart
extint  gpcrc  i2c  letimer  microphone  mvp  pcnt  pin  prs  pwm  record  route
rtc  sensors  sleep  spi  sysrtc  timer  usart  wdog
```

The previous version of this file said "USART complete, I2C/SPI/Timers next". That was wrong in both directions: I2C,
SPI and the timers are here, and USART is here but has never been watched doing anything on the board.

**Runnable examples**: <https://github.com/bitscrafts/efr32mg2x-examples> — a clone, a build and a guarded
flash, one crate per layer (PAC, HAL, radio, board, Expansion Base). Not published to crates.io: an examples-only
crate has no binary, so it installs only with `--example`, and its crate page would render nothing.

## What is in the tree, measured

35 public modules (`pub mod` in `src/lib.rs`) and 43 examples. A date next to an example means a record was
read back off the part — not that the code compiled; the table below is counted by the README generator, not by hand.

**Watched on the board (2026-11-15)**

| example | what came back |
|---|---|
| `hal_led_blink` | `PASS hal: pin primitive drives PA7, 1 Hz`, and the LED was watched blinking |
| `hal_i2c_bitbang` | `PASS scan n=3 3c 51 76` — the panel, the RTC and the BME680 answered (measured with the Expansion Base fitted) |
| `hal_rtt_hello` | heartbeats 0..8 read off RTT with `probe-rs` while the core ran |

**Watched on the board (2026-11-23)**

| example | what came back |
|---|---|
| `hal_i2c_recovery` | `PASS i2c-rec: antes=0x6a erro=1 depois=0x6a wr=1 rd6=1 nz=1 volta=0x40` — a refusal on an address where nothing is wired, the bus still answering after it, a plain `write`, and a six-byte IMU burst carrying real data |
| `hal_dma_stop` | `PASS dma-stop: st0=0 armed=1 stA=1 busyA=0 stS=0 ctl=1 stRe=0 stS2=0` — the channel is enabled while armed, disabled by `stop()`, and a blocking `transfer()` no longer leaves it armed |

**Ran, and the record says it did not work**

| example | record |
|---|---|
| — | none: every example that carried a red record on 2026-11-23 was closed on 2026-11-24, below |

**Watched on the board (2026-11-24) — the six that were red**

| example | what came back | what was actually wrong |
|---|---|---|
| `hal_timer_compare` | `PASS cmp: start=1850579377 deadline=1850582577 reached=1850582578 late=1 fired=1 en=1 run=1` | nothing: the compare fired one tick after its deadline, three thousand ticks after the flag was drained |
| `hal_imu_read` | `PASS imu 6a x-351 226 940 \|a\|=1028` | a `copy_from_slice` of 7 bytes into `buf[..8]` panicked before the record was written; a silent panic in an instrument reads as missing data |
| `hal_timer_ticks` | `PASS rtc: ticks=3277 cyc=1896280 fast=18961kHz late=1 fired=1 run=1` | the instrument waited on the clock it was testing. The reference is now SysTick; see [`docs/04-reference/DWT-NEEDS-A-DEBUG-SESSION.md`](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/04-reference/DWT-NEEDS-A-DEBUG-SESSION.md) |
| `hal_emu_voltage` | `PASS emu: VS2->VS1 5/5 , VS1->VS2 5/5`, three boots | the example overwrote its own verdict with an `info` line every second, so a reader never saw a verdict at all |
| `hal_mic_level` | `PASS mic 8 rms=7 pk=18 n=128` | nothing: the LED PWM that fought the microphone for TIMER0 was removed on 2026-11-21 and the README kept the old failure |
| `hal_wdog_feed` | `PASS wdog: fed 95 st80000000 sb0 lock1` | `feed()` wrote `CMD` without waiting for the previous clear to cross into the WDOG clock domain; the second of two back-to-back feeds was refused with a bus fault |

**Watched on the board (2026-11-25)**

| example | what came back | the control that makes it mean something |
|---|---|---|
| `hal_hfxo` | `PASS hfxo: sel=1 before=19M after=39M tick=32974 err=206` — the crystal selected, the core moved from 19 to 39 MHz, and a 1000 ms delay lasted 1006 ms by the SYSRTC | a delay still calibrated for 19 MHz would have taken ~2050 ms, i.e. ~67 000 ticks instead of 32 768: the window is narrow enough to tell the two apart |
| `hal_adc_read` | `PASS adc: gnd=0 ch0=0 ref=VBGR` — the converter returns zero codes for its own ground channel | the signal path is `hal_mic_level` (`rms=7 pk=18 n=128`), not this example: a converter that only ever returns zero cannot prove itself |
| `hal_spi_loopback` | `PASS spi: n=8 ok=8 err=0 out=a5..34 in=a5..34` — eight bytes out on MOSI, the same eight back on MISO, with a jumper between D10 and D9 | **measured, not argued**: routing MISO to an unjumped pin gives `FAIL spi: n=8 ok=1 err=0 out=a5..34 in=ff..ff` — a floating line reads `0xff`, which matches one byte of the pattern, so the "all eight" rule is what refuses it |
| `hal_spi_masters` | `PASS spi3: n=8 eusart0=8 usart0=8 eusart1=8 cross=0 control=0 err=0` — the three SPI masters (`EUSART0`, `USART0`, `EUSART1`) one after the other on the same pads, three boots agreeing | **measured**: each master sends eight bytes that share no byte with the other two, so the cross-count must be zero; the control sends a byte nobody sends and must match nothing. The first control — re-routing MISO off — read `control=8` once the jumper was fitted, because the wire ties the pads physically; the control was replaced, not explained away |
| `hal_usart_link` | `PASS usart: baud=115200 out=6 in=6 ok=6 45 46 52 33 32 21` — `EFR32!` out and the same six bytes back, with a jumper between D6 and D7; and with the peripheral's own loopback, `usart: selftest n=6 ok=6 bad=0 rxerr=0` | two defects measured on the way: a six-byte burst overruns the two-deep receiver (`in=3 ok=2 45 46 21`), so the instrument is one byte at a time; and a floating RX line reports `0x00` often enough that a byte-counting echo test passes on a board with nothing attached — `echo n=271` was that test, and the refusal that replaced it is in `docs/04-reference/USART-RX-REFUSAL-2026-11-25.md` |
| `hal_one_of_each` | `PASS one-of-each: sysrtc=1 sysrtc2=1 adc=1 adc2=1 wdog=1` — the first ask for SYSRTC and for the ADC succeeded, the *second* ask for each was refused, and a third peripheral was still free | the refusal is per slot, not a global stop: the same firmware proves both at once, so a table that simply refused everything could not pass |
| `hal_gpcrc` | `PASS gpcrc: poly=crc32 raw=340bc6d9 zlib=cbf43926 want=cbf43926 diff=1 seed=1` — the accelerator's result complemented is the published `zlib` check value for `123456789` | a matching number alone would also come from a driver returning a constant, so the record carries `diff=1` (a different message gives a different CRC) and `seed=1` (a different seed does too, which is what proves `CMD.INIT` loads it). The reference-manual description of this engine as MSB-first is wrong on this part, and the measurement is what says so |
| `hal_usart_rx_recovery` | `PASS usartrec: overran=1 err_pre=0 drained=3 err_drain=0 ok=6 bad=0` — an overrun latched, three bytes left in the FIFO, and the six bytes sent afterwards all accepted and identical | the part's `IF` flags latch and **nothing clears them**: six mechanisms measured, all leaving `RXOF` set. So the per-byte verdict had to come from `RXDATAX`, and this record is the one that shows the receiver surviving its first glitch instead of refusing for ever |
| `hal_mvp_bench` | `PASS mvp: n=64 reps=256 tick_mvp=123 tick_cpu=288 mvp_us_x100=1466 cpu_us_x100=3433 ratio_x100=234`, **no probe attached**, repeated within one tick over four boots | the clock is the SYSRTC, not the DWT: a counter that only runs under a debugger cannot support a claim about a product. The batch (256 additions) is what makes a 30.5 µs tick enough resolution |

**What each module is for**
- `record` is the one place a line of evidence is formatted, and it used to be eight: an instrument that prints `5` for one byte and `12` for the next cannot be read back without knowing which is which. Two digits always, every digit of a `u64`, `i64::MIN` without overflow, no allocation, and a truncation flag so a cut field is never read as a zero field. Its arithmetic is host-tested in `libraries/record-core`.
- `i2c` drives the hardware I2C0 peripheral, and on this board it reads and writes: `hal_i2c_hardware_imu` and
  `hal_i2c_recovery` are the examples that reach it. What looked like a silicon defect was two driver defects — the
  last byte of a reception has to be read with NACK, and the flags are only cleared through the `IF_CLR` alias — and
  both are in [`docs/04-reference/I2C0-READ-DEFECT.md`](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/04-reference/I2C0-READ-DEFECT.md) and [`docs/04-reference/I2C-RECOVERY-STATE.md`](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/04-reference/I2C-RECOVERY-STATE.md). `bus::BitBangI2c`
  is a different bus, for the Expansion Base's PC4/PC5 pair; I2C0 reaches one pin pair at a time.
- `clock` carries a bricking warning in its own source (`clock/clocks.rs`): an earlier implementation switched SYSCLK to
  HFXO before the HFXO was running, which stopped the CPU clock and locked the debug access port. No example touches it yet,
  on purpose. See [`docs/01-getting-started/BRICKING_INCIDENT_2026-06-29.md`](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/01-getting-started/BRICKING_INCIDENT_2026-06-29.md).

## Coverage: what this crate drives, and what it does not

A reader sees "SPI supported" and assumes the mode they need. This section says which mode, which peripheral and which
part of the device is here, and — as importantly — what is not. The full inventory, generated from the PAC's own list of
peripheral families and re-checked family by family, is
[`docs/04-reference/COVERAGE-BOUNDARY-2026-11-25.md`](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/04-reference/COVERAGE-BOUNDARY-2026-11-25.md).

### Driven, and proven on the part

| Peripheral | What is here | Proven by |
|---|---|---|
| `GPIO` | output, input, external interrupt on both vectors, peripheral routing | `hal_led_blink`, `hal_gpio_interrupt` |
| `CMU`, `EMU`, `HFXO`, `LFXO`, `LFRCO` | clock tree, energy modes, oscillator start | `hal_hfxo`, `hal_emu_voltage`, `hal_em2_sleep` |
| `SYSRTC0` | 32.768 kHz counter, compare, 64-bit extension | `hal_sysrtc_wrap`, `hal_sysrtc_wide`, `hal_timer_ticks` |
| `TIMER0` | count, compare, PWM | `hal_timer_compare`, `hal_timer0_pwm` |
| `TIMER1`–`TIMER4` | the same driver on each of the other four instances, one claim each | `hal_timer_instances` — overflows counted against a 50 ms SYSRTC window: `o1=48 o2=48 o3=48 o4=0 stopped=0 top=18999 clean=1 refused=1`, three boots identical; the stopped instance and the never-started one are the controls |
| `USART0` | asynchronous serial, echo, a receiver that refuses a broken frame | `hal_usart_link` |
| `EUSART1` | the same asynchronous driver on the second block, with the loopback closing through the pad | `hal_eusart1_loopback` — `eusart1: n=8 matched=8 seen=8 cfg0=0x00000002 st=0x00003063 control=0 div=297 routed=1 refused=1`, three boots identical; the control run (loopback off) reads zero, and an unrouted block reads zero too |
| `EUSART0` | SPI **master**, 8-bit, blocking; asynchronous serial with the block's own divider | `hal_spi_loopback` (jumper on D9–D10, plus the control that reads `ff..ff` without it), `hal_eusart_loopback` (internal loopback and a control with it off) |
| `I2C0` | I2C **master**: blocking, async, repeated start, NACK, recovery | `hal_i2c_hardware_imu`, `hal_i2c_async`, `hal_i2c_recovery` |
| `I2C1` | the second bus: same driver, **compiled and reachable through `Board::i2c1`, never exercised** -- nothing is wired to it on this board | -- |
| `IADC0` | single-ended conversion, microphone path | `hal_adc_read`, `hal_mic_level` |
| `LDMA` | channel ownership, transfers, `stop()`, chained descriptors | `hal_dma_stop`, `hal_dma_link` |
| `MVP` | matrix/vector co-processor on `binary16` | `hal_mvp_bench` |
| `WDOG0` | windowed watchdog with a feed budget | `hal_wdog_feed` |
| `GPCRC` | CRC-32 (reflected, the caller adds the final xor) | `hal_gpcrc` |
| `LETIMER0` | 24-bit counter, two compare channels, `TOP`; runs in EM2 | `hal_letimer_compare` |
| `PCNT0` | pulse counting from a GPIO producer; the ceiling is pulse width, not bandwidth | `hal_pcnt_gpio` |
| `PRS` | producers measured: a GPIO pin, a timer overflow, a software pulse | `hal_adc_prs_gpio`, `hal_adc_prs_timer`, `hal_adc_prs_trigger` |
| `BURTC` | 32-bit backup counter, power-of-two prescaler, `EM4GRPACLK` source; survives a core reset | `hal_burtc_backup` |
| `CMU` clock sources | `FSRCO`, `HFRCOEM23` (band read back from `CAL`), `ULFRCO` and `LFRCO` measured against the crystal | `hal_cmu_sources` |
| `ACMP0` | internal input mux and the 63-tap reference ladder, read as a ruler | `hal_acmp_ladder` |
| `RADIOAES` | AES-128 ECB, one block per call, through scatter-gather descriptor chains; the radio itself is not touched | `hal_aes_kat` (FIPS-197 C.1 and NIST SP 800-38A F.1.1, plus a decrypt round trip) |

### Compiled, not measured

Nothing **except `I2C1`**, which is handed out by `Board::i2c1` and has no record of a single transaction: nothing is wired to that bus on this board. Every other instance the crate hands out has a record on the part; the modes that are absent are named in section 3
of `docs/04-reference/COVERAGE-BOUNDARY-2026-11-25.md` instead of living here as a maybe.


`TIMER1`–`TIMER4` were on this line until 2026-11-27, when building the four of them turned up two defects in a
constructor nothing had ever called: the `EN`/`CFG` order (a bus fault on every call) and `clear_overflow` clearing
`TIMER0` from every instance. Record:
[`docs/04-reference/TIMER-INSTANCES-2026-11-27.md`](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/04-reference/TIMER-INSTANCES-2026-11-27.md).

### Absent: peripheral families this crate does not touch

`VDAC0`/`VDAC1` (DACs) · `ACMP1` (the second comparator) · `KEYSCAN` (keyboard matrix) · `DCDC` · `SMU` · `WDOG1` · and the infrastructure set `SYSCFG`, `DMEM`,
`BURAM`, `SCRATCHPAD`, `HOSTMAILBOX`, `ICACHE0` (`MSC` is *not* here: `Board::take` unlocks it and writes `READCTRL` for
the flash wait states).

Clock sources are selected through `CMU`: `clock` owns the tree the core runs on, `clock::sources` owns `FSRCO`,
`HFRCOEM23` and the datasheet limits they are judged against, and `burtc` selects `ULFRCO` for its own counter. None of
them has a driver of its own. The radio is a different crate: [`efr32mg24-radio`](https://crates.io/crates/efr32mg24-radio).

### Absent: modes of the peripherals that are here

| Peripheral | Missing mode |
|---|---|
| `EUSART0`, `USART0` | **SPI slave** — the driver is master only: it owns the clock and the chip select |
| `I2C0`, `I2C1` | **I2C slave** (address match, clock stretching as a target) |
| `USART0`, `EUSART1` | 9-bit / multiprocessor address mode; IrDA; ISO7816 |
| `EUSART0` | **I2S** — the audio mode of the same peripheral |
| `IADC0` | differential input — refused with an error rather than read as single-ended; and scan (several channels per trigger) |
| `TIMER1`–`TIMER4` | **PWM**, and input **capture** on any of the five | compare and counting are driven and measured on all five (`hal_timer_instances`); **PWM is `TIMER0` only** — `pwm::Timer0` is the crate's single PWM path, so a reader who needs a second PWM channel gets TIMER0's three and not a fourth timer. Named here rather than left to be discovered |
| `EUSART0` | a real device on the pads — the loopback proof uses a jumper, never a second chip |

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
efr32mg2x-hal = { version = "0.1.0", features = ["rt"] }
cortex-m = "0.7"
cortex-m-rt = "0.7"
panic-halt = "0.2"
```

### Minimal Example

```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg2x_hal::{
    clock::{Clocks, ClockConfig, HfxoConfig, LfxoConfig},
    delay::Delay,
    pac,
    prelude::*,
};
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::pin::Port;
use panic_halt as _;

#[entry]
fn main() -> ! {
    // `Board` takes the peripherals once and hands out the pieces: the clocks, the pins, the delay.
    let mut board = Board::take().expect("peripherals taken twice");
    let mut delay = board.delay();

    // PA7, active low: the LED the board ships with.
    let led = board.output_on(Port::A, 7, true);

    loop {
        led.on();
        delay.delay_ms(500);
        led.off();
        delay.delay_ms(500);
    }
}
```

## Documentation

### Getting Started

- **[docs/](docs/)** - Complete HAL documentation index
- **Build system and toolchain** — in the repository: [`docs/02-guides/HAL-BUILD-SYSTEM.md`](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/02-guides/HAL-BUILD-SYSTEM.md)
- **[docs/LINKER_SETUP.md](docs/LINKER_SETUP.md)** - Linker script configuration

### Module Documentation

- **[src/clock/](src/clock/)** - Clock Management Unit (CMU)
- **[src/pin/](src/pin.rs)** - GPIO digital I/O
- **[src/delay.rs](src/delay/)** - SysTick-based delays

### API Documentation

Generate complete API documentation:

```bash
cargo doc --no-deps --features rt --open
```

## Examples

43 examples, **32 of them watched on the part**. A `--` in the last column means nobody
has read a value back off the hardware for it; it does not mean the code is unverified and it does not
mean it works.

| example | what it shows | watched on hardware |
|---|---|---|
| `hal_acmp_ladder.rs` | `ACMP0` as a ruler: the internal ladder swept against a fixed internal source, read three times. | 2026-11-27 |
| `hal_adc_prs_gpio.rs` | `Source::GpioPin(n)` was an encoding nothing drove: the two producers measured before it were the CPU's own pulse | 2026-11-26 |
| `hal_adc_prs_timer.rs` | The PRS trigger measured on 2026-11-26 was driven by `ASYNC_SWPULSE` — a CPU write. That proves the converter's | 2026-11-26 |
| `hal_adc_prs_trigger.rs` | An ADC that only converts when the CPU asks it to is an ADC the CPU has to babysit. This part can start a single | 2026-11-26 |
| `hal_adc_read.rs` | The IADC on this part converts one channel at a time into a 12-bit code. The hard part of an ADC | 2026-11-25 |
| `hal_aes_kat.rs` | `RADIOAES` against the two published AES-128 known answers, plus the controls that make them mean something. | 2026-11-27 |
| `hal_burtc_backup.rs` | `BURTC` against `SYSRTC` over a known window: two counters on the same 32.768 kHz branch, and a control that takes the | 2026-11-27 |
| `hal_cmu_sources.rs` | The clocks nobody had measured: `FSRCO`, `HFRCOEM23`, `ULFRCO` -- and the crystal as the referee. | 2026-11-27 |
| `hal_delay_clock.rs` | The delay, measured against a clock that is not the delay. | 2026-11-27 |
| `hal_dma_irq.rs` | Does the LDMA end a transfer with an interrupt, with the core asleep, and does `IEN` gate it? | 2026-11-27 |
| `hal_dma_link.rs` | Do the LDMA's linked descriptors actually chain, and does the second leg move only because of the link? | 2026-11-27 |
| `hal_dma_stop.rs` | Does `stop()` really disable the DMA channel? | 2026-11-23 |
| `hal_em2_sleep.rs` | EM2: dez sonecas de um segundo, com o SYSRTC a acordar. | -- |
| `hal_emu_voltage.rs` | The EMU's voltage scale, changed and put back, reported rather than assumed. | 2026-11-24 |
| `hal_eusart1_loopback.rs` | EUSART1 in asynchronous mode: eight bytes out and back through `CFG0.LOOPBK`, and a control that reads none. | 2026-11-27 |
| `hal_eusart_loopback.rs` | EUSART0 in asynchronous mode: eight distinct bytes out and back through `CFG0.LOOPBK`, and a control that reads | 2026-11-27 |
| `hal_gpcrc.rs` | Is the CRC accelerator right? Check it against a number computed elsewhere. | -- |
| `hal_gpio_interrupt.rs` | The pad moved by its own pull resistor, `IEN` and the NVIC armed, and a counter only the handler increments. | 2026-11-27 |
| `hal_hfxo.rs` | HFXO: run from the crystal, and prove the rest of the system followed it. | 2026-11-25 |
| `hal_i2c_async.rs` | O IMU lido pelo caminho **assíncrono** do I2C, num executor mínimo. | -- |
| `hal_i2c_bitbang.rs` | The bit-banged transport: scan whatever two pins you give it. | 2026-11-15 |
| `hal_i2c_hardware_imu.rs` | The IMU on the hardware I2C bus: routed pins, a power gate, one byte back. | -- |
| `hal_i2c_recovery.rs` | After a NACK, does the bus still work? | 2026-11-23 |
| `hal_imu_read.rs` | The IMU at the HAL's level: two pins, a power gate and a driver. | 2026-11-24 |
| `hal_led_blink.rs` | Blink at the HAL's level: one pin primitive, no board. | 2026-11-15 |
| `hal_letimer_compare.rs` | Does `LETIMER0` hold the top it was given, and does a compare below it fire twice? | 2026-11-26 |
| `hal_mic_level.rs` | Shout at the board and the light gets brighter — with hardware PWM. | 2026-11-24 |
| `hal_mvp_bench.rs` | Does the MVP pay for itself? One number, measured both ways — with a clock that works without a probe. | 2026-11-25 |
| `hal_one_of_each.rs` | One of each: the board refuses to hand the same peripheral out twice. | -- |
| `hal_pcnt_gpio.rs` | The PRS producer measured on 2026-11-26 carries a pad's level, and the IADC converts once per rising edge of it. | 2026-11-26 |
| `hal_rtt_hello.rs` | Deferred-formatting logging over RTT, and a panic that says what happened. | 2026-11-15 |
| `hal_spi_loopback.rs` | SPI with a wire between MOSI and MISO: eight bytes out, the same eight bytes back. | 2026-11-25 |
| `hal_spi_masters.rs` | All three SPI masters on the same pads, one after the other, and each with its own control. | -- |
| `hal_sysrtc_wide.rs` | O tick de 64 bits do SYSRTC, o comparador a acordar o núcleo, e um relógio independente a confirmá-lo. | -- |
| `hal_sysrtc_wrap.rs` | A volta do contador de 32 bits, exercida em milissegundos em vez de 36,4 horas. | -- |
| `hal_timer0_pwm.rs` | Buzzer through the HAL: a calibrated tone on the base's piezo. | -- |
| `hal_timer0_repacing.rs` | Pace TIMER0 twice, and a third time: does the second write to `CFG` still land? | -- |
| `hal_timer_compare.rs` | Does the compare channel fire? Measured against the counter itself, with a wait long enough to matter. | 2026-11-24 |
| `hal_timer_instances.rs` | Do the other four timer instances actually count? `TIMER1`–`TIMER4` were "compiled, not measured" for months, and | 2026-11-27 |
| `hal_timer_ticks.rs` | The tick timer, measured rather than assumed. | 2026-11-24 |
| `hal_usart_link.rs` | USART on the header pads: a jumper D6–D7 for the on-board proof, or an adapter for an outside witness. | 2026-11-25 |
| `hal_usart_rx_recovery.rs` | Does the receiver recover after an overrun? It is the property a product depends on. | -- |
| `hal_wdog_feed.rs` | The watchdog, fed on purpose, and asked whether it took the lock. | 2026-11-24 |


## Building

### Prerequisites

```bash
# Install ARM Cortex-M33 target
rustup target add thumbv8m.main-none-eabihf

# Install probe-rs for flashing (optional)
cargo install probe-rs-tools
```

### Build Commands

```bash
# Check code
cargo check --features rt

# Build library
cargo build --features rt --release

# Build examples
cargo build --examples --features rt

# Run tests
cargo test --features rt
```

## Features

- **Safe abstractions**: Zero unsafe code in public API
- **Type-safe GPIO**: Pin modes enforced at compile time
- **embedded-hal v1.0**: Compatible with embedded-hal ecosystem
- **Hardware register access**: Direct manipulation of EFR32MG24 registers
- **Critical sections**: Safe atomic operations via cortex-m
- **Modular design**: Clean separation of concerns, all files <400 lines

## Where the state lives

The status groups at the top of this file are the status; there is no second one to drift away from it. The
workspace-wide authority is [`docs/03-development/STATUS.md`](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/03-development/STATUS.md) in the project repository.

## Staying out of the bootloader

The first 24 KiB of flash (0x08000000-0x08005FFF) hold the Gecko bootloader. An image that lands there bricks the
board. The linker script in this crate starts at **0x08006000**, and the flags are not optional either:

```toml
[target.thumbv8m.main-none-eabihf]
rustflags = ["-C", "link-arg=-Tlink.x", "-C", "link-arg=--nmagic"]
```

Without `--nmagic` the linker emits a small segment at 0x08000000 and some flash tools will happily write it. Read the
lowest load address of the image before flashing and refuse anything below 0x08006000.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://www.apache.org/licenses/LICENSE-2.0.txt))
- MIT license ([LICENSE-MIT](https://opensource.org/license/mit))

at your option.

---

**Version**: 0.1.0
**Last Updated**: November 14, 2026
**Status**: 21 modules in the tree; the measured/unmeasured split is at the top of this file