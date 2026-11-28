# eusart

EUSART0 in **asynchronous** (UART) mode.

## What it is

EUSART0 in **asynchronous** (UART) mode.

The crate's serial story was USART0 only, and the EUSART blocks were reachable only as `spi::Spi1`/`Spi2` in
synchronous mode. An EUSART is not a USART with extra registers: the frame lives in `FRAMECFG` instead of `FRAME`,
the data bits are encoded differently (`2` is eight bits here, `5` on the USART), the asynchronous divider is
`CLKDIV.DIV` while the synchronous one is `CFG2.SDIV`, and the block is clocked from `CMU_EUSART0CLKCTRL.CLKSEL`
rather than from the bus clock alone. A driver that reused the USART's register writes would have compiled and
produced no frames.

## What this driver is measured to do

`hal_eusart_loopback` writes eight distinct bytes through `CFG0.LOOPBK` and reads them back; a control run with the
same writes, the same routing and only the loopback bit cleared reads zero bytes. The record is in
`../../../../docs/../../../../docs/04-reference/EUSART-ASYNC-2026-11-27.md` and in the example's header.

## The clock, stated plainly

Asynchronous baud is derived from `EM01GRPCCLK`, which `CMU_EUSART0CLKCTRL` selects at reset and which
`Board::eusart0_async` pins explicitly. On this board that branch is `HFRCODPLL` — the core clock — so the board
passes its own `sysclk_hz`. A product that moves SYSCLK to the crystal without moving `EM01GRPCCLK` must pass the
frequency that branch actually runs at; the constructor takes a number rather than reading one, so a wrong
assumption is a caller's visible choice and not a driver's guess.

The divider arithmetic is `crate::cores::usart::eusart_clkdiv`, and it is **not** the USART's: the EUSART splits its
`CLKDIV` field into five fraction bits (`1 + DIV/32`) where the USART uses eight (`1 + DIV/256`). Measured, not
assumed — the first version of this driver reused the USART's function and produced 15 650 baud where 115 200 was
asked for (`../../../../docs/../../../../docs/04-reference/EUSART-ASYNC-2026-11-27.md`). Both functions do their multiplication in `u64`, which is
what the wrap that once wrote `52` where `2382` belonged was about.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `IF_FERR` | const | `IF.FERR` — a frame had no stop bit where one belonged. |
| `IF_PERR` | const | `IF.PERR` — a byte arrived with a parity error. |
| `IF_RXOF` | const | `IF.RXOF` — the receiver ran out of room and at least one byte is gone. |
| `actual_baud` | fn | — |
| `cfg0` | fn | — |
| `clear_error_flags` | fn | Write one to each error flag and report what it read before and after. |
| `divider` | fn | — |
| `flush` | fn | Wait until the last frame has left the shifter. |
| `last_rx_problem` | fn | — |
| `new_at` | fn | Configure EUSART0 from a peripheral clock in hertz. |
| `read_byte` | fn | Read one 8-bit frame, or `None` when the receiver holds nothing. |
| `read_frame` | fn | — |
| `rx_errors` | fn | — |
| `rx_flags` | fn | — |
| `set_internal_loopback` | fn | Connect the receiver to this peripheral's own transmitter, inside the part. |
| `status` | fn | — |
| `write` | fn | Write every byte in order. |
| `write_byte` | fn | Write one byte, waiting until the transmitter has room for it. |

## Registers it touches

None directly: this module composes other drivers and reaches no `pac::` peripheral of its own.

## What was measured

```
    eusart: ok=1 n=8 am=8 wr=8 err=0 ctl=0 div=297 cal=3290 ct=23 tt=23 bd=113975 if=0x000c2007 ft=0x000c2007 fc=0x000c2007 cf=0x00000002
    eusart1: n=8 matched=8 seen=8 cfg0=0x00000002 st=0x00003063 control=0 div=297 want=115200 err=0x00000000 routed=1 refused=1
    spi3: n=8 eusart0=8 usart0=8 eusart1=8 cross=0 control=0 err=0      PASS
```

Controls and the full story: [EUSART-ASYNC-2026-11-27.md](../../../../docs/04-reference/EUSART-ASYNC-2026-11-27.md), [EUSART1-2026-11-27.md](../../../../docs/04-reference/EUSART1-2026-11-27.md), [SPI-MASTERS-2026-11-27.md](../../../../docs/04-reference/SPI-MASTERS-2026-11-27.md)

## What is not proven

- `EUSART1` was the last row of the coverage boundary that read "compiled, not measured". (EUSART1-2026-11-27.md (`../../../../docs/04-reference/EUSART1-2026-11-27`))

## Using it

See `examples/hal_eusart1_loopback.rs`, `examples/hal_eusart_loopback.rs`
