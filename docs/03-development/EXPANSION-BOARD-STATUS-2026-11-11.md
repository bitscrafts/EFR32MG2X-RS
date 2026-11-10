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
| `pac_buzzer_pwm` | TIMER0 tone, self-calibrated | registers verified (`CC0_CFG=0x02`, `TOP=0x2942`, `PORTC_MODEL=0x4000` = PC03 push-pull), beep counter climbing, `tone=2700Hz` by construction — **no sound heard by the operator** |

## Buzzer: no sound — the two candidates

Measured on the running image: `CC0_CFG=0x00000002` (Output Compare),
`TOP=0x00002942`, `PORTC_MODEL=0x00004000` (PC03 = push-pull), and
`CC0_CTRL=0x00000000` (`CMOA=None`) in the silence phase. `PORTC_DIN=0` for
PC03 in every sample.

So the MCU side is configured; what is not proven is that the *pin* moves and
that the *buzzer* is connected. Two candidates, in order of likelihood:

1. **The base's buzzer trace is cut or has no buzzer fitted.** The Seeed base
   documents a cuttable trace for the buzzer. Physical check.
2. **The routed peripheral output does not drive the pin in MODE 4.** Worth one
   test: program a ~2 Hz square wave on CC0 and sample `PORTC_DIN` bit 3 over
   several seconds. A 2.7 kHz tone is invisible to a ~1 ms SWD sample, which is
   why the current image cannot tell "pin moving" from "pin still".

That test is the first thing to run tomorrow; it separates the firmware from the
board in one flash.

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

## HAL ladder (same board, drivers instead of registers)

| Example | Status |
|---|---|
| `hal_bus_scan` | PASS — `3c 51 76`; powers PD5 first |
| `hal_oled_hello` | PASS — panel bytes ACKed, text composed through the driver |
| `hal_button` | PASS — "button released at boot, LED follows" |
| `hal_env_display` | PASS — `T=31.68C P=1015hPa H=68.2%`, matches the PAC twin within sensor noise |
| `hal_imu` | written, blocked: **no device at 0x6A/0x6B** on this module (see below) |

HAL pieces: `board::Board` (peripherals + clock gates), `board::SensorPower`
(PD5), `board::Button`, `board::Led`, `bus::I2cBus` + `BitBangI2c` + `SharedBus`,
`display::Ssd1306`, `sensors::Bme68x`, `sensors::Lsm6ds3trc`.

The fourteen pre-June HAL examples (`01_clock` … `12_wdog`) were retired: they no
longer compile (`09_dma_adc_continuous` uses registers the PAC does not have) and
none was ever verified on hardware. They are staged outside the repository, not
deleted.

## IMU question, answered by measurement

With PD5 high, a full scan of the bus gives `3c 51 76` and nothing at **0x6A** or
**0x6B**. The seeed wiki's sketch for the Sense module drives PD5 high before
touching the IMU, so the power gate is not the missing piece — the part simply is
not answering on this bus. Two explanations remain, and they need the module
identified rather than another guess:

1. this is the standard XIAO MG24 (the IMU and the microphone are Sense-only);
2. the IMU is wired to another bus/pins on the Sense layout.

One more measured detail worth keeping: scanning with PD5 **low** and raising it
afterwards leaves the bus dead for the rest of that run, even though both lines
read 0x30 (high) afterwards. Power the rail first, then talk.

## Microphone (Sense) — written, one power cycle from verified

Seeed's own MG24 backend answers the hardware question: the mic is **analog**,
on **PC9**, powered by **PC8**, sampled by **IADC0** (not PDM). Both examples are
written: `pac_mic_vu` (register level) and `hal_mic_vu` (through the HAL driver
`sensors`/`microphone`). Neither can be verified yet because the converter is
wedged from the early experiments: `EN` reads `0x3` (enabled *and* disabling) and
any write to `SINGLE` faults at `BFAR=0x49004098`.

**That state survives a core reset and only a power cycle clears it** — unplug and
replug the board's USB, then run either example. Full detail, including the four
IADC rules that were learned the hard way, is in
`docs/04-reference/IADC-MIC-NOTES.md`.
