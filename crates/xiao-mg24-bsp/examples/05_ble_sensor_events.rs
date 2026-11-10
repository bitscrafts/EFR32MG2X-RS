//! BLE Sensor Integration with Event System
//!
//! Template for integrating onboard sensors (IMU, microphone) with BLE using
//! the event system for automatic connection management and GATT write handling.
//!
//! Hardware: XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
//!
//! # Onboard Sensors
//!
//! XIAO MG24 Sense includes:
//! - **LSM6DS3** - 6-axis IMU (accelerometer + gyroscope) on I2C
//! - **MSM381ACT001** - Analog MEMS microphone (PC8 power, PC9 input)
//!
//! # What This Example Shows
//!
//! - **Event system**: poll_event() for automatic connection management
//! - **BSP integration**: Board + Sensors + BLE working together
//! - **GATT write events**: Dynamic sampling rate control
//! - **GATT notifications**: Periodic sensor data updates
//! - **Power-efficient polling**: Event-driven architecture
//!
//! # BLE Service Structure
//!
//! Sensor Data Service:
//! ├─ Accelerometer Data (notify) - X, Y, Z axes (6 bytes)
//! ├─ Gyroscope Data (notify) - X, Y, Z axes (6 bytes)
//! └─ Sampling Rate (read/write) - Control update frequency (1 byte)
//!
//! # Hardware Connections (XIAO MG24 Sense)
//!
//! - LSM6DS3 IMU on I2C0
//!   - SDA: D4 (PA6)
//!   - SCL: D5 (PA5)
//!   - I2C Address: 0x6A
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
//!    cargo run --example 05_ble_sensor_events --target thumbv8m.main-none-eabihf --release
//!    ```
//!
//! 3. Connect with nRF Connect app:
//!    - Scan and connect to "XIAO Motion"
//!    - Find Sensor Data Service
//!    - Enable notifications on:
//!      - Accelerometer Data characteristic
//!      - Gyroscope Data characteristic
//!    - See simulated sensor data updating (10 Hz default)
//!
//! 4. Adjust sampling rate:
//!    - Write to Sampling Rate characteristic
//!    - Range: 1-100 Hz
//!    - Default: 10 Hz
//!    - Write event automatically updates internal rate!
//!
//! # Real Sensor Integration
//!
//! To integrate actual LSM6DS3 sensor:
//! 1. Uncomment I2C initialization (Step 2)
//! 2. Uncomment sensor configuration
//! 3. Uncomment sensor reads in main loop
//! 4. Remove simulated data generation

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m_rt::entry;
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
const GATTDB_ACCEL_DATA: CharHandle = CharHandle(3); // Accelerometer X,Y,Z (notify)
const GATTDB_GYRO_DATA: CharHandle = CharHandle(6); // Gyroscope X,Y,Z (notify)
const GATTDB_SAMPLING_RATE: CharHandle = CharHandle(9); // Update rate in Hz (read/write)

// Sampling configuration
const DEFAULT_SAMPLING_RATE_HZ: u8 = 10; // 10 Hz = 100ms interval
const MIN_SAMPLING_RATE_HZ: u8 = 1;
const MAX_SAMPLING_RATE_HZ: u8 = 100;

