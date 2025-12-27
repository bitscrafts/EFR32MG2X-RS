//! Watchdog Timer (WDOG) Example
//!
//! This example demonstrates the Watchdog Timer functionality including:
//! - Basic watchdog configuration
//! - Feeding (clearing) the watchdog
//! - Lock mechanism
//! - Warning mode (timeout without reset)
//! - Energy mode support
//!
//! # Hardware Requirements
//!
//! - XIAO MG24 Sense board
//! - LFXO/LFRCO for watchdog clock (automatically configured)
//!
//! # Expected Behavior
//!
//! 1. Configure watchdog with 2 second timeout
//! 2. Feed watchdog periodically to prevent reset
//! 3. Demonstrate lock mechanism
//! 4. Show warning mode (timeout without reset)
//! 5. Final test: intentional timeout causes device reset
//!
//! # Build and Flash
//!
//! ```bash
//! cargo build --example 12_wdog --features rt --release
//! cargo run --example 12_wdog --features rt --release
//! ```
//!
//! **Warning**: This example will cause a watchdog reset at the end!

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_hal::{
    clock::{ClockConfig, Clocks, HfxoConfig, LfxoConfig},
    delay::Delay,
    pac,
    prelude::*,
    wdog::{TimeoutPeriod, Wdog, WdogConfig},
};
use panic_halt as _;

#[entry]
fn main() -> ! {
    // Get peripheral singletons
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // SWD Safety: Keep SWD interface enabled in debug builds
    #[cfg(debug_assertions)]
    unsafe {
        // Keep HFXO enabled for SWD clock
        dp.cmu_s.clken0().modify(|_, w| w.hfxo0().set_bit());

        // Keep SWD interface enabled - PREVENTS BRICKING
    }

    // Configure clocks with XIAO MG24's 39 MHz HFXO and 32.768 kHz LFXO
    let (clocks, cmu) = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)),
            lfxo: Some(LfxoConfig::default()), // Enable LFXO for watchdog
        },
    )
    .expect("Clock configuration failed");

    let frozen_clocks = clocks.freeze(cmu);

    // Create delay provider
    let mut delay = Delay::new(cp.SYST, &frozen_clocks);

    // ═══════════════════════════════════════════════════════════════
    // Example 1: Basic Watchdog with Feeding
    // ═══════════════════════════════════════════════════════════════

    // Create configuration: 2 second timeout, runs only in EM0
    let config = WdogConfig::new(TimeoutPeriod::Cycles2k); // ~2 seconds at 1kHz

    // Enable watchdog
    let mut wdog = Wdog::new(dp.wdog0_s, config).expect("Failed to enable watchdog");

    // Feed watchdog multiple times to demonstrate normal operation
    for _ in 0..5 {
        delay.delay_ms(500); // Wait 500ms
        wdog.feed(); // Feed watchdog (resets timeout counter)
                     // No timeout because we're feeding every 500ms (< 2 second timeout)
    }

    // ═══════════════════════════════════════════════════════════════
    // Example 2: Check Watchdog Status
    // ═══════════════════════════════════════════════════════════════

    // Verify watchdog is enabled
    let is_enabled = wdog.is_enabled();
    let _ = is_enabled; // Use variable (prevents warning)

    // Check lock status (should be unlocked at this point)
    let is_locked = wdog.is_locked();
    let _ = is_locked; // Use variable

    // ═══════════════════════════════════════════════════════════════
    // Example 3: Lock Mechanism
    // ═══════════════════════════════════════════════════════════════

    // In production, you would lock the watchdog after configuration
    // to prevent malicious or accidental changes
    wdog.lock();

    // After locking, configuration cannot be changed until device reset
    let now_locked = wdog.is_locked();
    let _ = now_locked; // Should be true

    // Feeding still works when locked (that's the whole point!)
    wdog.feed();

    // ═══════════════════════════════════════════════════════════════
    // Example 4: Intentional Timeout (Final Test)
    // ═══════════════════════════════════════════════════════════════

    // **Warning**: This will cause a watchdog reset!
    //
    // We stop feeding the watchdog and wait for timeout.
    // After ~2 seconds, the device will reset.
    //
    // In a real application, a timeout indicates:
    // - Software hang/crash
    // - Infinite loop
    // - Deadlock
    // - Lost interrupt
    //
    // The watchdog reset recovers the system.

    delay.delay_ms(100); // Small delay before timeout

    // Stop feeding watchdog - let it timeout and reset
    loop {
        // Infinite loop without feeding
        // Watchdog will reset the device after ~2 seconds
        delay.delay_ms(100);

        // In a real application, this would be replaced with:
        // - Main event loop with periodic watchdog feeding
        // - Task scheduler that feeds watchdog only when healthy
        // - Safety-critical code that feeds watchdog after checks
    }

    // NOTE: Code never reaches here because watchdog resets the device
}

