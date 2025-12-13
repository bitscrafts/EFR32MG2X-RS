//! USART Communication Example
//!
//! This example demonstrates USART (serial) communication on the EFR32MG24.
//!
//! ## Hardware Configuration
//!
//! - USART0 TX: Configured via GPIO
//! - USART0 RX: Configured via GPIO
//! - Baud Rate: 115200
//! - Format: 8 data bits, no parity, 1 stop bit (8N1)
//!
//! ## Example Behavior
//!
//! This example demonstrates:
//! 1. USART initialization at 115200 baud
//! 2. Sending a welcome message
//! 3. Echo loop: reads bytes and echoes them back
//!
//! ## Usage
//!
//! Connect a USB-to-Serial adapter to USART0 TX/RX pins:
//! - TX: PA5 (transmit from MCU)
//! - RX: PA6 (receive to MCU)
//! - GND: Connect grounds
//!
//! Open a serial terminal at 115200 baud. Type characters and they will be echoed back.
//!
//! ## Hardware Register Operations
//!
//! The example demonstrates:
//! - CMU CLKEN0 register: Enable USART0 clock (bit 1)
//! - USART EN register: Enable USART peripheral
//! - USART FRAME register: Configure 8N1 format
//! - USART CLKDIV register: Set baud rate divider
//! - USART CMD register: Enable TX and RX
//! - USART STATUS register: Check TXBL, RXDATAV flags
//! - USART TXDATA register: Write transmit data
//! - USART RXDATA register: Read received data

#![no_std]
#![no_main]

use efr32mg24_hal as hal;
use hal::pac;
use hal::usart::{Config, Usart0};
use hal::clock::{Clocks, ClockConfig};
use panic_halt as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    // Take peripheral singletons
    let dp = pac::Peripherals::take().unwrap();

    // Initialize clocks with default configuration
    let (clocks, cmu) = Clocks::new(dp.cmu_s, ClockConfig::default())
        .expect("Clock configuration failed");
    let frozen_clocks = clocks.freeze(cmu);

    // Configure USART0 for 115200 baud, 8N1 (default)
    let mut usart = Usart0::new(dp.usart0_s, Config::default(), &frozen_clocks);

    // Send welcome message
    usart.write(b"\r\n");
    usart.write(b"EFR32MG24 USART Example\r\n");
    usart.write(b"=======================\r\n");
    usart.write(b"Type characters and they will be echoed back.\r\n");
    usart.write(b"\r\n");

    // Flush TX buffer
    usart.flush();

    // Echo loop: read bytes and echo them back
    loop {
        // Check for received data (non-blocking)
        if let Some(byte) = usart.read_byte() {
            // Echo the byte back
            usart.write_byte(byte);

            // Handle special characters
            match byte {
                b'\r' => {
                    // Carriage return: add line feed
                    usart.write_byte(b'\n');
                }
                b'\x03' => {
                    // Ctrl+C: Send message
                    usart.write(b"\r\nCtrl+C detected\r\n");
                }
                _ => {
                    // Normal character, already echoed
                }
            }
        }

        // Optionally add a small delay or yield to prevent busy-waiting
        // In a real application, you might want to use interrupts or WFI
        cortex_m::asm::nop();
    }
}
