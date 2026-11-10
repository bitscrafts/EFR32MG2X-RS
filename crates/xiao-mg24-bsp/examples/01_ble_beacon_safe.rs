//! BLE Beacon Example (Safe Wrapper)
//!
//! Demonstrates non-connectable BLE advertising using safe XiaoBle wrapper.
//!
//! Hardware: XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
//!
//! # What This Example Shows
//!
//! - Using the safe `XiaoBle` wrapper instead of raw BGAPI FFI
//! - Creating a beacon with just 3 lines of code
//! - 80% code reduction: 270 lines (unsafe FFI) → ~50 lines (safe wrapper)
//!
//! # Beacon Behavior
//!
//! Once running, the device will:
//! - Advertise as "XIAO Beacon"
//! - Broadcast every 100ms
//! - Be visible in BLE scanner apps (nRF Connect, LightBlue)
//! - NOT accept connections (beacon mode)
//!
//! # Scanning for This Beacon
//!
//! Use a BLE scanner app:
//! - iOS: LightBlue
//! - Android: nRF Connect
//! - Desktop: nRF Connect Desktop
//!
//! You should see "XIAO Beacon" in the device list.
//!
//! # Code Comparison
//!
//! **Unsafe FFI version** (efr32mg24-bluetooth-sys/examples/01_ble_beacon.rs):
//! - 270 lines of code
//! - Manual GAP AD format encoding (30-40 lines)
//! - Raw BGAPI FFI calls with error handling
//! - Verbose advertising setup
//!
//! **Safe wrapper version** (this file):
//! - ~50 lines of code (80% reduction!)
//! - Automatic GAP AD encoding
//! - Type-safe error handling
//! - Single-line beacon creation

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m_rt::entry;
use xiao_mg24_bsp::radio::ble::XiaoBle;
use xiao_mg24_bsp::Board;

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
    // Step 1: Initialize Board
    // ========================================================================

    // Get board singleton (initializes clocks, GPIO, etc.)
    let board = Board::take().unwrap();

    // Create delay provider from board
    let mut delay = board.into_delay();

    // ========================================================================
    // Step 2: Create BLE Beacon (Safe Wrapper - 3 Lines!)
    // ========================================================================

    // This single line does what took 200+ lines in the unsafe FFI version:
    // - Initializes Bluetooth stack
    // - Creates advertising set
    // - Sets device name
    // - Encodes GAP AD format
    // - Configures advertising parameters
    // - Starts non-connectable advertising
    let _ble = XiaoBle::beacon(&mut delay, "XIAO Beacon").expect("Failed to create BLE beacon");

    // Beacon is now broadcasting!
    // The device will appear as "XIAO Beacon" in BLE scanner apps

    // ========================================================================
    // Step 3: Event Loop
    // ========================================================================

    // In a real application, you could:
    // - Update advertising data dynamically
    // - Handle sensor readings
    // - Implement low-power sleep modes
    //
    // For a simple beacon, we just sleep and let the Bluetooth stack
    // handle advertising in the background.

    loop {
        cortex_m::asm::wfi(); // Wait for interrupt (low power mode)
    }
}
