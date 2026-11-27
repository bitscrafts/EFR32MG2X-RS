# route

Routing a peripheral resource to a physical pin.

## What it is

Routing a peripheral resource to a physical pin.

## Why this exists

On this part a peripheral works perfectly while reaching no pin at all: the signal has to be
pointed at a port and a pin, and then enabled. Until both happen, the driver looks alive and the
wire is silent. That is the defect the product has been living with -- `i2c`, `usart` and `spi`
worked internally and never touched a pad.

The registers are not on the peripheral: they are on the GPIO block, which is why looking for
`i2c0_ns/route.rs` finds nothing. The manual says it plainly (EFR32xG24 RM, page 816): to route
I2C0's SDA to PB03, write PORT = 1 and PIN = 3 into `GPIO_I2C0_SDAROUTE`, then set
`GPIO_I2C0_ROUTEEN.SDAPEN`. It also says the pin itself must be configured for the resource --
open-drain for an I2C line.

## What is pure here, and what is not

The arithmetic -- which bits carry the port, which carry the pin, whether a given port/pin pair
exists, and what number means open-drain -- is pure and tested on the host. Writing those values
into the registers is the driver's job and needs the part.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `PIN_MAX` | const | Four bits, so pins 0 to 15. |
| `PIN_SHIFT` | const | `PIN` occupies bits [19:16]: pin 0 to 15 of the chosen port. |
| `PORT_MAX` | const | Two bits, so four ports -- and a port number above three must be refused, not masked. |
| `PORT_SHIFT` | const | `PORT` occupies bits [1:0] of a route register: PA = 0, PB = 1, PC = 2, PD = 3. |
| `PinMode` | enum | — |
| `RouteError` | enum | — |
| `port_number` | fn | The number a port takes in a route register: PA = 0, PB = 1, PC = 2, PD = 3. |
| `route_value` | fn | The value to write into a route register for `port`/`pin`. |
| `value` | fn | The value for `MODEn`, as the manual numbers it. |

## Registers it touches

None directly: this module composes other drivers and reaches no `pac::` peripheral of its own.

## What was measured

```
    eusart1: n=8 matched=8 seen=8 cfg0=0x00000002 st=0x00003063 control=0 div=297 want=115200 err=0x00000000 routed=1 refused=1
```

Controls and the full story: [EUSART1-2026-11-27.md](../../../../docs/04-reference/EUSART1-2026-11-27.md)

## What is not proven

- routed through the crossbar with the interrupt ending the wait is not measured. (DMA-IRQ-2026-11-27.md (`../../../../docs/04-reference/DMA-IRQ-2026-11-27`))

## Using it

See `examples/hal_i2c_async.rs`, `examples/hal_i2c_hardware_imu.rs`, `examples/hal_i2c_recovery.rs`
