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
| `hal_imu` | PASS `id=6a a=970,79,142 g=1408,-2318,-402` — IMU on PB2/PB3, PD5 high |

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

## Microphone (Sense) — verified, PAC and HAL

Seeed's own MG24 backend answers the hardware question: the mic is **analog**,
on **PC9**, powered by **PC8**, sampled by **IADC0** (not PDM).

```
pac_mic_vu   mic rst=ok rms=562 pk=598 n=128
hal_mic_vu   hal mic rms=521 pk=544 n=128
```

128 samples per window, same noise floor through both layers, LED brightness
following the level. The rule that unlocked it: `SINGLE` is a WSYNC register and
**can only be written while the converter is enabled**; writing it disabled is a
bus fault that leaves the block stuck in `DISABLING`, a state that survives a
core reset. Full detail in `docs/04-reference/IADC-MIC-NOTES.md`.

## The user LED is mono, not RGB (checked 2026-11-11)

Three independent lines of evidence:

1. **Two indicators, one of them controllable.** The getting-started page lists
   `LEDs: 1x User LED, 1x Power LED`; the Sense product page's feature table
   lists `Indicators: User LED / Charging LED`. So there are two LED packages but
   only **one user LED** — the other belongs to the charger: it is on while USB
   power is present and, per the same page, "will not have any LED on when it is
   battery powered". Neither is RGB; boards with an addressable LED (XIAO
   ESP32C3/S3, RP2040) say `1x RGB LED` in that table instead.
2. **The wiki text calls it "the orange user indicator"** and "the orange LED on
   the right side", and the pin map names PA07 `User Light_Yellow`. One part,
   one colour.
3. **Electrical behaviour**: exactly one pin is configured as an output and the
   LED lights from it. Measured `PORTA_MODEL = 0x40000000` — PA7 push-pull and
   nothing else on PORTA; PORTB/PORTD untouched. A tri-colour RGB needs three
   GPIOs; an addressable RGB (WS2812) cannot be lit by a steady DC level at all,
   and would show garbage rather than a clean 1 Hz blink when toggled by a NOP
   loop — which is exactly what the operator saw.

So brightness (PWM) is the only dimension available: the VU meter examples vary
duty, never colour.

Worth recording: **the factory program on the Sense is this very demo** — Seeed
ships it with "The louder you shout, the brighter the light will be". `pac_mic_vu`
and `hal_mic_vu` reproduce the factory behaviour on hardware.

## IMU: resolved — the bus pins were swapped

Seeed's docs name the part (LSM6DS3TR-C, `0x6A`) and its enable (`PD5`) correctly;
their library also switches to `Wire1` for the MG24. What no page states is which
pins Wire1 uses **in which order**. The Arduino variant claims SDA1=PB3/SCL1=PB2;
measured on this board it is **SDA = PB2, SCL = PB3**. With the lines swapped the
sensor never answers, which is why scans of "the right pins" came back empty.

Found by measurement, not by another datasheet: `pac_pullup_probe` (input/no-pull
on every GPIO → `PORTB_DIN=0x0C` proves PB2/PB3 carry external pull-ups) and
`pac_imu_probe` (8 pin pairs × 2 orders × 2 addresses → `sda=p1.2 scl=p1.3 a=6a
id=6a`).

Working at both layers:

```
pac_imu   id=6a a=985,81,139mg
hal_imu   id=6a a=970,79,142 g=1408,-2318,-402
```

Detail, including the two traps that followed (`format_fixed` takes value×100;
one 12-byte burst covers both sensors), in
`docs/04-reference/IMU-INVESTIGATION.md`.

## Fall detection: a classifier, and where it belongs

`crates/motion-detection` — hardware free, no dependencies, host tested (5 tests).
It takes `[ax, ay, az]` in mg plus the elapsed milliseconds and returns events:
`Fall { peak_mg, freefall_ms }` or `Impact { peak_mg }`.

The state machine needs **all three phases** — free fall below 400 mg for at least
120 ms, then an impact above 2500 mg, then at least 800 ms of stillness — because
each one alone is a different event:

- shaking produces 1.8 g peaks and must not alarm: measured on this board, so the
  `shake_reports_impact_but_not_fall` test uses those very samples;
- setting the board down is not a fall, hence the free-fall requirement;
- a slam on the table has no free-fall phase, so it is reported as `Impact`.

