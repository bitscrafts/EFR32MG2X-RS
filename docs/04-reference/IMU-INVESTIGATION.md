# IMU (LSM6DS3TR-C) — resolved, 2026-11-11

**Status: working at both layers.** `WHO_AM_I` reads 0x6A and the accelerometer
measures one g, which is the physical check that the part is really sensing.

```
pac_imu     id=6a a=985,81,139mg g=2126,-2528,-288
hal_imu     id=6a a=970,79,142 g=1408,-2318,-402
```

The two agree within sensor noise (~1.5 %), which is the point of having built
them as twins: either can be used to judge the other.

## What the documentation got right, and what it did not say

Seeed's page is accurate about the device: `LSM6DS3 myIMU(I2C_MODE, 0x6A)` and
`pinMode(PD5, OUTPUT); digitalWrite(PD5, HIGH);` before `begin()`. Their library
also switches the bus with `#define Wire Wire1` for `ARDUINO_XIAO_MG24`.

What no page states is **which pins Wire1 is on, in which order**. The Arduino
core's variant answers that — and answers it wrong for this board:

```
variants/xiao_mg24/pins_arduino.h:  PIN_WIRE_SDA1 = D14 = PB3
                                    PIN_WIRE_SCL1 = D13 = PB2
measured on the board:             SDA = PB2,  SCL = PB3
```

The lines are swapped. With them in the variant's order the sensor never sees a
valid start condition, so every scan of "the right pins" came back empty while
the addresses, the enable and the pull-ups were all correct.

## How it was found

Not by reading another datasheet, but by measuring:

1. **`pac_pullup_probe`** — every GPIO set to *input, no pull*; any pin that still
   reads high is pulled up from outside the chip. `PORTB_DIN = 0x0C` identified
   PB2/PB3 as a real I2C bus (`PORTC_DIN = 0x34` confirmed the known one).
   The first version of this probe used MODE 0 and produced all zeros, because
   mode 0 disables the input buffer — a check that passes while measuring nothing.
2. **`pac_imu_probe`** — eight candidate pin pairs × both SDA/SCL orders × two
   addresses, with PD5 and PC8 held high. Result:
   `imu sda=p1.2 scl=p1.3 a=6a id=6a`.

## Two traps that cost time afterwards

- **`format_fixed` takes a value multiplied by 100.** Passing milligrams raw
  printed `a=9,0,1` for a reading of 975 mg — indistinguishable from a dead
  accelerometer. `hal_imu_regs` dumps the raw register bytes, which showed
  `ac=6e 3e …` = 15982 LSB = 975 mg: the sensor was right, the printout was not.
- **A single 12-byte burst from 0x22** covers gyro (0x22..0x27) and accel
  (0x28..0x2D). Reading the accelerometer as a second transaction happened to
  work here but is needless; the driver now mirrors the PAC path exactly.

## Facts worth keeping

| item | value |
|---|---|
| bus | PB2 = SDA, PB3 = SCL |
| power enable | PD5 high, ~10 ms before the first access |
| address | 0x6A (SA0 low); 0x6B would be the other strap |
| WHO_AM_I (0x0F) | 0x6A |
| configuration | CTRL3_C = 0x44 (BDU + IF_INC), CTRL1_XL = 0x40, CTRL2_G = 0x40 |
| scale | 0.061 mg/LSB at ±2 g, 8.75 mdps/LSB at ±250 dps |
| at rest | one axis ≈ 1000 mg; the gyro carries a small offset (~2 dps here) |

The IMU is **not** on the Expansion Base bus, so an I2C scan there shows
0x3C/0x51/0x76 and no 0x6A by design, not by fault.
