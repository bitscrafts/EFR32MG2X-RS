//! BLE LED Control with Event System
//!
//! Demonstrates BLE GATT server with event-driven connection and write handling.
//! Control the onboard LED via BLE with automatic connection management.
//!
//! Hardware: XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
//!
//! # What This Example Shows
//!
//! - **Event system**: poll_event() for automatic connection management
//! - **GATT write events**: Receive writes via BleEvent::GattWrite
//! - **Automatic reconnection**: Resume advertising on disconnect
//! - **Zero manual tracking**: No need for set_connected() workaround
//!
//! # Improvements Over 02_ble_led_control
//!
//! **Before** (manual tracking):
//! - Must call set_connected() every 3 seconds
//! - No disconnect detection
//! - 180 lines with boilerplate
//!
//! **After** (event system):
//! - Automatic connection state updates
//! - Disconnect events trigger auto-reconnection
//! - ~90 lines total
//!
//! # LED Control
//!
//! The LED can be controlled by writing to the LED Control characteristic:
//! - Write 0x00: Turn LED OFF
//! - Write 0x01: Turn LED ON
//! - Write 0x02: Toggle LED
//!
//! # Hardware Connections
//!
//! - LED on D0 (PB2) - Built-in LED on XIAO MG24
//!
//! # Testing
//!
//! 1. Extract Bluetooth blobs:
//!    ```bash
//!    cd efr32mg24-bluetooth-sys
//!    bash scripts/setup-bluetooth.sh
//!    ```
//!
//! 2. Build and flash:
//!    ```bash
//!    cargo run --example 04_ble_led_events --target thumbv8m.main-none-eabihf --release
//!    ```
//!
//! 3. Connect with nRF Connect app:
//!    - Scan and connect to "XIAO LED"
//!    - Find LED Control Service (custom UUID)
//!    - Write to LED Control characteristic (handle 3):
//!      - 0x00 = LED off
//!      - 0x01 = LED on
//!      - 0x02 = Toggle
//!    - Disconnect and observe automatic reconnection
//!
//! # Event Flow
//!
//! ```text
//! 1. Start advertising → "XIAO LED" visible
//! 2. Client connects → BleEvent::Connected
//!    - Connection state automatically set to true
//!    - Connection handle saved
//! 3. Client writes → BleEvent::GattWrite
//!    - LED updated based on value
//!    - State written to GATT database
//! 4. Client disconnects → BleEvent::Disconnected
//!    - Connection state automatically set to false
//!    - Advertising resumes automatically
//! 5. Repeat from step 2
//! ```

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m_rt::entry;
use embedded_hal::digital::OutputPin;
use xiao_mg24_bsp::radio::ble::{BleEvent, CharHandle, XiaoBle};
use xiao_mg24_bsp::Board;

// Panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}

// GATT database handles (from GATT Configurator)
const GATTDB_LED_CONTROL: CharHandle = CharHandle(3); // LED control characteristic (write)
const GATTDB_LED_STATE: CharHandle = CharHandle(5); // LED state characteristic (read)

// LED commands
const LED_OFF: u8 = 0x00;
const LED_ON: u8 = 0x01;
const LED_TOGGLE: u8 = 0x02;

#[entry]
fn main() -> ! {
    // ========================================================================
    // Step 1: Initialize Board (BSP Layer)
    // ========================================================================

    let mut board = Board::take().unwrap();
    let mut delay = board.into_delay();

    // Configure LED pin (D0 = PB2)
    let mut led = board.pins.d0.into_push_pull_output();
    led.set_low().ok(); // Start with LED off
    let mut led_state = LED_OFF;

    // ========================================================================
    // Step 2: Initialize Bluetooth Stack (Safe Wrapper)
    // ========================================================================

    let mut ble = XiaoBle::init(&mut delay).expect("Failed to initialize BLE");

    // Set device name (will appear in scanners)
    ble.set_device_name("XIAO LED").expect("Failed to set device name");

    // Initialize GATT database with default values
    ble.write_attribute(GATTDB_LED_CONTROL, &[LED_OFF])
        .expect("Failed to initialize LED control");
    ble.write_attribute(GATTDB_LED_STATE, &[LED_OFF])
        .expect("Failed to initialize LED state");

    // ========================================================================
    // Step 3: Start Connectable Advertising
    // ========================================================================

    // Start advertising: 100ms interval, 0 dBm TX power
    ble.start_connectable(100, 0)
        .expect("Failed to start advertising");

    // ========================================================================
    // Step 4: Event Loop with Automatic Connection Management
    // ========================================================================

    loop {
        // Poll for Bluetooth events (non-blocking)
        if let Some(event) = ble.poll_event() {
            match event {
                // ============================
                // Connection Event
                // ============================
                BleEvent::Connected { connection, .. } => {
                    // Connection state was AUTOMATICALLY updated by poll_event()!
                    // No need to call set_connected() manually
                    //
                    // The connection handle is now saved and ready for GATT operations
                }

                // ============================
                // Disconnection Event
                // ============================
                BleEvent::Disconnected { reason } => {
                    // Connection state was AUTOMATICALLY cleared by poll_event()!
                    //
                    // Resume advertising so clients can reconnect
                    ble.start_connectable(100, 0)
                        .expect("Failed to resume advertising");
                }

                // ============================
                // GATT Write Event
                // ============================
                BleEvent::GattWrite {
                    attribute, value, ..
                } => {
                    // Check if write is for LED control characteristic
                    if attribute == GATTDB_LED_CONTROL.value() && !value.is_empty() {
                        let command = value[0];

                        // Execute LED command
                        match command {
                            LED_OFF => {
                                led.set_low().ok();
                                led_state = LED_OFF;
                            }
                            LED_ON => {
                                led.set_high().ok();
                                led_state = LED_ON;
                            }
                            LED_TOGGLE => {
                                if led_state == LED_OFF {
                                    led.set_high().ok();
                                    led_state = LED_ON;
                                } else {
                                    led.set_low().ok();
                                    led_state = LED_OFF;
                                }
                            }
                            _ => {} // Unknown command, ignore
                        }

                        // Update LED state characteristic to reflect current state
                        ble.write_attribute(GATTDB_LED_STATE, &[led_state])
                            .ok();
                    }
                }

                // ============================
                // Other Events (Ignored for Now)
                // ============================
                _ => {
                    // GattRead, GattNotificationComplete, AdvertisingTimeout, Unknown
                    // Not needed for this simple LED control example
                }
            }
        }

        // Small delay to avoid busy-waiting (10ms polling interval)
        delay.delay_ms(10);
    }
}
