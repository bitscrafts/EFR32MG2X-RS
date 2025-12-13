//! Checkpoint Example: All Tier 1 & 2 Modules Integration
//!
//! This example demonstrates the integration of all implemented HAL modules:
//! - Clock Management (CMU)
//! - Delay (SysTick)
//! - GPIO (LED output, Button input)
//! - USART (Serial communication)
//!
//! ## Hardware Configuration
//!
//! Based on Seeed Studio XIAO MG24 Sense pinout:
//! - LED: PB2 (onboard LED)
//! - Button: PB3 (onboard button, active low with pull-up)
//! - USART0 TX: PA5 (for serial output)
//! - USART0 RX: PA6 (for serial input)
//!
//! ## Example Behavior
//!
//! 1. Initialize all peripherals with 39 MHz HFXO crystal
//! 2. Send startup banner over USART at 115200 baud
//! 3. Enter main loop:
//!    - Toggle LED every 500ms
//!    - Check button state (print when pressed)
//!    - Echo any received UART characters
//!    - Send periodic status messages
//!
//! ## Usage
//!
//! Connect a USB-to-Serial adapter:
//! - TX: PA5 (connect to adapter RX)
//! - RX: PA6 (connect to adapter TX)
//! - GND: Connect grounds
//!
//! Open serial terminal at 115200 baud to see output and interact.
//!
//! ## Hardware Register Operations
//!
//! This example exercises all hardware register operations implemented:
//! - CMU: SYSCLKCTRL, HFXOCTRL registers
//! - GPIO: PORTx_MODEL/MODEH, DOUT, DIN registers
//! - USART: EN, FRAME, CLKDIV, CMD, STATUS, TXDATA, RXDATA registers
//! - SysTick: CTRL, LOAD, VAL registers (via cortex_m)

#![no_std]
#![no_main]

