# I2C Master Mode Driver

This module provides I2C (Inter-Integrated Circuit) master mode functionality for the EFR32MG24 microcontroller.

## Phase A Update (December 13, 2025)

**Internal Changes Only** - No API changes for users.

**What Changed**:
- I2C peripheral clock enable now uses safe `FrozenClocks::enable_peripheral_clock()` instead of unsafe CMU pointer
- Fixed CMU ownership violation by using safe accessor pattern
- No changes to public API - `I2c0::new()`, `I2c1::new()` remain unchanged

Users do not need to modify their code. The internal implementation is now safer and follows Rust ownership semantics correctly.

---

## Features

- **I2C Master Mode**: Full I2C master implementation with 7-bit addressing
- **Multiple Peripherals**: Support for both I2C0 and I2C1
- **Configurable Speed**: Standard mode (100 kHz) and Fast mode (400 kHz)
- **embedded-hal v1.0**: Complete I2C trait implementations
- **Hardware Register Access**: Direct register manipulation for optimal performance
- **Blocking Operations**: Simple blocking read/write operations

## Hardware Overview

The EFR32MG24 has two I2C peripherals (I2C0 and I2C1), each with:
- Support for master and slave modes (this driver implements master only)
- Programmable clock frequency via CLKDIV register
- Hardware start/stop/ACK/NACK generation
- TX/RX FIFOs for efficient data transfer
- Automatic clock stretching support

### Key Registers

| Register | Purpose |
|----------|---------|
| **EN** | Enable I2C peripheral |
| **CTRL** | Control register (master/slave mode, auto-ACK, etc.) |
| **CMD** | Command register (START, STOP, ACK, NACK) |
| **STATUS** | Status flags (TXBL, RXDATAV, TXC, etc.) |
| **CLKDIV** | Clock divider for SCL frequency |
| **TXDATA** | Transmit data register |
| **RXDATA** | Receive data register |
| **IF** | Interrupt flags (NACK, arbitration lost, etc.) |

## Clock Configuration

The I2C clock frequency is derived from HCLK:

```
SCL frequency = HCLK / (8 * (CLKDIV + 1))
```

Example with 39 MHz HCLK:
- Standard 100 kHz: CLKDIV = 48  (39 MHz / (8 * 49) ≈ 99.5 kHz)
- Fast 400 kHz: CLKDIV = 11      (39 MHz / (8 * 12) ≈ 406 kHz)

The driver automatically calculates the correct CLKDIV value based on the configured speed and system clock.

## Usage

### Basic Initialization

```rust
use efr32mg24_hal::{
    clock::{Clocks, ClockConfig, HfxoConfig},
    i2c::{I2c0, Config, Speed},
    pac,
};

let dp = pac::Peripherals::take().unwrap();

// Configure clocks
let clocks = Clocks::new(dp.cmu_s, ClockConfig {
    hfxo: Some(HfxoConfig::new(39_000_000)),
    lfxo: Some(Default::default()),
}).freeze();

// Create I2C0 instance at 100 kHz
let mut i2c = I2c0::new(
    dp.i2c0_s,
    Config::new(Speed::Standard100kHz),
    &clocks
);
```

### Write Operation

```rust
// Write to device at address 0x6B
let data = [0x75, 0x80]; // Register 0x75, value 0x80
i2c.write(0x6B, &data)?;
```

### Read Operation

```rust
// Read from device at address 0x6B
let mut buffer = [0u8; 4];
i2c.read(0x6B, &mut buffer)?;
```

### Write-Read (Register Read)

```rust
// Read register 0x75 from device 0x6B
let mut buffer = [0u8; 1];
i2c.write_read(0x6B, 0x75, &mut buffer)?;
let who_am_i = buffer[0];
```

### Using embedded-hal Traits

```rust
use embedded_hal::i2c::I2c;

// Read WHO_AM_I register (0x75)
let mut data = [0u8; 1];
i2c.write_read(0x6B, 0x75, &mut data)?;
```

## Pin Configuration

**Note**: This module does NOT configure GPIO pins. Pin configuration must be done separately using the GPIO module or your board's BSP.

### Common I2C Pin Locations

**I2C0**:
- Location 0: SCL=PA5, SDA=PA6
- Location 1: SCL=PC4, SDA=PC5

**I2C1**:
- Location 0: SCL=PB2, SDA=PB3
- Location 1: SCL=PC6, SDA=PC7

**Pull-up Resistors**: External pull-up resistors (typically 4.7kΩ) are required on both SCL and SDA lines.

## Error Handling

The module defines three error types:

| Error | Description |
|-------|-------------|
| `Error::Nack` | Address or data not acknowledged by slave |
| `Error::Bus` | Bus error (arbitration lost, timeout, etc.) |
| `Error::InvalidData` | Invalid parameters (empty buffer, etc.) |

```rust
match i2c.write(0x6B, &data) {
    Ok(()) => {
        // Success
    }
    Err(Error::Nack) => {
        // Slave didn't acknowledge
        // - Wrong address?
        // - Device not present?
        // - Device busy?
    }
    Err(Error::Bus) => {
        // Bus error
    }
    Err(Error::InvalidData) => {
        // Invalid parameters
    }
}
```

## I2C Protocol Details

### Write Transaction

1. Send START condition
2. Send device address with W bit (bit 0 = 0)
3. Wait for ACK from slave
4. Send data bytes
5. Wait for ACK after each byte
6. Send STOP condition

### Read Transaction

1. Send START condition
2. Send device address with R bit (bit 0 = 1)
3. Wait for ACK from slave
4. Read data bytes
5. Send ACK after each byte (except last)
6. Send NACK after last byte
7. Send STOP condition

### Write-Read Transaction (Register Read)

1. Write phase: Send device address + register address
2. Read phase: Send device address with R bit + read data

The `write_read()` method combines both phases for convenience.

## Clock Management

The I2C peripheral clock is automatically enabled when creating an I2C instance:

```rust
// I2C0 clock enable (CMU CLKEN0 bit 14)
// I2C1 clock enable (CMU CLKEN0 bit 15)
```

The clock remains enabled for the lifetime of the I2C instance.

## Hardware Limitations

- **Master mode only**: Slave mode is not implemented
- **7-bit addressing**: 10-bit addressing is not supported
- **Blocking operations**: All operations wait for completion
- **No DMA**: Data transfer uses polling (DMA support planned for future)
- **No multi-master**: Arbitration handling not implemented

## Examples

See `examples/05_i2c.rs` for a complete working example.

## Implementation Status

- [x] I2C master mode initialization
- [x] Standard 100 kHz and Fast 400 kHz speeds
- [x] Blocking write operations
- [x] Blocking read operations
- [x] Write-read combined operations
- [x] embedded-hal v1.0 I2C trait
- [x] Error handling (NACK detection)
- [ ] Slave mode
- [ ] 10-bit addressing
- [ ] Non-blocking operations
- [ ] DMA support
- [ ] Multi-master arbitration
- [ ] SMBus support

## References

- EFR32MG24 Reference Manual, Chapter 21: I2C - Inter-Integrated Circuit Interface
- embedded-hal I2C traits: https://docs.rs/embedded-hal/latest/embedded_hal/i2c/
- I2C Specification: https://www.nxp.com/docs/en/user-guide/UM10204.pdf
