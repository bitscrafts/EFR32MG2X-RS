//! BLE Beacon Example
//!
//! Demonstrates non-connectable BLE advertising (beacon mode) using BGAPI.
//!
//! Hardware: XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
//!
//! # What This Example Shows
//!
//! - Initializing the HAL clock system
//! - Starting the Bluetooth stack (BGAPI)
//! - Creating an advertising set
//! - Configuring advertising data (device name, manufacturer data)
//! - Starting non-connectable advertising
//! - BLE event loop handling
//!
//! # Beacon Behavior
//!
//! Once running, the device will:
//! - Advertise as "XIAO Beacon"
//! - Broadcast every 100ms (configurable)
//! - Be visible in BLE scanner apps (nRF Connect, LightBlue)
//! - NOT accept connections (beacon mode)
//!
//! # Expected Power Consumption
//!
//! - Active advertising: ~5-8 mA @ 3.3V
//! - TX power at 0 dBm
//!
//! # Scanning for This Beacon
//!
//! Use a BLE scanner app:
//! - iOS: LightBlue
//! - Android: nRF Connect
//! - Desktop: nRF Connect Desktop
//!
//! You should see "XIAO Beacon" in the device list.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m_rt::entry;
use efr32mg24_bluetooth_sys as bt;
use efr32mg24_hal::{
    clock::{ClockConfig, Clocks, HfxoConfig},
    pac,
    prelude::*, // Import embedded-hal traits
};

// Panic handler (required for no_std)
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}

#[entry]
fn main() -> ! {
    // ========================================================================
    // Step 1: Initialize Hardware (HAL Layer)
    // ========================================================================

    // Get peripheral singletons
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks for XIAO MG24 (39 MHz HFXO crystal)
    let (clocks, cmu) = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)), // 39 MHz crystal
            lfxo: Some(Default::default()),          // 32.768 kHz crystal
        },
    )
    .expect("Clock configuration failed");

    // Freeze clocks to get FrozenClocks (required for Delay)
    let frozen_clocks = clocks.freeze(cmu);

    // Create system delay for initialization timing
    let mut delay = efr32mg24_hal::delay::Delay::new(cp.SYST, &frozen_clocks);

    // ========================================================================
    // Step 2: Initialize Bluetooth Stack (BGAPI)
    // ========================================================================

    // Start the Bluetooth stack
    // This initializes the BLE controller and host layers
    unsafe {
        let status = bt::sl_bt_system_start_bluetooth();
        if status != 0 {
            // Bluetooth initialization failed
            // In production, handle this error appropriately
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Small delay to allow stack initialization
    delay.delay_ms(100);

    // ========================================================================
    // Step 3: Create Advertising Set
    // ========================================================================

    let mut advertising_set: u8 = 0;

    unsafe {
        // Create an advertising set
        // Returns a handle we'll use for all advertising operations
        let status = bt::sl_bt_advertiser_create_set(&mut advertising_set as *mut u8);
        if status != 0 {
            // Failed to create advertising set
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // ========================================================================
    // Step 4: Configure Advertising Data
    // ========================================================================

    // Device name: "XIAO Beacon"
    // This appears in BLE scanner apps
    let device_name = b"XIAO Beacon";

    // Build advertising data packet (GAP AD format)
    // Format: [length, AD type, data]
    let mut adv_data = [0u8; 31]; // Max advertising data size
    let mut offset = 0;

    // Flags field (required for all advertising)
    // 0x06 = General Discoverable + BR/EDR Not Supported
    adv_data[offset] = 2; // Length
    adv_data[offset + 1] = 0x01; // AD Type: Flags
    adv_data[offset + 2] = 0x06; // Value: General Discoverable, LE only
    offset += 3;

    // Complete Local Name field
    adv_data[offset] = (device_name.len() + 1) as u8; // Length
    adv_data[offset + 1] = 0x09; // AD Type: Complete Local Name
    adv_data[offset + 2..offset + 2 + device_name.len()].copy_from_slice(device_name);
    offset += device_name.len() + 2;

    // Manufacturer Specific Data (optional)
    // Company ID: 0xFFFF (reserved for testing)
    // Custom data: [0x42, 0x00] (example: beacon ID)
    adv_data[offset] = 4; // Length
    adv_data[offset + 1] = 0xFF; // AD Type: Manufacturer Specific Data
    adv_data[offset + 2] = 0xFF; // Company ID LSB
    adv_data[offset + 3] = 0xFF; // Company ID MSB
    adv_data[offset + 4] = 0x42; // Custom data byte 1
    offset += 5;

    let adv_data_len = offset;

    // Set advertising data
    unsafe {
        let status = bt::sl_bt_advertiser_set_data(
            advertising_set,
            0, // Packet type: 0 = Advertising data
            adv_data_len,
            adv_data.as_ptr(),
        );
        if status != 0 {
            // Failed to set advertising data
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // ========================================================================
    // Step 5: Configure Advertising Parameters
    // ========================================================================

    // Set advertising timing
    // interval_min/max: 160 = 100ms (units of 0.625ms)
    // More frequent = more visible but higher power consumption
    unsafe {
        let status = bt::sl_bt_advertiser_set_timing(
            advertising_set,
            160, // interval_min (100ms)
            160, // interval_max (100ms)
            0,   // duration (0 = continuous)
            0,   // max_events (0 = unlimited)
        );
        if status != 0 {
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Set TX power to 0 dBm (balanced range/power)
    unsafe {
        let mut set_power: i16 = 0;
        let status = bt::sl_bt_advertiser_set_tx_power(
            advertising_set,
            0,                          // TX power: 0 dBm
            &mut set_power as *mut i16, // Actual power set (output)
        );
        if status != 0 {
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // ========================================================================
    // Step 6: Start Advertising
    // ========================================================================

    unsafe {
        // Start advertising
        // discover: 2 = Non-discoverable (no scan response)
        // connect: 0 = Non-connectable (beacon mode)
        let status = bt::sl_bt_advertiser_start(
            advertising_set,
            2, // Non-discoverable (general advertising)
            0, // Non-connectable
        );
        if status != 0 {
            // Failed to start advertising
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Beacon is now broadcasting!
    // The device will appear as "XIAO Beacon" in BLE scanner apps

    // ========================================================================
    // Step 7: Event Loop
    // ========================================================================

    // In a real application, you would:
    // - Process BLE events (connections, reads, writes)
    // - Update advertising data dynamically
    // - Handle sensor readings, button presses, etc.
    //
    // For this beacon example, we just keep advertising indefinitely

    loop {
        // Wait for interrupt (low power mode)
        // The BLE stack runs in the background via interrupts
        cortex_m::asm::wfi();

        // Optional: Update advertising data periodically
        // Example: increment counter, update sensor reading, etc.
    }
}
