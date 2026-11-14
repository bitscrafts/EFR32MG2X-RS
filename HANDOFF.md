# HANDOFF — EFR32MG2X-RS (XIAO MG24 Sense + Expansion Base)

**State: green, everything below verified on hardware.** Branch `master`, pushed to
`origin` (`bitscrafts/EFR32MG2X-RS`). Board left running `base_env_display` (BME680 on the panel), then
`base_ble_env_gatt` (the sensor over Bluetooth).

## The ladder, and how far it is built

```
PAC   ██████████  registers from the SVD, 26 examples, all measured on hardware
HAL   ██████████  pins, clocks, I2C, timers, ADC, IMU, microphone, BME680
RADIO █████████░  advertising, scanning, connections, GATT read/write/notify; pairing open
BSP   ████████░░  both boards usable: sensor, panel, RTC, buzzer, button; BLE sensor node
```

## Session 2026-11-14 — Bluetooth closes, and what closed it

1. **The Bluetooth stack works on hardware.** Beacon advertising with a name, scanning that decodes a known address, a
   connectable GATT server with read, write and notify, and the firmware echoing written bytes in its notifications --
   so the proof cannot be answered by a host cache.
2. **The BME680 reaches a phone.** `base_ble_env_gatt` publishes the carrier's sensor as the standard `0x181A`
   Environmental Sensing service with `0x2A6E` temperature, `0x2A6F` humidity and `0x2A6D` pressure, read and notified.
   Verified on the user's phone with nRF Connect, and byte for byte by `scripts/host-env-verify.py`.
3. **The heap defect that broke a second connection is contained.** A vendor-blob write of about 136 bytes into a
   64-byte allocation destroyed the block header that followed it, dropping 57 KiB of a 64 KiB arena out of reach.
   Blocks now carry 256 bytes of slack: with a client connected the arena keeps 53720 bytes instead of 72.
4. **The PSA layer for pairing exists and is verified on the board** -- P-256 generation, public-key export, ECDH,
   AES-CMAC, AES-CTR, AES-ECB -- behind a catalogue switch that is off in the product build.
5. **Documentation and hygiene for publication.** No references to the internal bench host or account remain; the
   vendored OpenOCD binary and three third-party `.hex` files left the tree (13 MB); the internal book tooling left the
   repository; CI now runs the same gate as the bench instead of a step that could not fail, and
   `scripts/check-examples-build.sh` builds all 26 examples with the features they declare.

**The one open question:** with the security manager enabled a pairing request is accepted and then fails at a DHKey
check with `pg=0 pe=0 pa=0` -- no cryptographic call was made -- so the fault is before the key exchange. The component
stays out of the product build until that is answered.

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
| `base_fall_detector` | app | `fall selftest real=clean synth=detected` |
| `bsp_env_clock` | BSP | `bsp dev=111 T=31C P=1017hPa H=67` |

## Layering, as built (2026-11-12)

```
crates/efr32mg24-pac              registers
crates/efr32mg24-hal              drivers + pin/bus primitives; no board pins anywhere
crates/xiao-mg24-bsp              the module: LED; feature "sense": IMU + microphone
crates/xiao-expansion-base-bsp    the base: OLED, BME68x, RTC, buzzer, button
libraries/motion-detection        domain policy: [i32;3] + dt_us -> Event, zero dependencies
```

Rules the gate enforces: nothing in `libraries/` may pull the hardware stack; every
`*-bsp` crate must have examples; every example living in a directory must be declared in
its manifest. `docs/03-development/EXAMPLES.md` lists each example with the hardware it
needs, and `docs/02-architecture/CRATE-ORGANISATION-RESEARCH.md` explains why the split is
this one (and how rp-hal, nrf-hal, stm32f4xx-hal and esp-hal each do it differently).

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
6. **Flash only through `scripts/flash.sh`** (dry-run by default, refuses
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

**On the board:** `base_fall_detector`, self tests `real=clean synth=ok 200Hz=2/2 400Hz=4/4`.
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