use efr32mg24_hal as hal;
use hal::pac;
use hal::clock::{Clocks, ClockConfig, HfxoConfig};
use hal::delay::Delay;
use hal::gpio::GpioExt;
use hal::usart::{Usart0, Config as UsartConfig};
use hal::prelude::*;
use panic_halt as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    // Take peripheral singletons
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // ========================================================================
    // Clock Initialization (CMU)
    // ========================================================================
    // Configure clocks with XIAO MG24's 39 MHz external crystal
    let (clocks, cmu) = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)),
            lfxo: Some(Default::default()),
        }
    ).expect("Clock configuration failed");

    let frozen_clocks = clocks.freeze(cmu);

    // ========================================================================
    // Delay Initialization (SysTick)
    // ========================================================================
    let mut delay = Delay::new(cp.SYST, &frozen_clocks);

    // ========================================================================
    // GPIO Initialization
    // ========================================================================
    let gpio = dp.gpio_s.split(&frozen_clocks);

    // Configure LED on PB2 as push-pull output (onboard LED on XIAO MG24)
    let mut led = gpio.portb.pb2.into_push_pull_output();

    // Configure button on PB3 as input with pull-up (onboard button, active low)
    let mut button = gpio.portb.pb3.into_pull_up_input();

    // ========================================================================
    // USART Initialization
    // ========================================================================
    // Configure USART0 for 115200 baud, 8N1 (default)
    let mut usart = Usart0::new(dp.usart0_s, UsartConfig::default(), &frozen_clocks);

    // ========================================================================
    // Startup Banner
    // ========================================================================
    usart.write(b"\r\n");
    usart.write(b"=====================================\r\n");
    usart.write(b"  EFR32MG24 HAL Checkpoint Example\r\n");
    usart.write(b"=====================================\r\n");
    usart.write(b"\r\n");
    usart.write(b"Modules Initialized:\r\n");
    usart.write(b"  [x] Clock Management (CMU)\r\n");
    usart.write(b"  [x] Delay (SysTick)\r\n");
    usart.write(b"  [x] GPIO (LED + Button)\r\n");
    usart.write(b"  [x] USART (Serial @ 115200)\r\n");
    usart.write(b"\r\n");

    // Display clock configuration
    usart.write(b"Clock Configuration:\r\n");
    usart.write(b"  HFXO: 39 MHz\r\n");
    usart.write(b"  PCLK: ");
    write_u32(&mut usart, frozen_clocks.pclk().into());
    usart.write(b" Hz\r\n");
    usart.write(b"\r\n");

    usart.write(b"Hardware Configuration:\r\n");
    usart.write(b"  LED: PB2 (onboard)\r\n");
    usart.write(b"  Button: PB3 (onboard, active low)\r\n");
    usart.write(b"  USART TX: PA5\r\n");
    usart.write(b"  USART RX: PA6\r\n");
    usart.write(b"\r\n");

    usart.write(b"Example Behavior:\r\n");
    usart.write(b"  - LED blinks every 500ms\r\n");
    usart.write(b"  - Button press detection\r\n");
    usart.write(b"  - UART echo (type to test)\r\n");
    usart.write(b"  - Status messages every 5s\r\n");
    usart.write(b"\r\n");
    usart.write(b"Ready! Press button or type...\r\n");
    usart.write(b"\r\n");

    usart.flush();

    // ========================================================================
    // Main Loop
    // ========================================================================
    let mut led_state = false;
    let mut button_was_pressed = false;
    let mut loop_counter: u32 = 0;

    loop {
        // Toggle LED every 500ms
        if led_state {
            led.set_high().ok();
        } else {
            led.set_low().ok();
        }
        led_state = !led_state;

        // Check button state (active low with pull-up)
        let button_pressed = button.is_low().unwrap_or(false);
        if button_pressed && !button_was_pressed {
            // Button just pressed (falling edge)
            usart.write(b"[BUTTON] Pressed!\r\n");
        } else if !button_pressed && button_was_pressed {
            // Button just released (rising edge)
            usart.write(b"[BUTTON] Released.\r\n");
        }
        button_was_pressed = button_pressed;

        // Check for received UART data (non-blocking)
        if let Some(byte) = usart.read_byte() {
            // Echo received character
            usart.write(b"[UART] Received: '");
            usart.write_byte(byte);
            usart.write(b"' (0x");
            write_hex_u8(&mut usart, byte);
            usart.write(b")\r\n");

            // Handle special commands
            match byte {
                b'h' | b'H' => {
                    usart.write(b"\r\nHelp Menu:\r\n");
                    usart.write(b"  h - Show this help\r\n");
                    usart.write(b"  s - Show status\r\n");
                    usart.write(b"  r - Reset counter\r\n");
                    usart.write(b"  Any other key - Echo\r\n");
                    usart.write(b"\r\n");
                }
                b's' | b'S' => {
                    usart.write(b"\r\nStatus:\r\n");
                    usart.write(b"  Loop count: ");
                    write_u32(&mut usart, loop_counter);
                    usart.write(b"\r\n");
                    usart.write(b"  LED state: ");
                    if led_state {
                        usart.write(b"ON\r\n");
                    } else {
                        usart.write(b"OFF\r\n");
                    }
                    usart.write(b"  Button: ");
                    if button_pressed {
                        usart.write(b"PRESSED\r\n");
                    } else {
                        usart.write(b"RELEASED\r\n");
                    }
                    usart.write(b"\r\n");
                }
                b'r' | b'R' => {
                    loop_counter = 0;
                    usart.write(b"[INFO] Counter reset.\r\n");
                }
                b'\r' => {
                    // Carriage return: add line feed
                    usart.write(b"\r\n");
                }
                _ => {
                    // Already echoed above
                }
            }
        }

        // Send periodic status message every 10 loops (5 seconds)
        if loop_counter % 10 == 0 {
            usart.write(b"[STATUS] Loop #");
            write_u32(&mut usart, loop_counter);
            usart.write(b" - LED: ");
            if led_state {
                usart.write(b"ON");
            } else {
                usart.write(b"OFF");
            }
            usart.write(b", Button: ");
            if button_pressed {
                usart.write(b"PRESSED");
            } else {
                usart.write(b"RELEASED");
            }
            usart.write(b"\r\n");
        }

        loop_counter = loop_counter.wrapping_add(1);

        // Delay 500ms before next iteration
        delay.delay_ms(500);
    }
}

// ============================================================================
// Helper Functions for Serial Output
// ============================================================================

/// Write a u32 value as decimal ASCII string to USART
fn write_u32(usart: &mut Usart0, mut value: u32) {
    if value == 0 {
        usart.write_byte(b'0');
        return;
    }

    // Convert to decimal digits (max 10 digits for u32)
    let mut digits = [0u8; 10];
    let mut count = 0;

    while value > 0 {
        digits[count] = (value % 10) as u8 + b'0';
        value /= 10;
        count += 1;
    }

    // Write digits in reverse order (most significant first)
    for i in (0..count).rev() {
        usart.write_byte(digits[i]);
    }
}

/// Write a u8 value as 2-digit hexadecimal ASCII string to USART
fn write_hex_u8(usart: &mut Usart0, value: u8) {
    let high = (value >> 4) & 0x0F;
    let low = value & 0x0F;

    usart.write_byte(hex_digit(high));
    usart.write_byte(hex_digit(low));
}

/// Convert a 4-bit value to ASCII hex digit
fn hex_digit(value: u8) -> u8 {
    if value < 10 {
        b'0' + value
    } else {
        b'A' + (value - 10)
    }
}
