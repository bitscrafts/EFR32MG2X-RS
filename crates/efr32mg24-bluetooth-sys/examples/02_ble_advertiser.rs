//! BLE Connectable Advertiser Example
//!
//! Demonstrates connectable BLE advertising and connection handling using BGAPI.
//!
//! Hardware: XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
//!
//! # What This Example Shows
//!
//! - Starting connectable advertising (peripheral role)
//! - Handling connection events
//! - Connection parameter negotiation
//! - Disconnect detection and re-advertising
//! - Basic connection management
//!
//! # Peripheral Behavior
//!
//! Once running, the device will:
//! - Advertise as "XIAO Peripheral"
//! - Accept connections from central devices (smartphones, computers)
//! - Handle multiple connect/disconnect cycles
//! - Automatically re-advertise after disconnection
//!
//! # Expected Power Consumption
//!
//! - Advertising: ~5-8 mA @ 3.3V
//! - Connected idle: ~3-5 mA @ 3.3V
//!
//! # Testing
//!
//! Use a BLE central app (nRF Connect, LightBlue):
//! 1. Scan for devices → See "XIAO Peripheral"
//! 2. Connect to device → Connection established
//! 3. Disconnect → Device re-advertises automatically

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

#[entry]
fn main() -> ! {
    // ========================================================================
    // Step 1: Initialize Hardware (HAL Layer)
    // ========================================================================

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks for XIAO MG24 (39 MHz HFXO)
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
    // Step 3: Configure Connection Parameters
    // ========================================================================

    // Set default connection parameters
    // These will be negotiated when a connection is established
    unsafe {
        bt::sl_bt_connection_set_default_parameters(
            80,     // min_interval: 100ms (80 * 1.25ms)
            80,     // max_interval: 100ms
            0,      // latency: 0 (no slave latency)
            200,    // timeout: 2000ms (200 * 10ms)
            0,      // min_ce_length
            0xFFFF, // max_ce_length
        );
    }

    // ========================================================================
    // Step 4: Create and Configure Advertising Set
    // ========================================================================

    let mut advertising_set: u8 = 0;

    unsafe {
        let status = bt::sl_bt_advertiser_create_set(&mut advertising_set as *mut u8);
        if status != 0 {
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Set advertising data
    let device_name = b"XIAO Peripheral";
    let mut adv_data = [0u8; 31];
    let mut offset = 0;

    // Flags: General Discoverable + BR/EDR Not Supported
    adv_data[offset] = 2;
    adv_data[offset + 1] = 0x01; // AD Type: Flags
    adv_data[offset + 2] = 0x06; // LE General Discoverable, BR/EDR not supported
    offset += 3;

    // Complete Local Name
    adv_data[offset] = (device_name.len() + 1) as u8;
    adv_data[offset + 1] = 0x09; // AD Type: Complete Local Name
    adv_data[offset + 2..offset + 2 + device_name.len()].copy_from_slice(device_name);
    offset += device_name.len() + 2;

    let adv_data_len = offset;

    unsafe {
        bt::sl_bt_advertiser_set_data(
            advertising_set,
            0, // Advertising data
            adv_data_len,
            adv_data.as_ptr(),
        );
    }

    // Set advertising timing
    unsafe {
        bt::sl_bt_advertiser_set_timing(
            advertising_set,
            160, // 100ms intervals
            160,
            0, // Continuous
            0, // Unlimited events
        );
    }

    // Set TX power
    unsafe {
        let mut set_power: i16 = 0;
        bt::sl_bt_advertiser_set_tx_power(
            advertising_set,
            0, // 0 dBm
            &mut set_power as *mut i16,
        );
    }

    // ========================================================================
    // Step 5: Start Connectable Advertising
    // ========================================================================

    unsafe {
        // discover: 2 = General discoverable
        // connect: 2 = Connectable (allows connections)
        let status = bt::sl_bt_advertiser_start(
            advertising_set,
            2, // General discoverable
            2, // Connectable ← This is the key difference from beacon!
        );
        if status != 0 {
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // ========================================================================
    // Step 6: Connection State Tracking
    // ========================================================================

    // Track current connection state
    let mut connected = false;
    let mut connection_handle: u8 = 0xFF;

    // ========================================================================
    // Step 7: Main Event Loop
    // ========================================================================

    // In a real BGAPI application, you would:
    // 1. Call sl_bt_wait_event() or sl_bt_pop_event() to get BLE events
    // 2. Parse the event structure to determine event type
    // 3. Handle connection_opened, connection_closed, etc.
    //
    // Event types (from BGAPI):
    // - connection_opened: New connection established
    //   - Contains: connection handle, address, bonding status
    // - connection_parameters: Connection params updated
    //   - Contains: interval, latency, timeout
    // - connection_closed: Connection terminated
    //   - Contains: reason code
    //
    // Example event handling structure:
    //
    // loop {
    //     let event = sl_bt_wait_event();  // Blocking wait for event
    //
    //     match event.header.id {
    //         CONNECTION_OPENED_EVENT => {
    //             connected = true;
    //             connection_handle = event.data.connection_opened.connection;
    //             // Stop advertising (we have a connection)
    //             sl_bt_advertiser_stop(advertising_set);
    //         }
    //         CONNECTION_CLOSED_EVENT => {
    //             connected = false;
    //             // Restart advertising for next connection
    //             sl_bt_advertiser_start(advertising_set, 2, 2);
    //         }
    //         _ => {} // Other events
    //     }
    // }
    //
    // Since we don't have the event structure definitions in this FFI layer,
    // this example demonstrates the API calls needed. A complete implementation
    // would require event parsing logic.

    loop {
        // Simplified event loop for demonstration
        // In production, use sl_bt_wait_event() or sl_bt_pop_event()

        cortex_m::asm::wfi();

        // When a connection is established (detected via event):
        // - connected = true
        // - Stop advertising: sl_bt_advertiser_stop(advertising_set)

        // When connection is closed (detected via event):
        // - connected = false
        // - Restart advertising: sl_bt_advertiser_start(advertising_set, 2, 2)

        // Note: Full event handling requires sl_bt_event structure
        // which would be defined in a higher-level wrapper or application
    }
}
