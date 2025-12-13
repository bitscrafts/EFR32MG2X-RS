# SPI Master Mode Driver

This module provides SPI (Serial Peripheral Interface) master mode functionality using USART and EUSART peripherals for the EFR32MG24 microcontroller.

## Features

- **SPI Master Mode**: Full SPI master implementation using USART/EUSART in synchronous mode
- **Three SPI Peripherals**:
  - **Spi0**: Using USART0 in SPI mode
  - **Spi1**: Using EUSART0 in SPI mode
  - **Spi2**: Using EUSART1 in SPI mode
- **All SPI Modes**: Mode 0-3 (CPOL/CPHA combinations)
- **Configurable Frequency**: Any frequency up to PCLK/2
- **Bit Order**: MSB-first or LSB-first
- **embedded-hal v1.0**: Complete SpiBus trait implementation for all peripherals
- **Hardware Register Access**: Direct register manipulation for optimal performance
- **Blocking Operations**: Simple blocking read/write/transfer operations

## Hardware Overview

The EFR32MG24 implements SPI using USART and EUSART peripherals in synchronous mode. Each peripheral can function as:
- UART (asynchronous mode)
- SPI (synchronous mode)
- Other synchronous protocols

This driver configures USART0, EUSART0, and EUSART1 for SPI master mode.

### Key Registers

#### USART (Spi0)

| Register | Purpose |
|----------|---------|
| **EN** | Enable USART peripheral |
| **CTRL** | Control register (SYNC mode, CLKPOL, CLKPHA, MSBF) |
| **FRAME** | Frame format (data bits) |
| **CLKDIV** | Clock divider for SCK frequency |
| **CMD** | Command register (TXEN, RXEN) |
| **STATUS** | Status flags (TXBL, RXDATAV, TXC) |
| **TXDATA** | Transmit data register |
| **RXDATA** | Receive data register |

#### EUSART (Spi1, Spi2)

| Register | Purpose |
|----------|---------|
| **EN** | Enable EUSART peripheral |
| **CFG0** | Configuration register 0 (SYNC mode, MSBF) |
| **CFG2** | Configuration register 2 (CLKPOL, CLKPHA, MASTER) |
| **CLKDIV** | Clock divider for SCK frequency |
| **CMD** | Command register (TXEN, RXEN) |
| **STATUS** | Status flags (TXFL, RXFL) |
| **TXDATA** | Transmit data register |
| **RXDATA** | Receive data register |

## Clock Configuration

The SPI clock frequency is derived from PCLK (peripheral clock):

```
SCK frequency = PCLK / (2 * (1 + CLKDIV/256))
```

Example with 39 MHz PCLK:
- 1 MHz SPI: CLKDIV = 4864  (39 MHz / (2 * 1 MHz) - 1) * 256
- 4 MHz SPI: CLKDIV = 1216  (39 MHz / (2 * 4 MHz) - 1) * 256
- 8 MHz SPI: CLKDIV = 512   (39 MHz / (2 * 8 MHz) - 1) * 256

The driver automatically calculates the correct CLKDIV value based on the configured frequency and system clock.

## SPI Modes

| Mode | CPOL | CPHA | Clock Idle | Data Sampled |
|------|------|------|------------|--------------|
| **Mode 0** | 0 | 0 | Low | Rising edge |
| **Mode 1** | 0 | 1 | Low | Falling edge |
| **Mode 2** | 1 | 0 | High | Falling edge |
| **Mode 3** | 1 | 1 | High | Rising edge |

Most SPI devices use **Mode 0** or **Mode 3**. Check your device datasheet.

## Usage

### Basic Initialization

```rust
use efr32mg24_hal::{
    clock::{Clocks, ClockConfig, HfxoConfig},
    spi::{Spi0, Spi1, Spi2, Config, Mode},
    pac,
};

let dp = pac::Peripherals::take().unwrap();

// Configure clocks
let clocks = Clocks::new(dp.cmu_s, ClockConfig {
    hfxo: Some(HfxoConfig::new(39_000_000)),
    lfxo: Some(Default::default()),
}).freeze();

// Create SPI0 instance (USART0) at 1 MHz, Mode 0
let mut spi0 = Spi0::new(
    dp.usart0_s,
    Config::new(Mode::Mode0, 1_000_000),
    &clocks
);

// Create SPI1 instance (EUSART0) at 1 MHz, Mode 0
let mut spi1 = Spi1::new(
    dp.eusart0_s,
    Config::new(Mode::Mode0, 1_000_000),
    &clocks
);

// Create SPI2 instance (EUSART1) at 4 MHz, Mode 3
let mut spi2 = Spi2::new(
    dp.eusart1_s,
    Config::new(Mode::Mode3, 4_000_000),
    &clocks
);
```

### Write Operation

```rust
// Write data to SPI device
let data = [0x01, 0x02, 0x03, 0x04];
spi.write(&data)?;
```

### Read Operation

```rust
// Read data from SPI device (sends zeros)
let mut buffer = [0u8; 4];
spi.read(&mut buffer)?;
```

### Full-Duplex Transfer

```rust
// Simultaneously send and receive
let tx_data = [0xAA, 0xBB, 0xCC, 0xDD];
let mut rx_data = [0u8; 4];
spi.transfer(&mut rx_data, &tx_data)?;
```

### Using embedded-hal Traits

```rust
use embedded_hal::spi::SpiBus;

// Read from SPI device
let mut buffer = [0u8; 4];
spi.read(&mut buffer)?;

// Write to SPI device
spi.write(&[0x01, 0x02, 0x03])?;
```

### Different SPI Modes

