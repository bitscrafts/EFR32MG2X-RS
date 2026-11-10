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
