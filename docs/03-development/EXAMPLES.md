# Example catalogue

Every example in the workspace, what it proves, and the evidence that it does. A row
without evidence is a claim, not a result: the `status` column says which is which.

Measured on: XIAO MG24 Sense on the Seeed Expansion Base, read over SWD from the
mailbox at `0x2003_F000` (`scripts/read-mailbox.sh`) or from RAM.

## PAC — register level, no HAL, no drivers

| example | proves | evidence | status |
|---|---|---|---|
| `pac_blink_base` | GPIO is enough to blink | 43 toggles in 6.83 s | green |
| `pac_blink_double` | two-phase blink | 56 toggles in 9.10 s → 3.08 Hz | green |
| `pac_timer_blink` | TIMER0 periodic, calibrated to 1 Hz | `32 toggles in 15.70 s` → 1.02 Hz, `top=0x240d` | green |
| `pac_gpio_button` | PC01 input with pull-up | `PORTC_DIN=0x02` at rest | green |
| `pac_i2c_scan` | I2C0 master, pins PC04/PC05 | completed, 0 devices (before the mode fix) | stale |
| `pac_i2c_probe` | reading a register over I2C0 | reads fail after a write — the defect it documents | red (by design) |
| `pac_i2c_bitbang_probe` | bit-banged master works where I2C0 does not | `bme=00/0x61`, `rtc=00/0x17/0x19` | green |
| `pac_oled_hello` | SSD1306 init and text | `298 bytes ACKed`, text visible | green |
| `pac_rtc_read` | PCF8563 calendar read | `2038-10-01 00:14:27 → 00:14:29` | green |
| `pac_bme680` | Bosch compensation from raw calibration | `T=31.93C P=1015hPa H=68.4%` | green |
| `pac_env_display` | two devices on one bus, on screen | `T=31.93C P=1015hPa H=68.4%` seen on the panel | green |
| `pac_buzzer_pwm` | TIMER0 CC0 PWM on PC03 | `tone=2700Hz`, intermittent beeps | green (MCU side) |
| `pac_mic_vu` | IADC0 analogue microphone + LED level | `rms=562 pk=598 n=128` | green |
| `pac_pullup_probe` | finds bus pins by external pull-ups | `a=000002f6 b=0000000c c=00000034 d=00000020` | green |
| `pac_imu_probe` | finds the IMU's pins and line order | `sda=p1.2 scl=p1.3 a=6a id=6a` | green |
| `pac_imu` | LSM6DS3TR-C at 0x6A | `id=6a a=985,81,139mg` | green |
| `pac_imu_capture` | 10 s ring buffer at 100 Hz for training data | `cap base=0x20000000 n=1000 rate=100Hz` | green |
| `pac_mvp_add` | the matrix/vector accelerator, programmed by hand | `mvp sum exp=396 got=0` | red (open) |

`pac_i2c_scan` is marked stale on purpose: it predates the pin-mode fix (the bus needs
`WIREDAND`, not push-pull) and its zero-device result is not a finding about the board.

## HAL — drivers, transport abstraction, no registers in the example

| example | proves | evidence | status |
|---|---|---|---|
| `hal_bus_scan` | the base bus, through `I2cBus` | `3c 51 76` | green |
| `hal_oled_hello` | SSD1306 driver | `oled acked≈1090` | green |
| `hal_env_display` | BME68x driver | `T=31.68C P=1015hPa H=68.2%` (PAC twin: 31.93/1015/68.4) | green |
| `hal_button` | button + LED through the board layer | `button released at boot, LED follows` | green |
| `hal_mic_vu` | microphone driver + LED level | `rms=521 pk=544 n=128` | green |
| `hal_imu` | IMU driver on its own bus | `id=6a a=970,79,142` | green |
| `hal_imu_regs` | control registers and raw bytes | `c1=40 c2=40 c3=44 ac=6e 3e …` (= 975 mg) | green |
| `hal_imu_monitor` | 15 s ring at 200 Hz, live values on the panel | `mon base=0x20000000 n=3000 200Hz pm8g` | green |
| `hal_rtc` | PCF8563 driver, kept across resets | `rtc 2026-11-10 12:00:57 kept` | green |
| `hal_buzzer` | TIMER0 PWM through the driver | `tone=2700Hz top=3518` | green |

## Application — a project, not a single file

| example | proves | evidence | status |
|---|---|---|---|
| `fall_detector/` (`main`, `app`, `ui`, `recorded`) | a classifier over real recordings; free fall + impact + stillness | `fall selftest real=clean synth=detected` | green |

## BSP — the board by name

| example | proves | evidence | status |
|---|---|---|---|
| `bsp_env_clock` | the facade carries an app end to end | `bsp dev=111 T=31C P=1017hPa H=67` | green |

The older `00_*` … `06_*` BSP examples belong to the June tree and are not part of the
verified set; `06_microphone_streaming` in particular targets an API that no longer
exists.

## Rebuilding and flashing

```bash
# PAC example
cargo build --release -p efr32mg24-pac --example pac_imu --features rt

# HAL example (the `rt` feature provides the vector table)
cargo build --release -p efr32mg24-hal --features rt --example hal_imu

# the application project
cargo build --release -p efr32mg24-hal --features rt --example fall-detector

# BSP example
cargo build --release -p xiao-mg24-bsp --example bsp_env_clock

# flashing: dry run first, then --confirm; the script refuses anything that would
# touch the bootloader
ELF=/projects/_cache/targets/thumbv8m.main-none-eabihf/release/examples/pac_imu
python3 scripts/elf-load-range.py "$ELF"
bash scripts/flash-lnx.sh "$ELF" --confirm
bash scripts/read-mailbox.sh
```