```rust
// Mode 3: CPOL=1, CPHA=1 (common for SD cards)
let spi_mode3 = Spi0::new(
    dp.usart0_s,
    Config::new(Mode::Mode3, 4_000_000),
    &clocks
);
```

### LSB-First Bit Order

```rust
use efr32mg24_hal::spi::BitOrder;

// Configure for LSB-first transmission
let config = Config::new(Mode::Mode0, 1_000_000)
    .with_bit_order(BitOrder::LsbFirst);
let spi = Spi0::new(dp.usart0_s, config, &clocks);
```

## Pin Configuration

**Note**: This module does NOT configure GPIO pins. Pin configuration must be done separately using the GPIO module or your board's BSP.

### Typical SPI Pin Setup

All SPI peripherals require the same pin types:
- **MOSI** (TX): Push-pull output with alternate function
- **MISO** (RX): Input
- **SCK** (CLK): Push-pull output with alternate function
- **CS**: GPIO pin (application-controlled, not part of peripheral)

**SPI0 (USART0) Pin Location Examples** (board-specific):
- Location 0: TX=PA5, RX=PA6, CLK=PC0
- Location 1: TX=PC1, RX=PC2, CLK=PC3

**SPI1 (EUSART0) Pin Location Examples** (board-specific):
- Refer to EFR32MG24 datasheet for EUSART0 pin locations

**SPI2 (EUSART1) Pin Location Examples** (board-specific):
- Refer to EFR32MG24 datasheet for EUSART1 pin locations

Check your board schematic or reference manual for exact pin mappings.

### Chip Select (CS) Management

The CS (chip select) pin is typically a GPIO pin controlled by application code:

```rust
// Pseudo-code for CS management
cs_pin.set_low();           // Assert CS
spi.write(&[command])?;     // Send command
spi.read(&mut buffer)?;     // Read response
cs_pin.set_high();          // De-assert CS
```

## Common Use Cases

### SPI Flash Memory

```rust
// Read flash ID
cs.set_low();
spi.write(&[0x9F])?;  // RDID command
spi.read(&mut id_buffer)?;
cs.set_high();

// Read data from address
cs.set_low();
spi.write(&[0x03, addr_h, addr_m, addr_l])?;  // READ + address
spi.read(&mut data)?;
cs.set_high();
```

### SD Card

```rust
// SD cards typically use Mode 0 or Mode 3
let spi_sd = Spi0::new(
    dp.usart0_s,
    Config::new(Mode::Mode0, 400_000), // Start at 400 kHz
    &clocks
);

// After initialization, can increase to 25 MHz
```

### SPI Sensor

```rust
// Read sensor register
cs.set_low();
spi.write(&[READ_CMD | register_addr])?;
spi.read(&mut value)?;
cs.set_high();
```

## Error Handling

The module defines three error types:

| Error | Description |
|-------|-------------|
| `Error::Overrun` | Data received before previous data was read |
| `Error::FrameError` | Frame format error |
| `Error::InvalidConfig` | Invalid configuration (mismatched buffer sizes) |

```rust
match spi.transfer(&mut rx, &tx) {
    Ok(()) => {
        // Success
    }
    Err(Error::Overrun) => {
        // Handle overrun
    }
    Err(Error::InvalidConfig) => {
        // Buffer size mismatch
    }
    Err(_) => {
        // Other error
    }
}
```

## SPI Protocol Details

### Full-Duplex Transfer

SPI is inherently full-duplex: every transmitted bit results in a received bit.

```
Master:  TX -> MOSI -> Device
Master:  RX <- MISO <- Device
```

Even during "write-only" operations, the master receives data (usually ignored).

### Timing

- **Setup Time**: Data must be stable before clock edge
- **Hold Time**: Data must remain stable after clock edge
- **Clock Frequency**: Limited by device, trace length, and PCLK

## Clock Management

The SPI peripheral clock is automatically enabled when creating an SPI instance:

```rust
// SPI0 (USART0) clock enable (CMU CLKEN0 bit 1)
// SPI1 (EUSART0) clock enable (CMU CLKEN1 eusart0 bit)
// SPI2 (EUSART1) clock enable (CMU CLKEN1 eusart1 bit)
```

The clock remains enabled for the lifetime of the SPI instance.

## Hardware Limitations

- **Master mode only**: Slave mode not implemented
- **8-bit transfers**: Only 8-bit data words supported
- **Blocking operations**: All operations wait for completion
- **No DMA**: Data transfer uses polling (DMA support planned)
- **No multi-master**: Arbitration not supported
- **Single CS**: CS must be managed by application GPIO

## Examples

See `examples/06_spi.rs` for a complete working example.

## Implementation Status

- [x] SPI master mode initialization
- [x] Configurable clock frequency (up to PCLK/2)
- [x] All 4 SPI modes (Mode 0-3)
- [x] MSB-first and LSB-first bit order
- [x] Blocking write operations
- [x] Blocking read operations
- [x] Full-duplex transfer operations
- [x] embedded-hal v1.0 SpiBus trait
- [x] Error handling
- [ ] Slave mode
- [ ] 9-bit, 10-bit, or 16-bit transfers
- [ ] Non-blocking operations
- [ ] DMA support
- [ ] Multi-master support

## References

- EFR32MG24 Reference Manual, Chapter 22: USART - Universal Synchronous Asynchronous Receiver/Transmitter
- embedded-hal SPI traits: https://docs.rs/embedded-hal/latest/embedded_hal/spi/
- SPI Specification: https://www.nxp.com/docs/en/data-sheet/MC68HC11E.pdf
