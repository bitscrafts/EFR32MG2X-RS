//! SPI Master Mode Example
//!
//! This example demonstrates SPI master mode usage on the EFR32MG24 using
//! all three available SPI peripherals: Spi0 (USART0), Spi1 (EUSART0), and Spi2 (EUSART1).
//!
//! # Hardware Setup
//!
//! This is a generic HAL example that does NOT assume any specific board.
//! You must configure the appropriate pins for your board before using SPI:
//!
//! ## SPI0 (USART0 in SPI mode)
//! - **MOSI** (Master Out Slave In): Transmit pin
//! - **MISO** (Master In Slave Out): Receive pin
//! - **SCK** (Serial Clock): Clock output
//! - **CS** (Chip Select): GPIO pin (managed by application)
//!
//! ## SPI1 (EUSART0 in SPI mode)
//! - Same pin requirements as SPI0
//!
//! ## SPI2 (EUSART1 in SPI mode)
//! - Same pin requirements as SPI0
//!
//! ## Common Pin Locations
//!
//! Board-specific details belong in BSP crates. See docs/BSP_PLAN.md.
//!
//! # Clock Configuration
//!
//! IMPORTANT: Adjust HFXO frequency for your board!
//! - XIAO MG24 Sense: 39 MHz
//! - BRD4186C (Radio Board): 38.4 MHz
//! - Custom board: Check your schematic
//!
//! # Usage
//!
//! ```bash
//! cargo build --example 06_spi --features rt --release
//! ```

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_hal as hal;
use hal::clock::{ClockConfig, Clocks, HfxoConfig};
use hal::pac;
use hal::spi::{Config, Mode, Spi0, Spi1, Spi2};
use panic_halt as _;

#[entry]
fn main() -> ! {
    let _cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks
    // IMPORTANT: Adjust HFXO frequency for your board!
    let clocks = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)), // Adjust for your board
            lfxo: Some(Default::default()),
        },
    )
    .freeze();

    // Create all three SPI instances
    // Mode 0: CPOL=0, CPHA=0 (most common)
    // 1 MHz clock frequency

    // SPI0: USART0 in SPI mode
    let mut spi0 = Spi0::new(dp.usart0_s, Config::new(Mode::Mode0, 1_000_000), &clocks);

    // SPI1: EUSART0 in SPI mode
    let mut spi1 = Spi1::new(dp.eusart0_s, Config::new(Mode::Mode0, 1_000_000), &clocks);

    // SPI2: EUSART1 in SPI mode (using Mode 3 as example)
    let mut spi2 = Spi2::new(dp.eusart1_s, Config::new(Mode::Mode3, 4_000_000), &clocks);

    // NOTE: Pin configuration is not shown here!
    // In a real application, configure MOSI/MISO/SCK pins using GPIO module
    // or use your board's BSP which handles this automatically.
    //
    // Example pin setup (board-specific):
    // - MOSI: Configure as push-pull output, alternate function
    // - MISO: Configure as input
    // - SCK: Configure as push-pull output, alternate function
    // - CS: Configure as push-pull output (GPIO, not USART/EUSART)

    // Example 1: Write data using SPI0 (USART0)
    let write_data = [0x01, 0x02, 0x03, 0x04];
    match spi0.write(&write_data) {
        Ok(()) => {
            // Data written successfully via SPI0
        }
        Err(_e) => {
            // Handle error
        }
    }

    // Example 2: Read data using SPI1 (EUSART0)
    let mut read_buffer = [0u8; 4];
    match spi1.read(&mut read_buffer) {
        Ok(()) => {
            // Data read into read_buffer via SPI1
            // read_buffer now contains data from device
        }
        Err(_e) => {
            // Handle error
        }
    }

    // Example 3: Full-duplex transfer using SPI2 (EUSART1)
    // Simultaneously send and receive data
    let tx_data = [0xAA, 0xBB, 0xCC, 0xDD];
    let mut rx_data = [0u8; 4];
    match spi2.transfer(&mut rx_data, &tx_data) {
        Ok(()) => {
            // rx_data contains received bytes via SPI2
            // tx_data was transmitted
        }
        Err(_e) => {
            // Handle error
        }
    }

    // Example 4: Read from SPI register
    // Common pattern: send register address, then read data
    //
    // This would typically involve:
    // 1. Assert CS (chip select) low
    // 2. Write register address
    // 3. Read register data
    // 4. De-assert CS high
    //
    // CS is typically a GPIO pin controlled by application code,
    // not managed by the SPI peripheral itself.

    // Example register read pattern (pseudo-code):
    // cs_pin.set_low();
    // spi.write(&[REGISTER_ADDRESS])?;
    // spi.read(&mut buffer)?;
    // cs_pin.set_high();

    // Example 5: Using different SPI modes
    //
    // Mode 0: CPOL=0, CPHA=0 (clock low when idle, sample on rising edge)
    // Mode 1: CPOL=0, CPHA=1 (clock low when idle, sample on falling edge)
    // Mode 2: CPOL=1, CPHA=0 (clock high when idle, sample on falling edge)
    // Mode 3: CPOL=1, CPHA=1 (clock high when idle, sample on rising edge)
    //
    // Most SPI devices use Mode 0 or Mode 3.
    // Check your device datasheet for the correct mode.

    // Example 6: Different frequencies
    //
    // Common SPI frequencies:
    // - 1 MHz: Config::new(Mode::Mode0, 1_000_000)
    // - 4 MHz: Config::new(Mode::Mode0, 4_000_000)
    // - 8 MHz: Config::new(Mode::Mode0, 8_000_000)
    //
    // Maximum frequency depends on:
    // - HCLK frequency
    // - SPI device maximum speed
    // - PCB trace length and capacitance

    // Example 7: Working with SPI flash memory
    //
    // Typical SPI flash operations:
    //
    // Read ID:
    // cs.set_low();
    // spi.write(&[0x9F])?;  // RDID command
    // spi.read(&mut id_buffer)?;
    // cs.set_high();
    //
    // Read data:
    // cs.set_low();
    // spi.write(&[0x03, addr_h, addr_m, addr_l])?;  // READ command + address
    // spi.read(&mut data_buffer)?;
    // cs.set_high();
    //
    // Write enable:
    // cs.set_low();
    // spi.write(&[0x06])?;  // WREN command
    // cs.set_high();

    loop {
        // Your application code here
    }
}
