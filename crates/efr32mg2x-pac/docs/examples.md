## Examples

Every file here compiles for the target in CI. The date in the last column is the day the example was watched running
on the board itself, not merely built -- an example nobody has ever run is an example that does not work.

| example | what it shows | run on hardware |
|---|---|---|
| `pac_bme680` | PAC-only BME680/BME688 read on the Expansion Base I2C bus (0x76 / 0x77) | 2026-11-15 |
| `pac_button_read` | PAC-only read of the Expansion Base user button on PC01 (XIAO pin D1) | 2026-11-15 |
| `pac_buzzer_pwm` | PAC-only buzzer drive on the Expansion Base: TIMER0 PWM on PC03 (D3) | 2026-11-15 |
| `pac_env_display` | PAC-only environment display: BME680 (or BME688) readings on the | 2026-11-15 |
| `pac_i2c_bitbang_probe` | Bit-banged I2C on PC04 (SDA) / PC05 (SCL), for the Expansion Base devices | 2026-11-15 |
| `pac_i2c_probe` | PAC-only I2C0 probe of the Expansion Base devices, following the vendor | 2026-11-15 |
| `pac_i2c_scan` | PAC-only I2C bus scan on I2C0, pins PC04 (SDA) / PC05 (SCL) | 2026-11-15 |
| `pac_imu_capture` | Capture real IMU data into RAM, so a classifier can be trained on what the | 2026-11-15 |
| `pac_imu_probe` | Find the IMU by trying pin pairs and both SDA/SCL orders | 2026-11-15 |
| `pac_imu_read` | PAC-level IMU: the on-board LSM6DS3TR-C at I2C address 0x6A | 2026-11-15 |
| `pac_led_blink` | PAC-level LED blink for the XIAO MG24 Sense (EFR32MG24B220F1536IM48-B) | 2026-11-15 |
| `pac_led_blink_base` | PAC-only blink, baseline: one ON/OFF pulse group, 1_000_000 NOP iterations | 2026-11-15 |
| `pac_led_blink_double` | PAC-only DOUBLE blink: `blink blink ______ blink blink ______ ...` | 2026-11-15 |
| `pac_led_blink_timer` | PAC-only blink on a hardware timer: TIMER0 clocked from HFRCOEM23 | 2026-11-15 |
| `pac_mic_level` | PAC-level microphone → LED: the Arduino "VU meter" demo, at register level | 2026-11-15 |
| `pac_minimal` | The smallest program that runs on the board, from the PAC alone | -- |
| `pac_mvp_add` | MVP: the on-chip matrix/vector processor adds two fp16 vectors, from the PAC | 2026-11-15 |
| `pac_oled_hello` | PAC-only demo on the Expansion Base OLED: SSD1306 128x64 at 0x3C | 2026-11-15 |
| `pac_pullup_probe` | Which pins have an external pull-up? A measurement, not a datasheet reading | 2026-11-15 |
| `pac_rtc_read` | PAC-only read of the Expansion Base RTC (PCF8563 at 0x51) | 2026-11-15 |
| `pac_sysrtc_absolute_rate` | What rate does SYSRTC actually count at? Measured in wall-clock time, not against another chip clock | 2026-11-15 |
| `pac_sysrtc_compare_flag` | One question, one short answer: with the counter provably running, does the compare flag set? | 2026-11-15 |
| `pac_sysrtc_dsb` | Does the bus stall belong to SYSRTC, or to a pending SYSRTC write? | 2026-11-15 |
| `pac_sysrtc_gates` | Does the SYSRTC clock gate live in the secure CMU view, the non-secure one, or both? | 2026-11-15 |
| `pac_sysrtc_group0_irq` | Which group raises which interrupt line? | 2026-11-15 |
| `pac_sysrtc_probe` | Which SYSRTC write faults, in the vendor's own order | 2026-11-15 |
| `pac_sysrtc_rate` | SYSRTC ticks per unit of time, under two clock selections, reported over RTT | 2026-11-15 |
| `pac_sysrtc_raw` | SYSRTC through raw addresses, with a fault handler that reports instead of hanging | 2026-11-15 |
| `pac_sysrtc_trace` | The SYSRTC sequence with a log line before and after every write, to see where it dies | 2026-11-15 |

### One of them, in full: `pac_led_blink_base`

```rust,ignore
//! PAC-only blink, baseline: one ON/OFF pulse group, 1_000_000 NOP iterations
//! per phase, no gap -- i.e. the continuous blink the board has been running.
//!
//! Compare against `pac_blink_double`. Verified over SWD, not by eye:
//! `scripts/read-led.sh` (mode7=4, DOUT toggles, DIN follows) and
//! `scripts/measure-blink-rate.sh` (toggle counter read from the mailbox twice,
//! divided by the wall-clock gap).
//! VERIFIED ON HARDWARE (2026-11-15)

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_probe as _;

#[path = "common/blink.rs"]
mod blink;

const PULSES: u32 = 1;
const PULSE_ITERS: u32 = 1_000_000;
const GAP_ITERS: u32 = 0;
use defmt_rtt as _;

#[entry]
fn main() -> ! {
    blink::run_pattern(PULSES, PULSE_ITERS, GAP_ITERS)
}
```
