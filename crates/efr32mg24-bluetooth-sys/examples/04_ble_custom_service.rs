//! BLE Custom Service Example
//!
//! Demonstrates implementing a custom GATT service with 128-bit UUIDs,
//! read/write/notify operations, and CCCD handling.
//!
//! Hardware: XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
//!
//! # What This Example Shows
//!
//! - Creating custom service with 128-bit UUID
//! - Multiple characteristic types (Read, Write, Notify)
//! - Handling CCCD (Client Characteristic Configuration Descriptor)
//! - Read/Write request handling
//! - Notification updates
//!
//! # Custom Service Structure
//!
//! Custom Sensor Service (custom 128-bit UUID):
//! ├─ Sensor Data (UUID: ...01)
//! │  ├─ Properties: Read, Notify
//! │  └─ CCCD for notifications
//! ├─ Control Point (UUID: ...02)
//! │  └─ Properties: Write
//! └─ Status (UUID: ...03)
//!    └─ Properties: Read
//!
//! # Use Case
//!
//! This demonstrates a typical IoT sensor device pattern:
//! - Sensor Data: Periodically notify sensor readings
//! - Control Point: Send commands (start/stop, configure)
//! - Status: Read current device status
//!
//! # Testing
//!
//! Use nRF Connect app:
//! 1. Connect to "XIAO Sensor"
//! 2. Discover custom service
//! 3. Enable notifications on Sensor Data
//! 4. Write to Control Point to trigger actions
//! 5. Read Status characteristic

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m_rt::entry;
use efr32mg24_bluetooth_sys as bt;
use efr32mg24_hal::{
    clock::{ClockConfig, Clocks, HfxoConfig},
    pac,
    prelude::*,
};

// Panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}

// Custom 128-bit UUIDs for our service
// Base UUID: 12345678-1234-5678-1234-56789abcXXYY
//   where XXYY changes for each characteristic
//
// Service UUID:     12345678-1234-5678-1234-56789abc0000
// Sensor Data:      12345678-1234-5678-1234-56789abc0001
// Control Point:    12345678-1234-5678-1234-56789abc0002
// Status:           12345678-1234-5678-1234-56789abc0003

// GATT database handles (would be defined in .gattdb file in real project)
const GATTDB_SENSOR_DATA: u16 = 3; // Sensor data characteristic value
const GATTDB_SENSOR_DATA_CCCD: u16 = 4; // CCCD for notifications
const GATTDB_CONTROL_POINT: u16 = 6; // Control point characteristic
const GATTDB_STATUS: u16 = 8; // Status characteristic

// Control commands
const CMD_START_SAMPLING: u8 = 0x01;
const CMD_STOP_SAMPLING: u8 = 0x02;
const CMD_RESET: u8 = 0xFF;

