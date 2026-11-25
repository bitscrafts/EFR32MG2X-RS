# Drivers for the EFR32MG24, on top of the PAC

Typed drivers for the peripherals this board actually has, built on `efr32mg2x-pac` and nothing else. The point of the
layer is that a mistake becomes hard to express: a pin knows its port and cannot be handed to a driver for another one,
an I2C transfer borrows the bus instead of taking it, and the clock gates are opened by the driver that needs them.

```rust
use efr32mg2x_hal::{gpio::Pin, i2c::I2c};

let mut led = Pin::new(p.gpio_s, Port::A, 7);   // PA7, the XIAO's LED
led.set_high();                                 // active-low, so this is "off"
```

## Add this before you wonder why it does not link

```toml
[dependencies]
efr32mg2x-hal = "0.1"
cortex-m = { version = "0.7", features = ["critical-section-single-core"] }
```

Without `critical-section-single-core` the image fails to link with `undefined symbol:
_critical_section_1_0_acquire`. The PAC exposes the `critical-section` feature, but the *implementation* belongs to the
architecture crate — and nothing in the error says so. This is measured, not recalled: it is the first thing a fresh
project hits.

## What is in here

| module | what it covers |
|---|---|
| `clock`, `delay` | the CMU tree and a delay that counts real cycles |
| `gpio` | a pin with a port, read-modify-write helpers, interrupts |
| `i2c`, `usart`, `spi` | blocking transfers with the bus owned by the driver |
| `timer` | PWM and one-shot, with the prescaler worked out from the clock |
| `adc`, `dma` | single conversions, and continuous capture through LDMA |
| `rtc`, `wdog`, `emu` | the low-power side |
| `sensors` | the BME680 and the IMU, over I2C |
| `microphone` | the analog microphone: IADC, DMA and RMS |

## Four things the silicon does that the headers do not say

- **No `DOUTTGL` register on this part.** Toggle with read-modify-write. Code written for other EFR32 families compiles
  and does nothing.
- **`MODEH` exists only on PORTA and PORTC.** Writing it for PORTB or PORTD lands in the neighbouring register.
- **I2C0 reads and writes; what it needed was two driver rules.** The last byte of a reception must be read with NACK,
  or the follower keeps driving and the master never closes the reception; and the `IF` flags are only cleared through
  the `IF_CLR` alias at `+0x2000`, so writing 1 to the plain register leaves a NACK that refuses the next transaction.
  Both measured: [`docs/04-reference/I2C-RECOVERY-STATE.md`](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/04-reference/I2C-RECOVERY-STATE.md) and [`docs/04-reference/I2C0-READ-DEFECT.md`](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/04-reference/I2C0-READ-DEFECT.md).
- **The default clock makes every delay 2.3x short.** The counter is right; the clock is wrong. Fixed by bringing the
  HFXO up before counting, and documented in [`docs/04-reference/DELAY-CLOCK-DEFECT.md`](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/04-reference/DELAY-CLOCK-DEFECT.md) so that the next person reads a
  cause instead of a symptom.

## Testing

The drivers that can be tested without a board are: the pure logic lives in `efr32mg2x-bt-heap`,
`efr32mg2x-aes-modes`, `efr32mg2x-aes-ctr-radio` and `efr32mg2x-motion-detection`, which run under
`cargo test` on the host. The drivers in this crate need the part; each one has an example in the repository that was
run on a XIAO MG24 Sense and judged by something other than its own report.

## Licence

MIT OR Apache-2.0.
