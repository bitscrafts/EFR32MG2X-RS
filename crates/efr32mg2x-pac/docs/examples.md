## Examples

Every file here compiles for the target in CI. The date in the last column is the day the example was watched running
on the board itself, not merely built -- an example nobody has ever run is an example that does not work.

| example | what it shows | run on hardware |
|---|---|---|
| `pac_bme680` | PAC-only BME680/BME688 read on the Expansion Base I2C bus (0x76 / 0x77) | -- |
| `pac_button_read` | PAC-only read of the Expansion Base user button on PC01 (XIAO pin D1) | -- |
| `pac_buzzer_pwm` | PAC-only buzzer drive on the Expansion Base: TIMER0 PWM on PC03 (D3) | -- |
| `pac_env_display` | PAC-only environment display: BME680 (or BME688) readings on the | -- |
| `pac_i2c_bitbang_probe` | Bit-banged I2C on PC04 (SDA) / PC05 (SCL), for the Expansion Base devices | -- |
| `pac_i2c_probe` | PAC-only I2C0 probe of the Expansion Base devices, following the vendor | -- |
| `pac_i2c_scan` | PAC-only I2C bus scan on I2C0, pins PC04 (SDA) / PC05 (SCL) | -- |
| `pac_imu_capture` | Capture real IMU data into RAM, so a classifier can be trained on what the | -- |
| `pac_imu_probe` | Find the IMU by trying pin pairs and both SDA/SCL orders | -- |
| `pac_imu_read` | PAC-level IMU: the on-board LSM6DS3TR-C at I2C address 0x6A | -- |
| `pac_led_blink` | PAC-level LED blink for the XIAO MG24 Sense (EFR32MG24B220F1536IM48-B) | 2026-06-29 |
| `pac_led_blink_base` | PAC-only blink, baseline: one ON/OFF pulse group, 1_000_000 NOP iterations | -- |
| `pac_led_blink_double` | PAC-only DOUBLE blink: `blink blink ______ blink blink ______ ...` | -- |
| `pac_led_blink_timer` | PAC-only blink on a hardware timer: TIMER0 clocked from HFRCOEM23 | -- |
| `pac_mic_level` | PAC-level microphone → LED: the Arduino "VU meter" demo, at register level | -- |
| `pac_minimal` | The smallest program that runs on the board, from the PAC alone | -- |
| `pac_mvp_add` | MVP: the on-chip matrix/vector processor adds two fp16 vectors, from the PAC | -- |
| `pac_oled_hello` | PAC-only demo on the Expansion Base OLED: SSD1306 128x64 at 0x3C | -- |
| `pac_pullup_probe` | Which pins have an external pull-up? A measurement, not a datasheet reading | -- |
| `pac_rtc_read` | PAC-only read of the Expansion Base RTC (PCF8563 at 0x51) | -- |
| `pac_sysrtc_absolute_rate` | What rate does SYSRTC actually count at? Measured in wall-clock time, not against another chip clock | -- |
| `pac_sysrtc_compare_flag` | One question, one short answer: with the counter provably running, does the compare flag set? | -- |
| `pac_sysrtc_dsb` | Does the bus stall belong to SYSRTC, or to a pending SYSRTC write? | -- |
| `pac_sysrtc_gates` | Does the SYSRTC clock gate live in the secure CMU view, the non-secure one, or both? | -- |
| `pac_sysrtc_group0_irq` | Which group raises which interrupt line? | -- |
| `pac_sysrtc_probe` | Which SYSRTC write faults, in the vendor's own order | -- |
| `pac_sysrtc_rate` | Two questions, one flash: is GRP0_CTRL really at 0x68, and is 0x240 really the clock select? | -- |
| `pac_sysrtc_raw` | SYSRTC through raw addresses, with a fault handler that reports instead of hanging | -- |

### One of them, in full: `pac_led_blink_base`

```rust,ignore
//! PAC-only blink, baseline: one ON/OFF pulse group, 1_000_000 NOP iterations
//! per phase, no gap -- i.e. the continuous blink the board has been running.
//!
//! Compare against `pac_blink_double`. Verified over SWD, not by eye:
//! `scripts/read-led.sh` (mode7=4, DOUT toggles, DIN follows) and
//! `scripts/measure-blink-rate.sh` (toggle counter read from the mailbox twice,
//! divided by the wall-clock gap).

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

#[path = "common/blink.rs"]
mod blink;

const PULSES: u32 = 1;
const PULSE_ITERS: u32 = 1_000_000;
const GAP_ITERS: u32 = 0;

#[entry]
fn main() -> ! {
    blink::run_pattern(PULSES, PULSE_ITERS, GAP_ITERS)
}
```
