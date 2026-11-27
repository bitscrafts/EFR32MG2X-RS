# bus

Bit-banged I2C master on the XIAO's two I2C buses.

## What it is

Bit-banged I2C master on the XIAO's two I2C buses.

This is the working transport on this die. The on-chip I2C0 master does not
complete a read that follows a write — fourteen attempts are recorded in
`../../../../docs/../../../../docs/04-reference/I2C0-READ-DEFECT.md` — so the bus is driven from software
through PAC GPIO registers. No external crate, no HAL of another project.

Electrical contract (established by measurement, not assumption):

- pins in SVD MODE 8 (`WIREDAND`, open drain),
- `DOUT` = 0 pulls the line low, `DOUT` = 1 releases it,
- pull-ups come from the Expansion Base (both lines read 1 when idle),
- `DIN` samples SDA.

`HALF_BIT_CYCLES` is a CPU-cycle count, so the bus clock follows SYSCLK.
Anything from ~1 us per half-bit upwards is inside the I2C timing; slower is
always safe, and a sensor refresh does not care about the difference between
90 kHz and 400 kHz.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `BitBangI2c` | struct | Software I2C master over PAC GPIO registers, on whatever pins it is given. |
| `Bus` | struct | — |
| `BusError` | enum | — |
| `I2cBus` | trait | A 7-bit-addressed I2C master. |
| `SharedBus` | struct | A bus shared between drivers. |
| `fn` | const | — |
| `new` | fn | Wrap a bus cell for handing to a driver. |

## Registers it touches

None directly: this module composes other drivers and reaches no `pac::` peripheral of its own.

## What was measured

No mailbox record in `docs/` names this module, so treat its API as unmeasured on hardware. The first measurement belongs here, with the line the board produced and the controls around it.

## What is not proven

- documentation | **B** | `deny(missing_docs)` now holds on all 27 modules (the five exemptions are gone, and the module that needed them -- `cores::i2c::sim` -- is test-only). (HAL-MATURITY-2026-11-25.md (`../../../../docs/04-reference/HAL-MATURITY-2026-11-25`))

## Using it

See `examples/hal_i2c_bitbang.rs`, `examples/hal_imu_read.rs`, `examples/hal_timer0_pwm.rs`
