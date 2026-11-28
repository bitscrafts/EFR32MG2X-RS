# usart

USART (Universal Synchronous/Asynchronous Receiver/Transmitter) driver

## What it is

USART (Universal Synchronous/Asynchronous Receiver/Transmitter) driver

This module provides a safe interface to the EFR32MG24 USART peripherals for
serial communication. It supports:

- Configurable baud rates
- 8 or 9 data bits
- None, even, or odd parity
- 1 or 2 stop bits
- Blocking TX/RX operations

## Hardware Registers

The driver directly manipulates these USART hardware registers:

- **EN**: Enable register (USART peripheral enable)
- **FRAME**: Frame format (data bits, parity, stop bits)
- **CLKDIV**: Clock divider for baud rate generation
- **CMD**: Command register (enable TX/RX, clear FIFOs)
- **STATUS**: Status flags (TXBL, RXDATAV, TXC, etc.)
- **TXDATA**: Transmit data register (8-bit write-only)
- **RXDATA**: Receive data register (8-bit read-only)

## Example

```no_run
use efr32mg2x_hal::usart::{Usart0, Config};
use efr32mg2x_hal::clock::Clocks;

let dp = pac::Peripherals::take().unwrap();
let clocks = Clocks::new(dp.CMU_S);

// Configure USART0 for 115200 baud, 8N1
let mut usart = Usart0::new(dp.USART0_S, Config::default(), &clocks);

// Transmit a byte
usart.write_byte(b'H');

// Receive a byte
if let Some(data) = usart.read_byte() {
// Process received data
}
```

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Config` | struct | — |
| `DataBits` | enum | — |
| `Error` | enum | — |
| `Parity` | enum | — |
| `StopBits` | enum | — |
| `Usart0` | struct | USART0 peripheral. |
| `flush` | fn | Flush the TX buffer (blocking) Waits until all transmitted data has been sent (TX complete). |
| `flush_receiver` | fn | Flush the receiver with `CMD.CLEARRX`, which the manual gives as the way to clear its state. |
| `fn` | const | Create a new USART configuration with common settings Default: 8 data bits, no parity, 1 stop bit. |
| `last_rx_problem` | fn | — |
| `new` | fn | Create a new USART0 instance Takes ownership of the USART0_S peripheral and configures it according to the provided configuration. |
| `new_at` | fn | The same driver, built from a peripheral clock in hertz instead of a `FrozenClocks`. |
| `overran` | fn | — |
| `read_byte` | fn | Read a single byte (non-blocking) Returns `Some(byte)` if data is available, `None` otherwise. |
| `rx_errors` | fn | — |
| `rx_flags` | fn | — |
| `set_internal_loopback` | fn | Connect the receiver to this peripheral's own transmitter, inside the part. |
| `write` | fn | Write a slice of bytes (blocking) Writes all bytes in the slice sequentially. |
| `write_byte` | fn | Write a single byte (blocking) Waits until the TX buffer has space, then writes the byte. |

## Registers it touches

`Peripherals`, `Usart0S`

## What was measured

```
    eusart: ok=1 n=8 am=8 wr=8 err=0 ctl=0 div=297 cal=3290 ct=23 tt=23 bd=113975 if=0x000c2007 ft=0x000c2007 fc=0x000c2007 cf=0x00000002
    eusart1: n=8 matched=8 seen=8 cfg0=0x00000002 st=0x00003063 control=0 div=297 want=115200 err=0x00000000 routed=1 refused=1
    spi3: n=8 eusart0=8 usart0=8 eusart1=8 cross=0 control=0 err=0      PASS
    usart: selftest n=6 ok=0 bad=6 rxerr=7 pre=1f cleared=1f post=1f
```

Controls and the full story: [EUSART-ASYNC-2026-11-27.md](../../../../docs/04-reference/EUSART-ASYNC-2026-11-27.md), [EUSART1-2026-11-27.md](../../../../docs/04-reference/EUSART1-2026-11-27.md), [SPI-MASTERS-2026-11-27.md](../../../../docs/04-reference/SPI-MASTERS-2026-11-27.md), [USART-RX-REFUSAL-2026-11-25.md](../../../../docs/04-reference/USART-RX-REFUSAL-2026-11-25.md)

## What is not proven

- `EUSART1` was the last row of the coverage boundary that read "compiled, not measured". (EUSART1-2026-11-27.md (`../../../../docs/04-reference/EUSART1-2026-11-27`))

## Using it

See `examples/hal_eusart1_loopback.rs`, `examples/hal_eusart_loopback.rs`, `examples/hal_one_of_each.rs`, `examples/hal_usart_link.rs`, `examples/hal_usart_rx_recovery.rs`