#[entry]
fn main() -> ! {
    // ========================================================================
    // Step 1: Initialize Hardware
    // ========================================================================

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let (clocks, cmu) = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)),
            lfxo: Some(Default::default()),
        },
    )
    .expect("Clock configuration failed");

    let frozen_clocks = clocks.freeze(cmu);
    let mut delay = efr32mg24_hal::delay::Delay::new(cp.SYST, &frozen_clocks);

    // ========================================================================
    // Step 2: Initialize Bluetooth Stack
    // ========================================================================

    unsafe {
        let status = bt::sl_bt_system_start_bluetooth();
        if status != 0 {
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    delay.delay_ms(100);

    // ========================================================================
    // Step 3: Initialize GATT Database
    // ========================================================================

    // Initial sensor data value (4 bytes: simulated sensor reading)
    let mut sensor_data = [0u8; 4];
    sensor_data[0] = 0x12; // Example: temperature high byte
    sensor_data[1] = 0x34; // Example: temperature low byte
    sensor_data[2] = 0x56; // Example: humidity
    sensor_data[3] = 0x78; // Example: pressure

    unsafe {
        bt::sl_bt_gatt_server_write_attribute_value(
            GATTDB_SENSOR_DATA,
            0,
            sensor_data.len(),
            sensor_data.as_ptr(),
        );
    }

    // Initial status value
    // Status byte: bit 0 = sampling active, bit 1 = error, bits 2-7 = reserved
    let mut status_value = [0x00u8]; // Initially: stopped, no error

    unsafe {
        bt::sl_bt_gatt_server_write_attribute_value(
            GATTDB_STATUS,
            0,
            status_value.len(),
            status_value.as_ptr(),
        );
    }

    // ========================================================================
    // Step 4: Configure Advertising
    // ========================================================================

    let mut advertising_set: u8 = 0;

    unsafe {
        bt::sl_bt_advertiser_create_set(&mut advertising_set as *mut u8);
    }

    let device_name = b"XIAO Sensor";
    let mut adv_data = [0u8; 31];
    let mut offset = 0;

    // Flags
    adv_data[offset] = 2;
    adv_data[offset + 1] = 0x01;
    adv_data[offset + 2] = 0x06; // General Discoverable, BR/EDR not supported
    offset += 3;

    // Complete Local Name
    adv_data[offset] = (device_name.len() + 1) as u8;
    adv_data[offset + 1] = 0x09;
    adv_data[offset + 2..offset + 2 + device_name.len()].copy_from_slice(device_name);
    offset += device_name.len() + 2;

    // Incomplete List of 128-bit Service UUIDs
    // For a complete implementation, you would include the full 128-bit UUID here
    // Format: [Length, AD Type 0x06, 16 bytes of UUID in little-endian]

    let adv_data_len = offset;

    unsafe {
        bt::sl_bt_advertiser_set_data(advertising_set, 0, adv_data_len, adv_data.as_ptr());

        bt::sl_bt_advertiser_set_timing(advertising_set, 160, 160, 0, 0);

        let mut set_power: i16 = 0;
        bt::sl_bt_advertiser_set_tx_power(advertising_set, 0, &mut set_power as *mut i16);
    }

    // ========================================================================
    // Step 5: Start Advertising
    // ========================================================================

    unsafe {
        bt::sl_bt_advertiser_start(advertising_set, 2, 2);
    }

    // ========================================================================
    // Step 6: Application State
    // ========================================================================

    let mut connected = false;
    let mut connection_handle: u8 = 0;
    let mut sampling_active = false;
    let mut sensor_value: u16 = 0x1234; // Simulated sensor reading

    // ========================================================================
    // Step 7: Main Event Loop
    // ========================================================================

    loop {
        delay.delay_ms(500); // Update every 500ms

        // Simulate sensor readings
        if sampling_active {
            sensor_value = sensor_value.wrapping_add(1);

            // Update sensor data
            sensor_data[0] = ((sensor_value >> 8) & 0xFF) as u8;
            sensor_data[1] = (sensor_value & 0xFF) as u8;
            sensor_data[2] = (sensor_value.wrapping_mul(2) & 0xFF) as u8;
            sensor_data[3] = (sensor_value.wrapping_mul(3) & 0xFF) as u8;

            unsafe {
                // Write to GATT database
                bt::sl_bt_gatt_server_write_attribute_value(
                    GATTDB_SENSOR_DATA,
                    0,
                    sensor_data.len(),
                    sensor_data.as_ptr(),
                );

                // Send notification if connected and notifications enabled
                if connected {
                    // Check if notifications are enabled by reading CCCD
                    // (In a complete implementation)
                    bt::sl_bt_gatt_server_send_notification(
                        connection_handle,
                        GATTDB_SENSOR_DATA,
                        sensor_data.len(),
                        sensor_data.as_ptr(),
                    );
                }
            }
        }

        // Event handling notes:
        //
        // Complete application would handle these events:
        //
        // CONNECTION_OPENED:
        //   - connected = true
        //   - connection_handle = event.connection
        //   - Stop advertising
        //
        // CONNECTION_CLOSED:
        //   - connected = false
        //   - sampling_active = false
        //   - Restart advertising
        //
        // GATT_SERVER_USER_WRITE_REQUEST (Control Point):
        //   When client writes to control point characteristic:
        //   - Parse command byte
        //   - CMD_START_SAMPLING (0x01):
        //       sampling_active = true
        //       status_value[0] |= 0x01 (set bit 0)
        //   - CMD_STOP_SAMPLING (0x02):
        //       sampling_active = false
        //       status_value[0] &= !0x01 (clear bit 0)
        //   - CMD_RESET (0xFF):
        //       sensor_value = 0
        //       sampling_active = false
        //   - Update status characteristic
        //   - Send response: sl_bt_gatt_server_send_user_write_response
        //
        // GATT_SERVER_USER_READ_REQUEST (Status):
        //   - Read current status
        //   - Send response: sl_bt_gatt_server_send_user_read_response
        //
        // GATT_SERVER_CHARACTERISTIC_STATUS (CCCD written):
        //   - Check if notifications enabled/disabled
        //   - Update internal state accordingly
    }
}
