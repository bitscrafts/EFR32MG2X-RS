//! BLE GATT Server Example - Heart Rate Service
//!
//! Demonstrates implementing a standard Bluetooth SIG service (Heart Rate Service)
//! with GATT server operations using BGAPI.
//!
//! Hardware: XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
//!
//! # What This Example Shows
//!
//! - Implementing standard Bluetooth SIG Heart Rate Service (0x180D)
//! - Heart Rate Measurement characteristic (0x2A37)
//! - Sending notifications to connected clients
//! - GATT attribute operations (read/write/notify)
//! - Client Characteristic Configuration Descriptor (CCCD)
//!
//! # Service Structure
//!
//! Heart Rate Service (0x180D):
//! ├─ Heart Rate Measurement (0x2A37) - Notify
//! │  └─ CCCD (0x2902) - Read/Write (enable notifications)
//! └─ Body Sensor Location (0x2A38) - Read (optional)
//!
//! # Expected Behavior
//!
//! Once connected via BLE app:
//! - Enable notifications on Heart Rate Measurement
//! - Receive simulated heart rate values every second
//! - Values range from 60-180 bpm
//!
//! # Testing
//!
//! Use nRF Connect or similar BLE app:
//! 1. Connect to "XIAO HRM"
//! 2. Discover Heart Rate Service (0x180D)
//! 3. Enable notifications on characteristic 0x2A37
//! 4. Observe heart rate values updating every second

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

// Heart Rate Service UUIDs (Bluetooth SIG assigned)
const HEART_RATE_SERVICE_UUID: u16 = 0x180D;
const HEART_RATE_MEASUREMENT_CHAR_UUID: u16 = 0x2A37;
const BODY_SENSOR_LOCATION_CHAR_UUID: u16 = 0x2A38;

// GATT attribute handles
// Note: These would normally be obtained from a GATT database
// or by using sl_bt_gatt_server_find_attribute()
// For this example, we use typical handle values
const GATTDB_HEART_RATE_MEASUREMENT: u16 = 3; // Characteristic value handle
const GATTDB_BODY_SENSOR_LOCATION: u16 = 6; // Body sensor location handle

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

    // Write initial Heart Rate Measurement value
    // Format: [Flags, HR Value]
    // Flags byte:
    //   Bit 0: 0 = Heart Rate Value is UINT8
    //   Bit 1-2: Sensor Contact Status (2 = supported and contact detected)
    //   Bit 3: 0 = Energy Expended not present
    //   Bit 4: 0 = RR-Interval not present
    let mut hr_measurement = [0u8; 2];
    hr_measurement[0] = 0x06; // Flags: UINT8, sensor contact detected
    hr_measurement[1] = 75; // Initial heart rate: 75 bpm

    unsafe {
        bt::sl_bt_gatt_server_write_attribute_value(
            GATTDB_HEART_RATE_MEASUREMENT,
            0, // offset
            hr_measurement.len(),
            hr_measurement.as_ptr(),
        );
    }

    // Write Body Sensor Location value
    // 1 = Chest
    let sensor_location = [0x01u8];
    unsafe {
        bt::sl_bt_gatt_server_write_attribute_value(
            GATTDB_BODY_SENSOR_LOCATION,
            0,
            sensor_location.len(),
            sensor_location.as_ptr(),
        );
    }

    // ========================================================================
    // Step 4: Configure Advertising
    // ========================================================================

    let mut advertising_set: u8 = 0;

    unsafe {
        bt::sl_bt_advertiser_create_set(&mut advertising_set as *mut u8);
    }

    // Set advertising data with Heart Rate Service UUID
    let device_name = b"XIAO HRM";
    let mut adv_data = [0u8; 31];
    let mut offset = 0;

    // Flags
    adv_data[offset] = 2;
    adv_data[offset + 1] = 0x01; // AD Type: Flags
    adv_data[offset + 2] = 0x06; // General Discoverable, BR/EDR not supported
    offset += 3;

    // Complete Local Name
    adv_data[offset] = (device_name.len() + 1) as u8;
    adv_data[offset + 1] = 0x09; // AD Type: Complete Local Name
    adv_data[offset + 2..offset + 2 + device_name.len()].copy_from_slice(device_name);
    offset += device_name.len() + 2;

    // Complete List of 16-bit Service UUIDs
    adv_data[offset] = 3; // Length
    adv_data[offset + 1] = 0x03; // AD Type: Complete List of 16-bit Service UUIDs
    adv_data[offset + 2] = (HEART_RATE_SERVICE_UUID & 0xFF) as u8; // UUID LSB
    adv_data[offset + 3] = ((HEART_RATE_SERVICE_UUID >> 8) & 0xFF) as u8; // UUID MSB
    offset += 4;

    let adv_data_len = offset;

    unsafe {
        bt::sl_bt_advertiser_set_data(advertising_set, 0, adv_data_len, adv_data.as_ptr());

        bt::sl_bt_advertiser_set_timing(advertising_set, 160, 160, 0, 0);

        let mut set_power: i16 = 0;
        bt::sl_bt_advertiser_set_tx_power(advertising_set, 0, &mut set_power as *mut i16);
    }

    // ========================================================================
    // Step 5: Start Connectable Advertising
    // ========================================================================

    unsafe {
        bt::sl_bt_advertiser_start(advertising_set, 2, 2);
    }

    // ========================================================================
    // Step 6: Heart Rate Simulation Loop
    // ========================================================================

    let mut heart_rate: u8 = 60; // Start at 60 bpm
    let mut connected = false;
    let mut connection_handle: u8 = 0;

    loop {
        // Delay 1 second between measurements
        delay.delay_ms(1000);

        // Simulate heart rate changes
        heart_rate = if heart_rate >= 180 {
            60 // Reset to 60 bpm
        } else {
            heart_rate + 1 // Increment
        };

        // Update heart rate measurement value
        hr_measurement[1] = heart_rate;

        unsafe {
            // Write new value to GATT database
            bt::sl_bt_gatt_server_write_attribute_value(
                GATTDB_HEART_RATE_MEASUREMENT,
                0,
                hr_measurement.len(),
                hr_measurement.as_ptr(),
            );

            // If connected, send notification to client
            // In a complete implementation, you would:
            // 1. Check if notifications are enabled (CCCD = 0x0001)
            // 2. Only send if client has enabled notifications
            //
            // For this example, we call sl_bt_gatt_server_send_notification
            // which will only send if the client has enabled notifications
            if connected {
                bt::sl_bt_gatt_server_send_notification(
                    connection_handle,
                    GATTDB_HEART_RATE_MEASUREMENT,
                    hr_measurement.len(),
                    hr_measurement.as_ptr(),
                );
            }
        }

        // Event handling notes:
        //
        // In a complete application with event loop:
        //
        // CONNECTION_OPENED event:
        //   - connected = true
        //   - connection_handle = event.connection
        //   - Stop advertising
        //
        // CONNECTION_CLOSED event:
        //   - connected = false
        //   - Restart advertising
        //
        // GATT_SERVER_CHARACTERISTIC_STATUS event:
        //   - If CCCD written:
        //     - Check if notifications enabled (value == 0x0001)
        //     - Start/stop sending notifications accordingly
        //
        // GATT_SERVER_USER_READ_REQUEST event:
        //   - Handle read requests for characteristics
        //   - Send response with sl_bt_gatt_server_send_user_read_response
        //
        // GATT_SERVER_USER_WRITE_REQUEST event:
        //   - Handle write requests
        //   - Send response with sl_bt_gatt_server_send_user_write_response
    }
}
