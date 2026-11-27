# spi

SPI Master Mode Driver

## What it is

SPI Master Mode Driver

This module provides SPI (Serial Peripheral Interface) master mode functionality using
USART peripherals in SPI mode for the EFR32MG24 microcontroller.

# Hardware Registers

- **EN**: Enable register
- **CTRL**: Control register (sync mode enable, master/slave)
- **FRAME**: Frame format (data bits, clock polarity/phase, bit order)
- **CLKDIV**: Clock divider for SCK frequency
- **CMD**: Command register (TX/RX enable)
- **STATUS**: Status flags (TXBL, RXDATAV, TXC)
- **TXDATA**: Transmit data register
- **RXDATA**: Receive data register

# Features

- SPI master mode using USART peripherals
- Configurable clock frequency
- All 4 SPI modes (Mode 0-3)
- MSB-first and LSB-first bit order
- Blocking transfer operations
- embedded-hal v1.0 SPI traits

# Example

```no_run
use efr32mg2x_hal::{
clock::{Clocks, ClockConfig, HfxoConfig},
spi::{Spi0, Config, Mode},
pac,
};

let dp = pac::Peripherals::take().unwrap();

// Configure clocks
let clocks = Clocks::new(dp.cmu_s, ClockConfig {
hfxo: None,
lfxo: None,
}).freeze();

// Create SPI instance (1 MHz, Mode 0)
let mut spi = Spi0::new(
dp.usart0_s,
Config::new(Mode::Mode0, 1_000_000),
&clocks
);

// Transfer data
let tx_data = [0x01, 0x02, 0x03];
let mut rx_data = [0u8; 3];
spi.transfer(&mut rx_data, &tx_data).unwrap();
```

## API at a glance

| item | kind | what it does |
|---|---|---|
| `BitOrder` | enum | — |
| `Config` | struct | — |
| `Error` | enum | — |
| `Mode` | enum | — |
| `Phase` | enum | — |
| `Polarity` | enum | — |
| `Spi0` | struct | SPI0 using USART0 in SPI master mode. |
| `Spi1` | struct | SPI1 using EUSART0 in SPI master mode. |
| `Spi2` | struct | SPI2 using EUSART1 in SPI master mode. |
| `fn` | const | Create new SPI configuration # Arguments * `mode` - SPI mode (CPOL/CPHA) * `frequency` - SPI clock frequency in Hz # Example ```no_run let config = Config::new(Mode::Mode0, 1_000_000); // 1 MHz, Mode 0 ```. |
| `new` | fn | Creates a new SPI0 instance using USART0 # Arguments * `usart` - USART0_S peripheral instance * `config` - SPI configuration (mode, frequency, bit order) * `clocks` - Frozen clock configuration # Example ```no_run let spi = Spi0::new(dp.usart0_s, Config::new(Mode::Mode0, 1_000_000), &clocks); ```. |
| `new_at` | fn | The same driver, built from the HFCLK in hertz instead of a `FrozenClocks`. |
| `read` | fn | Read data (half-duplex, send zeros) # Arguments * `words` - Buffer to store read data # Returns `Ok(())` on success, `Err(Error)` on failure. |
| `transfer` | fn | Transfer data (full-duplex) Simultaneously sends and receives data. |
| `write` | fn | Write data (half-duplex, discard received data) # Arguments * `words` - Data to write # Returns `Ok(())` on success, `Err(Error)` on failure. |

## Registers it touches

`Eusart0S`, `Eusart1S`, `Peripherals`, `Usart0S`

## What was measured

```
    spi: n=8 ok=8 err=0 out=a5..34 in=a5..34                               PASS   SPI on a D10-D9 jumper
    spi: n=8 ok=1 err=0 out=a5..34 in=ff..ff                               FAIL   …the same firmware with MISO moved off the jumper
```

Controls and the full story: [HAL-REVIEW-BRIEF-2026-11-25.md](../../../../docs/04-reference/HAL-REVIEW-BRIEF-2026-11-25.md)

## What is not proven

- documentation | **B** | `deny(missing_docs)` now holds on all 27 modules (the five exemptions are gone, and the module that needed them -- `cores::i2c::sim` -- is test-only). (HAL-MATURITY-2026-11-25.md (`../../../../docs/04-reference/HAL-MATURITY-2026-11-25`))

## Using it

See `examples/hal_spi_loopback.rs`
