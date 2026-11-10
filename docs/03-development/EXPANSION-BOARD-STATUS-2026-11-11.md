# Expansion Base — example status, 2026-11-11

Every row is a claim backed by a measurement taken on the board, or it says
plainly what is missing. Nothing here is "should work".

## Working, verified

| Example | What it proves | Evidence |
|---|---|---|
| `pac_blink_base` | PAC-only blink, NOP loop | LED seen; `mode7=4`, DIN follows DOUT |
| `pac_blink_double` | pattern requested by the operator | LED seen; 6.16 toggle/s measured |
| `pac_timer_blink` | TIMER0 at 1 Hz, independent of the boot path | 1.02 Hz measured twice; `TOP` calibrated to 9229 |
| `pac_gpio_button` | PC01 input with pull-up | `PORTC_MODEL=0x20` (mode 1, pull via DOUT), `DIN bit1=1` at rest |
| `pac_i2c_scan` / `pac_i2c_probe` | bus reaches the three devices | 0x3C, 0x51, 0x76 answer (bit-bang master) |
| `pac_oled_hello` | SSD1306 text | 298 bytes ACKed; **text seen by the operator** |
| `pac_rtc_read` | PCF8563 date/time | `2038-10-01 00:14:27` then `00:14:29`, `VL=1` |
| `pac_bme680` | Bosch compensation end to end | `T=31.9 C`, `P=1015 hPa` — matches a 30 C room |
| `pac_env_display` | sensor + panel together | `T=31.93C P=1015hPa H=68.4%`, **all three lines seen on the panel** |
| `pac_buzzer_pwm` | TIMER0 tone, self-calibrated | registers verified, `tone=2700Hz` by construction, beep counter climbing — **not yet audible to the operator** |

## Not done yet

| Target | Pin(s) | Notes for tomorrow |
|---|---|---|
| microSD | CS = D2/PC02, SPI: SCK D8/PA03, MISO D9/PA04, MOSI D10/PA05 | SPI master + SD CMD0/CMD8/ACMD41 init; report CID |
| Grove UART | TX D6/PC06, RX D7/PC07 (EUSART1) | needs a loopback jumper or a peer to verify; echo test |
| Grove A0 | D0/PC00 | ADC single conversion; a floating pin reads noise, so report the raw spread |
| Grove I2C (2nd port) | same bus as the panel | already covered by the scan |
| Servo header | 5 V, needs a PWM channel | TIMER0 CC1/CC2 in PWM mode |

## Things the hardware taught us (do not re-learn these)

1. **I2C0 will not complete a read after a write.** Evidence and every failed
   workaround: `docs/04-reference/I2C0-READ-DEFECT.md`. The examples use a
   bit-banged master built from PAC GPIO registers.
2. **BME68x calibration is not laid out from 0x8A in register order.**
   `par_t1` is coefficient index 31/32 (inside the block at 0xE1); index 0/1 is
   `par_t2`. Using 0x8A as `par_t1` reports -1 C on a 32 C bench.
   `par_h1`/`par_h2` share byte 24 (low and high nibble).
3. **The BME68x field block starts at 0x1D** (byte 0 = status, `new_data` in
   bit 7), not at 0x22.
4. **A BME68x mode change is only accepted from sleep.** Otherwise the write is
   ACKed and ignored, and `ctrl_meas` reads back as sleep.
5. **The SSD1306 must be driven in a single addressing mode.** Page commands are
   ignored in horizontal mode; the panel then paints noise while every byte is
   still ACKed.
6. **TIMER0 register access is state-dependent.** `CFG` and `CCx_CFG` require
   `EN=0`; `CCx_CTRL` and `TOP` require `EN=1`; a wrong-state write is a precise
   bus fault (`CFSR=0x8200`) with `BFAR` = the register.
7. **`CFG=0x00010003` selects the quadrature decoder** (MODE=3): `STATUS.RUNNING`
   reads 1, `CNT` stays at 0. Use MODE=Up.
8. **HFRCOEM23 comes up in a low band** (~5 MHz measured, not the 20 MHz of its
   name), and the timer clock must be measured, not assumed. Both the 1 Hz blink
   and the buzzer self-calibrate against a measured tick rate.
9. **`IF.CC0` does not clear by writing 1** on this part — counting it in a poll
   loop counts one stale event per iteration (36998 "events", 9249 "Hz"). Time
   with `CNT` differences instead.
10. **Halt/read/resume over SWD disturbs nothing** as long as the image is
    idempotent; the mailbox record with its magic/seq protocol is what makes
    autonomous verification possible.
