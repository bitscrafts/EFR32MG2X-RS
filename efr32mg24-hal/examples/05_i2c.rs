//! Generic I2C Master Mode Example
//!
//! This example demonstrates I2C master mode configuration and operation
//! using the embedded-hal I2C traits.
//!
//! # Hardware Setup (Generic)
//!
//! This example is hardware-agnostic. You need to:
//! 1. Connect an I2C device to I2C0 (or I2C1)
//! 2. Configure SCL and SDA pins for your board
//! 3. Add pull-up resistors (typically 4.7kΩ) to SCL and SDA
//! 4. Set the correct device address (0x6B in this example)
//!
//! # Clock Configuration
//!
//! Configure clocks according to your board's crystal frequency:
//! - XIAO MG24 Sense: 39 MHz HFXO
//! - Custom boards: Adjust HfxoConfig accordingly
//!
//! # Pin Configuration (Not shown in this example)
//!
//! Pin configuration should be done in your board support package (BSP).
//! Typical I2C0 pins:
//! - SCL: PA5 (Location 0) or PC4 (Location 1)
//! - SDA: PA6 (Location 0) or PC5 (Location 1)
//!

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_hal as hal;
use hal::clock::{ClockConfig, Clocks, HfxoConfig};
use hal::i2c::{Config, I2c0, Speed};
use hal::pac;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let _cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks
    // IMPORTANT: Adjust HFXO frequency for your board!
    // - XIAO MG24 Sense: 39 MHz
    // - BRD4186C (Radio Board): 38.4 MHz
    // - Custom board: Check your schematic
    let (clocks, cmu) = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)), // Adjust for your board
            lfxo: Some(Default::default()),
        },
    )
    .expect("Clock configuration failed");

    let frozen_clocks = clocks.freeze(cmu);

    // Create I2C0 instance with 100 kHz (standard mode)
    // For 400 kHz (fast mode), use Speed::Fast400kHz
    let mut i2c = I2c0::new(
        dp.i2c0_s,
        Config::new(Speed::Standard100kHz),
        &frozen_clocks,
    );

    // NOTE: Pin configuration is not shown here!
    // In a real application, configure SCL/SDA pins using GPIO module
    // or use your board's BSP which handles this automatically.

    // Example I2C device address (change for your device)
    const DEVICE_ADDR: u8 = 0x6B;

    // Example 1: Write single register
    // Write 0x80 to register 0x6B (reset command for many IMUs)
    let write_data = [0x6B, 0x80];
    match i2c.write(DEVICE_ADDR, &write_data) {
        Ok(()) => {
            // Write successful
        }
        Err(_e) => {
            // Handle error (NACK, bus error, etc.)
        }
    }

    // Example 2: Read single register
    // Read WHO_AM_I register (0x75) from device
    let mut buffer = [0u8; 1];
    match i2c.write_read(DEVICE_ADDR, 0x75, &mut buffer) {
        Ok(()) => {
            // buffer[0] now contains WHO_AM_I value
            let who_am_i = buffer[0];
            let _ = who_am_i; // Use value
        }
        Err(_e) => {
            // Handle error
        }
    }

    // Example 3: Read multiple bytes
    // Read 6 bytes starting from register 0x22
    let mut data = [0u8; 6];
    match i2c.write_read(DEVICE_ADDR, 0x22, &mut data) {
        Ok(()) => {
            // data[] now contains 6 bytes from device
        }
        Err(_e) => {
            // Handle error
        }
    }

    // Example 4: Using embedded-hal I2C trait directly
    let write_buf = [0x20, 0x9F]; // Example: configure register
    match i2c.write(DEVICE_ADDR, &write_buf) {
        Ok(()) => {
            // embedded-hal write successful
        }
        Err(_e) => {
            // Handle embedded-hal error
        }
    }

    // Main loop
    loop {
        // Continuous I2C operations
        // Read sensor data, process, etc.

        // For demonstration, read WHO_AM_I continuously
        let mut who_am_i = [0u8; 1];
        if i2c.write_read(DEVICE_ADDR, 0x75, &mut who_am_i).is_ok() {
            // Successfully read WHO_AM_I
            let _ = who_am_i[0];
        }

        // Add delay between reads
        // delay.delay_ms(100);
    }
}
