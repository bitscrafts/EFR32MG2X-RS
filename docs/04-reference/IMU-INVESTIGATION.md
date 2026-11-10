# IMU (LSM6DS3TR-C) — investigation log, 2026-11-11

**Status: not reachable.** Every measurement below is from the board, with the
firmware described, not from a datasheet reading. The sensor is fitted (the
module is a Sense, and the microphone — a Sense-only part — works), but nothing
we tried makes it answer on any bus we can drive.

## What the sources say

| source | claim |
|---|---|
| `Seeed_Arduino_LSM6DS3/LSM6DS3.cpp`, `#if defined(ARDUINO_XIAO_MG24)` | `#define Wire Wire1`, `#define PIN_LSM6DS3TR_C_POWER PD5` |
| Silicon Labs Arduino core `variants/xiao_mg24/pins_arduino.h` | `WIRE_HOWMANY 2`; `PIN_WIRE_SDA/SCL = D4/D5 = PC4/PC5`; `PIN_WIRE_SDA1/SCL1 = D14/D13 = PB3/PB2` |
| LSM6DS3 datasheet | `WHO_AM_I` = 0x69 (LSM6DS3) or 0x6A (LSM6DS3-C/TR-C); address 0x6A with SA0 low, 0x6B with SA0 high |
| Sense product page | "IMU heating: dedicated heating resistors around the IMU with GPIO-controlled drive" |

## Measurements

1. **Base bus (PC4/PC5), PD5 high** — `3c 51 76`, no `6a`, no `6b`. Repeated with
   PC8 high: identical. So the panel, the RTC and the BME68x answer and the IMU
   does not.
2. **I2C1 (PB3/PB2)** per the variant file — the scan returns **nothing at all**,
   and `WHO_AM_I` reads `0xff` (no device).
3. **Both sensor rails high** (PD5 = IMU per Seeed, PC8 = microphone) — same as
   above.
4. **Do PB2/PB3 carry an I2C bus at all?** With every GPIO set to *input, no
   pull*: `PORTB_DIN = 0x0C`, so PB2 and PB3 read high with no internal pull —
   there are external pull-ups on those two pins, which is what an I2C bus looks
   like. The same measurement shows the base bus (`PORTC_DIN = 0x34`, PC4/PC5)
   and PA1/PA2, PA4..PA7 pulled up as well.
5. **Enable search**: 22 candidate enable pins driven high in turn (PD5 first,
   then PA0..PA6, PB0/PB1/PB4/PB5, PC0..PC3/PC6/PC7, PD0..PD4), probing **0x6A
   and 0x6B** after each — `tried=22 en=p255.255`, nothing answered on either
   address.

## What this rules out, and what it does not

Ruled out:

- "the IMU is on the base bus and we were scanning the wrong address" — both
  addresses were probed on that bus via a full 0x08..0x77 scan;
- "the IMU needs PD5" — PD5 high was in every configuration;
- "the IMU is on PB2/PB3 but unpowered, and its enable is one of the obvious
  pins" — 22 pins tried with both addresses;
- "our bit-bang master is broken" — the same code, same day, reads the panel, the
  RTC and the BME68x on the other bus, and reports the correct chip ids.

Not ruled out:

- the IMU is on **SPI** (the LSM6DS3 supports it), in which case no I2C scan will
  ever find it;
- the IMU sits behind the SAMD11 or another switch on this board revision;
- the part is fitted but not bonded/functional on this unit;
- the enable is an active-low signal we have been holding high.

## Next steps that do not require guessing

1. **Look at the board**: the module's silkscreen and the location of the IMU
   relative to the visible microphone. If the IMU package is not populated, the
   question is answered.
2. **Continuity from the IMU pads** (if reachable) to the MG24 pins — that ends
   the guessing in one measurement.
3. **Try SPI** on the pins the LSM6DS3 driver would use in `SPI_MODE`
   (`LSM6DS3.h` defines the CS pin per board); a SPI probe needs no address and
   `WHO_AM_I` answers directly.
4. **Try active-low enable**: repeat the 22-pin search driving each pin low.

Until one of those is done, the HAL driver (`sensors::Lsm6ds3trc`) is written and
compiles, the board wiring is written (`board::SensorPower`, PD5), and both are
tested as far as they can be without a device that answers.
