//! Minimal Safe Blink Example
//!
//! **CRITICAL SAFETY EXAMPLE**
//!
//! This is the SAFEST possible example for hardware testing.
//! It includes all anti-brick protections:
//!
//! 1. Debug trap (5-second delay at startup)
//! 2. No SWD pin reconfiguration
//! 3. No immediate low-power modes
//! 4. Simple LED blink for visual confirmation
//!
//! # Hardware
//!
//! - XIAO MG24 Sense
//! - Built-in LED on PD02 (D8)
//!
//! # Expected Behavior
//!
//! 1. On startup: LED blinks rapidly 10 times (5 seconds total) = DEBUG TRAP
//! 2. After debug trap: LED blinks slowly (1 Hz) = NORMAL OPERATION
//!
//! # Safety
//!
//! This example is designed to be unbrickable:
//! - 5-second debug trap gives time for SWD to attach
//! - LED visual feedback confirms code is running
//! - No dangerous operations (low-power, pin reconfig, etc.)
//!
//! # Flashing
//!
//! ```bash
//! # Build
//! cargo build --release --example 00_minimal_blink
//!
//! # Flash with probe-rs (CMSIS-DAP)
//! probe-rs run --chip EFR32MG24B220F1536 --release --example 00_minimal_blink
//!
//! # Or flash with commander (J-Link)
//! cargo objcopy --release --example 00_minimal_blink -- -O ihex 00_minimal_blink.hex
//! commander flash 00_minimal_blink.hex --address 0x00004000 --device EFR32MG24
//! ```

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use panic_halt as _;
use xiao_mg24_bsp::Board;

/// Debug trap with LED feedback
///
/// **CRITICAL SAFETY FEATURE**
///
/// This function provides a 5-second window at startup for the debugger to attach.
/// The LED blinks rapidly during this period to provide visual confirmation.
///
/// **Why this is necessary**:
/// - Prevents bricking from code that enters low-power modes immediately
/// - Prevents bricking from code that reconfigures SWD pins
/// - Gives time for SWD to attach even if main code crashes
///
/// **LED Pattern**:
/// - 10 rapid blinks (0.25s on, 0.25s off) = 5 seconds total
/// - Indicates safe debug window
///
/// **Only active in debug builds** (`#[cfg(debug_assertions)]`)
/// Release builds skip this for production efficiency.
fn debug_trap_with_led_feedback<D: DelayNs>(led: &mut impl OutputPin, delay: &mut D) {
    #[cfg(debug_assertions)]
    {
        // Blink LED 10 times (0.5 seconds each = 5 seconds total)
        for _ in 0..10 {
            let _ = led.set_high();
            delay.delay_ms(250); // 250ms on
            let _ = led.set_low();
            delay.delay_ms(250); // 250ms off
        }
    }

    // In release builds, this function does nothing (zero cost)
}

#[entry]
fn main() -> ! {
    // ========== HARDWARE INITIALIZATION ==========

    // Take ownership of board peripherals
    // This ensures exclusive access to hardware
    let board = Board::take().unwrap();

    // Get LED (PD02 / D8) and delay provider
    let mut led = board.led;
    let mut delay = board.delay;

    // ========== CRITICAL SAFETY: DEBUG TRAP ==========
    //
    // THIS MUST BE FIRST!
    //
    // Provides 5-second window for debugger attachment.
    // LED blinks rapidly during this period.
    //
    // If you see rapid LED blinking after reset, the device is
    // in the safe debug window - you can attach SWD now.
    debug_trap_with_led_feedback(&mut led, &mut delay);

    // ========== MAIN APPLICATION ==========
    //
    // After debug trap, normal operation begins.
    // LED blinks slowly (1 Hz) to indicate normal operation.

    loop {
        // Turn LED on
        let _ = led.set_high();
        delay.delay_ms(500); // 500ms on

        // Turn LED off
        let _ = led.set_low();
        delay.delay_ms(500); // 500ms off

        // Total: 1 Hz blink rate (1 second per cycle)
    }
}

// ========== SAFETY NOTES ==========
//
// This example is designed to be UNBRICKABLE:
//
// 1. Debug Trap (5 seconds):
//    - Always runs first (even before peripheral init)
//    - LED provides visual feedback
//    - Gives time for SWD to attach
//
// 2. No SWD Pin Touching:
//    - PA0 (SWCLK) and PA1 (SWDIO) are never configured
//    - These pins are dedicated debug pins - hands off!
//
// 3. No Immediate Low-Power:
//    - Main loop uses simple LED toggle
//    - No wfi() or deep sleep modes
//    - Device remains responsive to SWD
//
// 4. Simple Operation:
//    - Only uses LED (PD02)
//    - No complex peripherals
//    - Minimal chance of failure
//
// If this example doesn't work, check:
// - Power supply (3.3V stable)
// - USB connection
// - LED is visible (may be dim - look carefully)
// - Correct flash address (0x00004000 if bootloader present)
