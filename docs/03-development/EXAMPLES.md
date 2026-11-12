# Example catalogue

Every example in the workspace, grouped by the crate whose level it demonstrates. **The crate a
file lives in tells you the smallest hardware it runs on**, and the name says which level it
belongs to.

## Naming convention

`<level>_<subject>[_<qualifier>]`, lower snake case, no numeric prefixes:

| prefix | crate | what it means |
|---|---|---|
| `pac_` | `efr32mg24-pac` | register-level proof of concept; pins are named inline on purpose |
| `hal_` | `efr32mg24-hal` | one HAL module and the pins it is given |
| `module_` | `xiao-mg24-bsp` | the XIAO module on the desk, no carrier underneath |
| `base_` | `xiao-expansion-base-bsp` | a device that is on the Expansion Base |
| — | `libraries/*` | an example that runs on the host, because the crate has no hardware |

The PAC and HAL examples name pins inline. That is the antipattern the layering exists to
prevent, and it is deliberate at those two levels: the point of a `pac_` example is the
registers, and the point of a `hal_` example is the driver. The wiring that must not be
repeated lives in the two BSP crates, once.

## `efr32mg24-pac` — chip level (19)

`cortex-m-rt` + `efr32mg24-pac` + `panic_halt`, nothing else. Several exist because a
HAL-level failure needed a second opinion at the lowest level.

| example | what it proves |
|---|---|
| `pac_led_blink`, `pac_led_blink_base`, `pac_led_blink_double`, `pac_led_blink_timer` | GPIO output, then a timer-paced blink; the rate is measured |
| `pac_i2c_scan`, `pac_i2c_probe`, `pac_i2c_bitbang_probe` | the buses, and that the on-chip I2C0 cannot read after a write |
| `pac_pullup_probe`, `pac_imu_probe` | external pull-ups on PB2/PB3; the 8×2×2 pin/order/address matrix that found the IMU |
| `pac_button_read` | input with pull-up |
| `pac_oled_hello`, `pac_env_display`, `pac_rtc_read`, `pac_bme680` | the three base devices, driven by registers alone |
| `pac_buzzer_pwm`, `pac_mic_level` | TIMER0 tone; IADC0 single conversions |
| `pac_imu_read`, `pac_imu_capture`, `pac_mvp_add` | the IMU at register level; a recording at 400 Hz; the MVP accelerator (does not execute — see `MVP-NOTES`) |

## `efr32mg24-hal` — driver level (2)

The HAL's own modules, with the pins named by the caller. Nothing here needs the base.

| example | what it proves |
|---|---|
| `hal_i2c_bitbang` | the bit-banged transport on a pair of pins the caller chooses, including the acknowledge that makes an empty bus visible |
| `hal_timer0_pwm` | a TIMER0 compare channel on a named pin, calibrated against the CPU clock (the 3× `asm::delay` trap is in its header) |

## `xiao-mg24-bsp` — the module (4)

Runs with the XIAO alone. The IMU and microphone exist only on the **Sense** variant, so those
two are gated by `--features sense` in the manifest — a plain MG24 module skips them cleanly.

| example | needs | what it proves |
|---|---|---|
| `module_led_blink` | any MG24 | `Module::take()` and one LED, with no port or address in the file |
| `module_imu` | Sense | PD5 gate, bus on PB2/PB3, `WHO_AM_I = 0x6A`, and `|a| ≈ 1 g` as the invariant |
| `module_imu_regs` | Sense | the raw register bytes, so a scale-factor mistake is visible as a number |
| `module_mic_level` | Sense | PC8 enable, IADC0 window, RMS and peak, LED following the level |

## `xiao-expansion-base-bsp` — the carrier (7 + 1 project)

Needs the base; the IMU ones also need the Sense module.

| example | needs | what it proves |
|---|---|---|
| `base_env_clock` | module + base | the facade end to end: OLED, BME68x, PCF8563 — no address, pin or register in the file |
| `base_env_display` | module + base | temperature, pressure and humidity on the panel |
| `base_oled_hello` | module + base | text through the display driver |
| `base_rtc` | module + base | PCF8563 keeps time across a reset, and says where the time came from |
| `base_imu` | module Sense + base | the IMU read and shown, with the base's panel |
| `base_imu_monitor` | module Sense + base | live axes and peak on the panel, with a 10 s ring buffer |
| `base_buzzer_button` | module + base | TIMER0 CC0 tone and the PC01 button, with the LED answering both |
| `base_fall_detector/` (`main`, `app`, `ui`, `recording`, `recorded`, `falls`, `falls_400hz`) | module Sense + base | the classifier calibrated on nine real drops, recording while it classifies, alarming on a fall |

## `libraries/motion-detection` — no hardware (1 example, 9 tests)

| example | what it proves |
|---|---|
| `replay` | a recording through the same code path the firmware uses: 174 ms of free fall and a 9.3 g landing produce exactly one fall |

`cargo run --example replay -p motion-detection`.

## The rule the gate enforces

A `*-bsp` crate that exposes hardware with zero examples fails the gate: a crate whose API
nothing builds is a crate whose API is not tested. Every example that lives in a directory must
also be declared in its manifest, or `--examples` silently skips it.

```bash
bash scripts/gate.sh    # VERDICT: PASS — pac 19, hal 2, module 4, base 7 (+1), motion-detection 1
```

## Bluetooth, at every level where it can exist

| Level | Examples | What it proves | Why not here |
|---|---|---|---|
| `efr32mg24-pac` | — | — | the radio is a blob; there are no registers to write |
| `efr32mg24-hal` | — | — | a HAL that depended on MSLA blobs would tax every GPIO user |
| `efr32mg24-bluetooth-sys` | `bt_version` | the bindings, the blobs and the CMSIS stubs link | — |
| `efr32mg24-radio` | `ble_beacon`, `ble_advertise_custom`, `ble_scanner` | the stack, both directions, and a framed payload | `ble_gatt` waits on PSA crypto |
| `xiao-mg24-bsp` | `module_ble_beacon` | a module user adds BLE with one dev-dependency | — |
| `module_ble_motion` | `xiao-mg24-bsp` | the IMU on the air: 10-byte frame, 10 Hz, no connection |
| `module_ble_fall` | `xiao-mg24-bsp` | the calibrated classifier's verdict broadcast: flags, falls, peak, free-fall |
| `xiao-expansion-base-bsp` | `base_ble_env` | the product shape: sensor reading in the advertising payload | — |

The two blanks are the interesting entries. PAC and HAL have no Bluetooth examples **by design**, and
saying so is part of the catalogue: the radio is not a peripheral you program, it is a stack you link.

## One rule per rung

An example names **only its own crate**:

```
pac_*     may name efr32mg24_pac        and nothing above it
hal_*     may name efr32mg24_hal        never the PAC, never a board
module_*  may name xiao_mg24_bsp        (its `hal::` re-export included)
base_*    may name xiao_expansion_base_bsp  (its `hal::` and `module::` re-exports included)
```

Both directions are checked by `scripts/gate.sh`: no PAC type outside the PAC crate, and no
`efr32mg24_hal::` inside a BSP example. The second check was added because fourteen BSP examples named
the HAL directly — an example of the layer below, living in the wrong crate.