// ═══════════════════════════════════════════════════════════════
// Alternative Example Configurations (commented out)
// ═══════════════════════════════════════════════════════════════

#[allow(dead_code)]
fn example_run_in_all_modes() {
    // Watchdog continues running even in low-power modes
    let _config = WdogConfig::new(TimeoutPeriod::Cycles8k) // ~8 seconds
        .run_in_all_modes(); // Runs in EM0, EM1, EM2, EM3

    // Useful for battery-powered applications that sleep
    // but still need watchdog protection
}

#[allow(dead_code)]
fn example_debug_friendly() {
    // Watchdog freezes when debugger is attached
    let _config = WdogConfig::new(TimeoutPeriod::Cycles2k).debug_run(false); // Freeze in debug mode (default)

    // This prevents nuisance resets during debugging
}

#[allow(dead_code)]
fn example_warning_mode() {
    // Warning-only mode: timeout sets interrupt flag but doesn't reset
    let _config = WdogConfig::new(TimeoutPeriod::Cycles1k).warning_only(); // Disable reset on timeout

    // Useful for testing/development:
    // - Check if timeouts would occur in production
    // - Log timeout events for diagnostics
    // - Graceful degradation instead of hard reset
}

#[allow(dead_code)]
fn example_long_timeout() {
    // Very long timeout for slow operations
    let _config = WdogConfig::new(TimeoutPeriod::Cycles256k); // ~4.3 minutes

    // Use cases:
    // - Firmware updates (don't timeout during flash erase)
    // - Sensor calibration (may take minutes)
    // - Network operations (long timeouts acceptable)
}

#[allow(dead_code)]
fn example_short_timeout() {
    // Very short timeout for fast response
    let _config = WdogConfig::new(TimeoutPeriod::Cycles129); // ~129ms

    // Use cases:
    // - Safety-critical systems (fail fast)
    // - High-frequency control loops (quick recovery)
    // - Testing watchdog functionality
}

// ═══════════════════════════════════════════════════════════════
// Production Usage Pattern
// ═══════════════════════════════════════════════════════════════

#[allow(dead_code)]
fn production_pattern_example() {
    // 1. Configure watchdog early in main()
    // 2. Lock configuration to prevent tampering
    // 3. Feed watchdog in main loop or task scheduler
    // 4. Only feed after health checks pass

    // Example health checks before feeding:
    fn system_health_ok() -> bool {
        // Check critical invariants:
        // - Stack integrity
        // - Heap integrity
        // - Communication alive
        // - Sensors responding
        // - No error flags set
        true // Placeholder
    }

    fn main_loop() {
        loop {
            // Do work...

            // Only feed watchdog if system is healthy
            if system_health_ok() {
                // wdog.feed();
            }

            // If unhealthy, let watchdog reset the system
            // This is safer than running with errors
        }
    }

    let _ = main_loop; // Avoid unused warning
}