Deliberately **not** in the HAL. A driver answers "what did the accelerometer
say"; deciding that 2.5 g is an impact is product policy, and putting it in the
HAL would make every user of the driver inherit one product's thresholds. The
example (`hal_fall_detector`) is the application: it composes the HAL with the
algorithm crate, and the dependency runs application → algorithm, never back.

Verified on hardware, with two data-driven self tests run at boot:

```
fall selftest real=clean synth=detected
```

- *real*: 320 samples recorded from this board while it was shaken and set down
  (peak 1601 mg, dips to 630 mg) — no false fall;
- *synthetic*: 250 ms of free fall, a 6 g impact, then stillness — detected.

The IMU had to be reconfigured for this: ±8 g at 416 Hz, sampled at 200 Hz. At the
original ±2 g/104 Hz an impact either clips or falls between samples, which is
exactly why the first two captures contained no event at all.

## Linker fix: the RAM was limited to 4.5 KB

`memory.x` carried `_stack_start = 0x20001208`, the stack pointer the Arduino core
happens to use, which left the linker 4.5 KB of RAM in total: the 6 KB capture
buffer failed to link with "stack end address is not below stack start". The stack
now starts just under the SWD mailbox page (`0x2003F000`) and the linker gets
252 KB. All twelve PAC examples and every HAL example still build.

## RTC: the last base device now has a driver

`crates/efr32mg24-hal/src/rtc/pcf8563.rs` — the calendar chip at 0x51, seven bytes
in one transaction (seconds through years, BCD) so a reading cannot straddle a
rollover. Two behaviours matter more than the register map:

- **`VL` is surfaced, not hidden.** Bit 7 of the seconds register means "clock
  integrity not guaranteed" after a power loss. The driver returns
  `RtcError::ClockInvalid` rather than a plausible-looking wrong time: a clock that
  lies is worse than a clock that complains.
- **the report says where the value came from.** The first version of the example
  printed `kept` after the chip had in fact been written from the build-time
  reference, because the flag described the intent at boot instead of what
  happened. It now prints `ref-set` or `kept` according to what it really did.

Verified on hardware, including the part that a calendar chip exists for:

```
rtc 2026-11-10 12:00:02 ref-set   <- chip had lost power (VL set)
rtc 2026-11-10 12:00:48 kept      <- retained across the flash
rtc 2026-11-10 12:00:57 kept      <- advanced 9 s over a reset + 6 s wait
```

Two unit tests cover the BCD round trip and reject invalid nibbles (0x0A is not a
digit; a bus glitch must not become a time).

HAL parity for the base board is now complete: panel, BME68x, RTC, button, buzzer,
microphone, IMU. What is left of the ladder is the BSP facade and the peripherals
behind it (microSD, Grove UART/ADC, servo), then the radio.

## Buzzer in the HAL, and two calibration traps

`crates/efr32mg24-hal/src/buzzer.rs` drives the base's piezo on PC03 through TIMER0
CC0, reachable as `board.buzzer()`. Silence is `CC0_CTRL.CMOA = none` with the counter
still running — stopping the timer stops `CNT`, which breaks any wait that uses it.

Two mistakes, both caught by the report rather than by ear:

1. **`CNT` wraps.** With `TOP = 0xFFFF` the counter rolls over every 3.4 ms at 19 MHz,
   so a 10 ms calibration window measures the remainder, not the count. The driver now
   halves the window until it sees no wrap.
2. **`asm::delay(n)` is not `n` cycles.** Counting delay *iterations* against timer
   ticks inflated the rate by exactly 3.0 (19 MHz became 57 MHz). The calibration now
   counts **CPU cycles** with DWT, which is what the vendor's own MVP driver does.

Measured on the board:

```
buzzer tick=19000000Hz (sysclk=19000000) top=3518 tone=2700Hz
```

`TOP = 3518 = 19 MHz / (2 × 2700)`, and the tone is exact by construction. The one
remaining assumption is SYSCLK itself — 19 MHz is what this board produced when the
1 Hz blink was calibrated against a stopwatch — and the driver says so in the report
and allows `set_sysclk_hz` to override it. A tone measured against an assumed clock is
not the same claim as a tone measured with an instrument, and the output does not
pretend otherwise.
