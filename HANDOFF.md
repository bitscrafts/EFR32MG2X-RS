# HANDOFF — EFR32MG2X-RS (XIAO MG24 Sense + Expansion Base)

**State: green, everything below verified on hardware.** Branch `master`, pushed to
`origin` (`bitscrafts/EFR32MG2X-RS`). Board left running `bsp_env_clock`.

## The ladder, and how far it is built

```
PAC  ██████████  drivers at register level, 15 examples, all measured
HAL  ████████░░  drivers for every base device + IMU + mic; examples per device
BSP  ███░░░░░░░  ExpansionBase facade works (bsp_env_clock); old modules pending retirement
RADIO ░░░░░░░░░  not started
```

## What runs today, and the evidence

| example | layer | evidence |
|---|---|---|
| `pac_env_display` / `hal_env_display` | PAC / HAL | `T=31.9C P=1015hPa H=68.3%` on the panel |
| `pac_bme680` | PAC | Bosch compensation, `par_t1=25975` |
| `pac_rtc_read` | PAC | real calendar read |
| `pac_oled_hello` / `hal_oled_hello` | PAC / HAL | text on the 128×64 panel |
| `pac_gpio_button` / `hal_button` | PAC / HAL | `PC01` pull-up, `DIN=1` at rest |
| `pac_buzzer_pwm` | PAC | `tone=2700Hz`, intermittent, measured |
| `pac_mic_vu` / `hal_mic_vu` | PAC / HAL | `rms≈550`, 128 samples, LED follows level |
| `pac_imu` / `hal_imu` | PAC / HAL | `id=6a a=970,79,142` (one g) |
| `pac_pullup_probe` | PAC | found the IMU bus by external pull-ups |
| `pac_imu_probe` | PAC | `sda=p1.2 scl=p1.3 a=6a id=6a` |
| `hal_rtc` | HAL | `rtc 2026-11-10 12:00:57 kept` (ticks across resets) |
| `fall-detector` | app | `fall selftest real=clean synth=detected` |
| `bsp_env_clock` | BSP | `bsp dev=111 T=31C P=1017hPa H=67` |

## Layering, as agreed

- **HAL = drivers.** A driver answers "what did the device say". Accelerometer range
  and ODR are driver concerns and are configurable; *deciding that 2.5 g is an
  impact* is not.
- **`motion-detection` = algorithm crate.** Hardware free, host tested, no
  dependencies. The fall detector lives here, not in the HAL: an earlier version put
  it in `hal::sensors::fall` and that was wrong.
- **BSP = this board.** Addresses, pins, buses, power gates, conflicts.
  `crates/xiao-mg24-bsp/src/expansion_base.rs`.
- **Examples are applications.** They compose; the dependency runs application →
  algorithm → driver, never back.

## Facts that cost time, do not relearn them

1. **The IMU is on PB2/PB3 (SDA=PB2, SCL=PB3), not on the base bus**, and the
   Arduino variant has the two lines swapped. `PD5` powers it. See
   `docs/04-reference/IMU-INVESTIGATION.md`.
2. **The microphone is analogue**: PC8 enables it, PC9 is the input, IADC0 samples
   it. `IADC.SINGLE` can only be written while the converter is **enabled**; writing
   it disabled faults and wedges the engine until a power cycle
   (`docs/04-reference/IADC-MIC-NOTES.md`).
3. **The on-chip I2C0 master cannot complete a read after a write.** Both buses are
   bit-banged (`docs/04-reference/I2C0-READ-DEFECT.md`).
4. **`format_fixed` takes value×100.** Passing milligrams raw prints the reading
   divided by 100 — it looks exactly like a dead sensor.
5. **The MVP is compiler-programmed**, not hand-programmable in practice: the
   reference manual removed its register chapter in Rev 1.1 and the SDK compiles
   `.tflite` into register programs. `pac_mvp_add` builds and flashes but does not
   execute yet; next step is a known-good SDK program as an oracle
   (`docs/04-reference/MVP-NOTES.md`).
6. **Flash only through `scripts/flash-lnx.sh`** (dry-run by default, refuses
   `load_min < 0x08006000`, verifies by read-back). The bootloader at
   `0x08000000` has been intact after every flash.

## Next, in order

1. Retire the June BSP modules (`board.rs`, `pins.rs`, radio/sensor modules) — they
   compile but were written against an older HAL API and are not what the facade
   uses.
2. microSD (SPI, chip select shares a pin — the facade is where that conflict should
   be refused), then Grove UART/ADC and the servo.
3. A real fall dataset: `hal_imu_monitor` records 15 s at 200 Hz and `pac_imu_capture`
   10 s at 100 Hz; the classifier needs several genuine drops, not shakes.
4. MVP: run a Silicon Labs sample as the oracle, dump the register block before
   `CMD`, diff against `pac_mvp_add`.
5. Radio last, as agreed.

## Fall detector — state at the end of 2026-11-11 (read this before touching it)

**On the board:** `fall-detector`, self tests `real=clean synth=ok 200Hz=2/2 400Hz=4/4`.
It classifies **and** records while it classifies, and it beeps three times on a fall.

**Proven with real drops:**

| round | what | result |
|---|---|---|
| 1 | 2 drops, 200 Hz | 2/2 |
| 2 | 5 drops, 200 Hz | 3/3 detectable (two lost their free fall between samples) |
| 3 | 4 drops, 400 Hz | 4/4 |
| 4 | 4 drops onto a bed, real timing | 4/4 — free fall 176–252 ms, landing 11.7–13.8 g, settle 1.04–1.17 s |
| control | handling recording | 0 false alarms **on a time base now known to be wrong** |

**Open, and it is the only thing between "promising" and "validated":** the shaking control
has to be re-run with the current firmware. The old one was measured while the loop claimed
400 Hz and ran at 158, and the round that did get recorded filled the log 22 times over
because every sample above 2 g was its own "impact". Both are fixed; the measurement is not
made yet.

Read it with `scripts/read-fall-log.sh` (dumps and decodes; addresses come from the ELF, not
from this file). Three counters decide it: `FALLS` must be 0 after shaking, and equal to the
number of drops after dropping.

**Facts that cost time in this session, not to be relearned:**

- The loop **cannot** be trusted to run at the rate it aims for. It measured **235 Hz** with a
  mean interval of 4249 µs and stalls of **63.75 ms** when the display flushes. Always pass
  the measured interval, never the intended one.
- An impact is **one event per episode**, reported when the bang ends with the peak it
  reached. One event per sample turned three seconds of shaking into 1430 log entries.
- A recording must carry its own timing (`recording::DTS`). The optimiser **deleted** that
  array until the writes became volatile — the recording would have claimed to be self-timed
  and carried nothing.
- The ring's unwritten slots read as zero, and a zero sample looks exactly like free fall.
  Analyse only `TOTAL` samples.
- A replug resets the board and loses the RAM log. A log that must survive belongs in BURAM.
