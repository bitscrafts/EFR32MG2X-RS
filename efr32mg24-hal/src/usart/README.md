# USART Module

Universal Synchronous/Asynchronous Receiver/Transmitter (USART) driver for EFR32MG24.

## Status

**Implementation Status**: Complete (Phase A - Safe CMU Access)
**Hardware Register Access**: Yes (USART0_S)
**embedded-hal-nb**: v1.0 traits implemented
**Date**: December 13, 2025

## Phase A Update (December 13, 2025)

**Internal Changes Only** - No API changes for users.

**What Changed**:
- USART peripheral clock enable now uses safe `FrozenClocks::enable_peripheral_clock()` instead of unsafe CMU pointer
- Fixed CMU ownership violation by using safe accessor pattern
- No changes to public API - `Usart0::new()`, `Usart1::new()`, etc. remain unchanged

Users do not need to modify their code. The internal implementation is now safer and follows Rust ownership semantics correctly.

## Features

- Configurable baud rates (up to PCLK/16)
- 8 or 9 data bits
- None, even, or odd parity
- 1 or 2 stop bits
- Blocking TX/RX operations
- Non-blocking embedded-hal-nb traits
- Hardware register manipulation

## Hardware Registers

The driver manipulates these USART0_S hardware registers directly:

| Register | Purpose | Access |
|----------|---------|--------|
| **EN** | Enable peripheral | Write |
| **FRAME** | Frame format (data bits, parity, stop bits) | Read/Write |
| **CLKDIV** | Clock divider for baud rate | Read/Write |
| **CMD** | Command register (TX/RX enable/disable) | Write |
| **STATUS** | Status flags (TXBL, RXDATAV, TXC, etc.) | Read |
| **TXDATA** | Transmit data buffer (8-bit) | Write |
| **RXDATA** | Receive data buffer (8-bit) | Read |

### Clock Configuration

The USART peripheral requires USART0 clock to be enabled in CMU:

```rust
// CMU_S CLKEN0 register, bit 1 = USART0 clock
cmu.clken0().modify(|r, w| unsafe {
    w.bits(r.bits() | (1 << 1))
});
```

### Baud Rate Calculation

```
CLKDIV = 256 * (PCLK / (oversample * baudrate) - 1)
```

For asynchronous mode:
- Oversample = 16
- CLKDIV field = bits [22:3] of CLKDIV register

Example: 115200 baud @ 39 MHz PCLK
```
CLKDIV = 256 * (39000000 / (16 * 115200) - 1) = 5376
```

## Module Organization

### mod.rs (217 lines)

Main module coordinator with USART peripheral implementation:
- `Usart0` struct: USART0 peripheral wrapper
- `new()`: Initialize and configure USART
- `write_byte()`: Blocking single byte transmission
- `read_byte()`: Non-blocking single byte reception
- `write()`: Blocking multi-byte transmission
- `flush()`: Wait for TX completion

### types.rs (106 lines)

Type definitions and configuration structures:
- `Config`: USART configuration (baud rate, data bits, parity, stop bits)
- `DataBits`: 8 or 9 data bits
- `Parity`: None, Even, Odd
- `StopBits`: 1 or 2 stop bits
- `Error`: USART error types (Framing, Parity, Overrun)

### traits.rs (57 lines)

embedded-hal-nb trait implementations:
- `ErrorType`: Error type association
- `Write<u8>`: Non-blocking write trait
- `Read<u8>`: Non-blocking read trait
- `flush()`: Wait for TX completion

## Usage

### Basic Initialization

```rust
use efr32mg24_hal::usart::{Usart0, Config};
use efr32mg24_hal::clock::Clocks;

let dp = pac::Peripherals::take().unwrap();
let clocks = Clocks::new(dp.CMU_S);

// Configure USART0 for 115200 baud, 8N1 (default)
let mut usart = Usart0::new(dp.USART0_S, Config::default(), &clocks);
```

### Custom Configuration

```rust
use efr32mg24_hal::usart::{Config, DataBits, Parity, StopBits};

let config = Config::new(9600)
    .data_bits(DataBits::Eight)
    .parity(Parity::Even)
    .stop_bits(StopBits::Two);

let mut usart = Usart0::new(dp.USART0_S, config, &clocks);
```

### Blocking Write

```rust
// Write single byte
usart.write_byte(b'H');

// Write string
usart.write(b"Hello, world!\r\n");

// Wait for transmission to complete
usart.flush();
```

### Non-Blocking Read

```rust
// Check for received data
if let Some(byte) = usart.read_byte() {
    // Process received byte
}
```

### embedded-hal-nb Traits

```rust
use embedded_hal_nb::serial::{Write, Read};

// Non-blocking write (returns WouldBlock if buffer full)
match usart.write(b'A') {
    Ok(()) => { /* Byte written */ }
    Err(nb::Error::WouldBlock) => { /* TX buffer full, try again */ }
    Err(nb::Error::Other(e)) => { /* Handle error */ }
}

// Non-blocking read (returns WouldBlock if no data)
match usart.read() {
    Ok(byte) => { /* Process byte */ }
    Err(nb::Error::WouldBlock) => { /* No data available */ }
    Err(nb::Error::Other(e)) => { /* Handle error */ }
}

// Non-blocking flush
match usart.flush() {
    Ok(()) => { /* TX complete */ }
    Err(nb::Error::WouldBlock) => { /* Still transmitting */ }
    Err(nb::Error::Other(e)) => { /* Handle error */ }
}
```

## Hardware Details

### USART0_S Peripheral

- Base Address: Defined in PAC
- Instance: Secure (TrustZone enabled)
- Pins: Configurable via GPIO module (TX/RX)

### Status Flags

| Flag | Bit | Description |
|------|-----|-------------|
| **RXENS** | 0 | Receiver enabled |
| **TXENS** | 1 | Transmitter enabled |
| **TXC** | 13 | TX complete |
| **TXBL** | 15 | TX buffer level (ready for data) |
| **RXDATAV** | 17 | RX data valid |
| **RXFULL** | 18 | RX FIFO full |

### Command Register

| Bit | Field | Description |
|-----|-------|-------------|
| 0 | RXEN | Receiver enable |
| 1 | RXDIS | Receiver disable |
| 2 | TXEN | Transmitter enable |
| 3 | TXDIS | Transmitter disable |
| 10 | CLEARTX | Clear TX buffer |
| 11 | CLEARRX | Clear RX buffer |

## Limitations

- Only USART0 is currently implemented
- No hardware flow control (RTS/CTS)
- No DMA support
- Blocking operations may cause delays

## Future Enhancements

- EUSART0/EUSART1 support (Enhanced USART)
- Hardware flow control
- DMA-based transfers
- Interrupt-driven operation
- Multiprocessor mode
- IrDA mode
- SPI mode (USART in SPI configuration)

## Reference Documentation

- EFR32MG24 Reference Manual, Chapter 18: USART
- Silicon Labs EMLIB USART driver
- embedded-hal-nb v1.0 specification

## Testing

Tested with hardware register access verification:
- Clock enable in CMU
- Frame configuration
- Baud rate calculation
- TX/RX operations

## Line Count

- mod.rs: 217 lines
- types.rs: 106 lines
- traits.rs: 57 lines
- **Total**: 380 lines

**Last Updated**: December 4, 2025