#[entry]
fn main() -> ! {
    // ========================================================================
    // Step 1: Initialize Board (BSP Layer)
    // ========================================================================

    let board = Board::take().unwrap();
    let mut delay = board.into_delay();

    // ========================================================================
    // Step 2: Initialize I2C for Sensor Communication (Optional)
    // ========================================================================

    // Configure I2C0 for LSM6DS3 IMU
    // SDA = D4 (PA6), SCL = D5 (PA5)
    //
    // Uncomment when ready to use actual sensor:
    //
    // let i2c_pins = (board.pins.d4, board.pins.d5);
    // let mut i2c = hal::i2c::I2c::new(
    //     board.peripherals.i2c0,
    //     i2c_pins,
    //     400_000, // 400 kHz Fast Mode
    //     &board.clocks,
    // );

    // ========================================================================
    // Step 3: Initialize Bluetooth Stack (Safe Wrapper)
    // ========================================================================

    let mut ble = XiaoBle::init(&mut delay).expect("Failed to initialize BLE");

    // Set device name (will appear in scanners)
    ble.set_device_name("XIAO Motion")
        .expect("Failed to set device name");

    // Initialize GATT database with default sampling rate
    ble.write_attribute(GATTDB_SAMPLING_RATE, &[DEFAULT_SAMPLING_RATE_HZ])
        .expect("Failed to initialize sampling rate");

    // ========================================================================
    // Step 4: Start Connectable Advertising
    // ========================================================================

    // Start advertising: 100ms interval, 0 dBm TX power
    ble.start_connectable(100, 0)
        .expect("Failed to start advertising");

    // ========================================================================
    // Step 5: Event Loop with Sensor Sampling
    // ========================================================================

    let mut sampling_rate_hz = DEFAULT_SAMPLING_RATE_HZ;
    let mut last_sample_ms = 0u32;
    let mut tick_counter = 0u32;

    // Simulated sensor data (for demonstration)
    let mut accel_x: i16 = 0;
    let mut gyro_z: i16 = 0;

    loop {
        // ====================================================================
        // Poll for Bluetooth Events
        // ====================================================================

        if let Some(event) = ble.poll_event() {
            match event {
                // ============================
                // Connection Event
                // ============================
                BleEvent::Connected { .. } => {
                    // Connection state automatically updated!
                    // Reset sampling timer on new connection
                    last_sample_ms = tick_counter;
                }

                // ============================
                // Disconnection Event
                // ============================
                BleEvent::Disconnected { .. } => {
                    // Connection state automatically cleared!
                    // Resume advertising for next connection
                    ble.start_connectable(100, 0)
                        .expect("Failed to resume advertising");
                }

                // ============================
                // GATT Write Event (Sampling Rate Change)
                // ============================
                BleEvent::GattWrite {
                    attribute, value, ..
                } => {
                    // Check if write is for sampling rate characteristic
                    if attribute == GATTDB_SAMPLING_RATE.value() && !value.is_empty() {
                        let new_rate = value[0];

                        // Validate range (1-100 Hz)
                        if (MIN_SAMPLING_RATE_HZ..=MAX_SAMPLING_RATE_HZ).contains(&new_rate) {
                            sampling_rate_hz = new_rate;

                            // Update GATT database to reflect accepted value
                            ble.write_attribute(GATTDB_SAMPLING_RATE, &[sampling_rate_hz])
                                .ok();
                        }
                    }
                }

                // ============================
                // Other Events (Ignored)
                // ============================
                _ => {}
            }
        }

        // ====================================================================
        // Periodic Sensor Sampling and Notification
        // ====================================================================

        // Only sample and notify if connected
        if ble.is_connected() {
            // Calculate sample interval in milliseconds
            let sample_interval_ms = 1000 / sampling_rate_hz as u32;

            // Check if it's time for next sample
            if (tick_counter - last_sample_ms) >= sample_interval_ms {
                last_sample_ms = tick_counter;

                // ============================
                // Read Sensor Data (Simulated)
                // ============================

                // For real sensor, uncomment this:
                //
                // let mut accel_data = [0u8; 6];
                // i2c.write_read(_LSM6DS3_ADDR, &[_OUTX_L_XL], &mut accel_data)
                //     .ok();
                //
                // let mut gyro_data = [0u8; 6];
                // i2c.write_read(_LSM6DS3_ADDR, &[_OUTX_L_G], &mut gyro_data)
                //     .ok();

                // Generate simulated sensor data (sine wave)
                accel_x = (accel_x + 100) % 2000 - 1000; // ±1000 (±2g scale)
                gyro_z = (gyro_z + 50) % 1000 - 500; // ±500 (±250 dps scale)

                // Format data: X, Y, Z axes (6 bytes each, little-endian i16)
                let accel_data = [
                    (accel_x & 0xFF) as u8,
                    (accel_x >> 8) as u8,
                    0x00,
                    0x00, // Y = 0
                    0x00,
                    0x00, // Z = 0
                ];

                let gyro_data = [
                    0x00,
                    0x00, // X = 0
                    0x00,
                    0x00, // Y = 0
                    (gyro_z & 0xFF) as u8,
                    (gyro_z >> 8) as u8,
                ];

                // ============================
                // Send GATT Notifications
                // ============================

                // Notify accelerometer data
                ble.notify(GATTDB_ACCEL_DATA, &accel_data).ok();

                // Notify gyroscope data
                ble.notify(GATTDB_GYRO_DATA, &gyro_data).ok();
            }
        }

        // ====================================================================
        // Timing and Delay
        // ====================================================================

        // 1ms tick for timing
        delay.delay_ms(1);
        tick_counter += 1;

        // Wrap counter every ~49 days
        if tick_counter >= 0xFFFF_FFFF {
            tick_counter = 0;
            last_sample_ms = 0;
        }
    }
}
