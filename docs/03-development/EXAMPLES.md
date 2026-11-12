# Example catalogue

Every example in the workspace, grouped by the crate whose level it demonstrates, with the
hardware it needs. The grouping is not cosmetic: **the crate a file lives in tells you the
smallest board it runs on.**

```
efr32mg24-pac              chip level    any pin is legitimate; the point is the registers
efr32mg24-hal              driver level  a driver and the pins it is given, named inline
xiao-mg24-bsp              module        the XIAO on the desk, no base underneath it
xiao-expansion-base-bsp    module + base the carrier's own devices
libraries/                 no hardware   runs on the host
```

The PAC and HAL examples name pins inline on purpose. They are proof of concept at their own
level — the antipattern is the point, and it is what makes them readable. The wiring that must
not be repeated lives in the two BSP crates, once.

## `efr32mg24-pac` — chip level (19 examples)

No HAL, no BSP: `cortex-m-rt` + `efr32mg24-pac` + `panic_halt` and nothing else. Each one
proves a register sequence against the die, and several of them exist because a HAL-level
failure needed a second opinion at the lowest level.

| example | what it proves |
|---|---|
| `blink_xiao`, `pac_blink_base`, `pac_blink_double`, `pac_timer_blink` | GPIO output, then a timer-paced blink; `pac_timer_blink` measures the toggle rate |
| `pac_i2c_scan`, `pac_i2c_probe`, `pac_i2c_bitbang_probe` | the buses, and that the on-chip I2C0 cannot read after a write (see the defect doc) |
| `pac_pullup_probe`, `pac_imu_probe` | external pull-ups on PB2/PB3; the 8×2×2 pin/order/address matrix that found the IMU |
| `pac_gpio_button` | input with pull-up |
| `pac_oled_hello`, `pac_env_display`, `pac_rtc_read`, `pac_bme680` | the three base devices, driven by registers alone |
| `pac_buzzer_pwm`, `pac_mic_vu` | TIMER0 tone; IADC0 single conversions |
| `pac_imu`, `pac_imu_capture`, `pac_mvp_add` | the IMU at register level; a 400 Hz recording; the MVP accelerator (does not execute — see `MVP-NOTES`) |

## `efr32mg24-hal` — driver level (10 examples)

Each drives one HAL module and states its pins. `Bus`, `Port` and the pin primitives are
parameters, so the same driver serves the module and the base.

| example | what it proves |
|---|---|
| `hal_bus_scan` | the bit-banged transport on the base's pins: three devices answer |
| `hal_oled_hello`, `hal_env_display` | the display driver and the BME68x driver with compensation |
| `hal_rtc`, `hal_buzzer`, `hal_button` | PCF8563; TIMER0 PWM on a named pin; input with pull-up |
| `hal_imu`, `hal_imu_regs`, `hal_imu_monitor` | the LSM6DS3TR-C driver, raw registers, and a live monitor |
| `hal_mic_vu` | the IADC0 microphone driver with 128-sample windows |

## `xiao-mg24-bsp` — the module (10 examples)

Runs with the XIAO alone. The two `sense_*` examples need `--features sense`, and the manifest
says so, so a plain MG24 module skips them instead of failing.

| example | needs | what it proves |
|---|---|---|
| `00_blink` | any MG24 | `Module::take()` and one LED, with no port or address in the file |
| `sense_imu` | Sense | PD5 gate, bus on PB2/PB3, `WHO_AM_I = 0x6A`, and `|a| ≈ 1 g` |
| `sense_microphone` | Sense | PC8 enable, IADC0 window, RMS and peak, LED following the level |

## `xiao-expansion-base-bsp` — the carrier (2 examples + 1 project)

Needs the base, and the Sense module for the IMU.

| example | needs | what it proves |
|---|---|---|
| `base_env_clock` | module + base | the facade end to end: OLED, BME68x, PCF8563 — no address, pin or register in the file |
| `base_buzzer_button` | module + base | buzzer on TIMER0 CC0 and button on PC01, with the LED answering both |
| `fall_detector/` (`main`, `app`, `ui`, `recording`, `recorded`, `falls`, `falls_400hz`) | module Sense + base | the classifier calibrated on nine real drops, recording while it classifies, alarming on a fall |

## `libraries/motion-detection` — no hardware (1 example, 9 tests)

| example | what it proves |
|---|---|
| `replay` | a recording is fed through the same code path the firmware uses, on a laptop: 174 ms of free fall and a 9.3 g landing produce exactly one fall |

Run with `cargo run --example replay -p motion-detection`.

## Why the counts matter

The gate fails a `*-bsp` crate that exposes hardware with zero examples, because a crate whose
API nothing builds is a crate whose API is not tested. That rule is what keeps `sense_imu` and
`base_buzzer_button` from disappearing in a future refactor — and it is checked, not promised:

```bash
bash scripts/gate.sh    # VERDICT: PASS
```
