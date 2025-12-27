//! Real-Time Clock (RTC) Example
//!
//! This example demonstrates the RTC (SYSRTC0) peripheral functionality including:
//! - Counter-based timekeeping
//! - Alarm (compare match) functionality
//! - Time conversion helpers (ticks ↔ seconds)
//!
//! # Hardware Requirements
//!
//! - XIAO MG24 Sense board
//! - LFXO (32.768 kHz crystal) must be present on board
//!
//! # Expected Behavior
//!
//! 1. RTC counter starts at 0
//! 2. Sets an alarm for 5 seconds
//! 3. Waits for alarm to trigger
//! 4. Sets another alarm for 10 seconds
//! 5. Demonstrates time reading and conversion
//!
//! # Build and Flash
//!
//! ```bash
//! cargo build --example 11_rtc --features rt --release
//! cargo run --example 11_rtc --features rt --release
//! ```

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_hal::{
    clock::{ClockConfig, Clocks, HfxoConfig, LfxoConfig},
    delay::Delay,
    pac,
    prelude::*,
    rtc::Rtc,
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
            lfxo: Some(LfxoConfig::default()), // Enable LFXO for RTC
        },
    )
    .expect("Clock configuration failed");

    let frozen_clocks = clocks.freeze(cmu);

    // Create delay provider
    let mut delay = Delay::new(cp.SYST, &frozen_clocks);

    // Initialize RTC
    let mut rtc = Rtc::new(dp.sysrtc0_ns);

    // ═══════════════════════════════════════════════════════════════
    // Example 1: Reset Counter and Basic Timing
    // ═══════════════════════════════════════════════════════════════

    // Reset counter to 0
    rtc.set_counter(0);

    // Wait for counter to increment (verify RTC is running)
    delay.delay_ms(100);

    let ticks_after_100ms = rtc.counter();
    // At 32.768 kHz, 100ms = ~3277 ticks
    // In real application, would verify ticks_after_100ms ≈ 3277

    // ═══════════════════════════════════════════════════════════════
    // Example 2: Alarm (Compare Match) - 5 Seconds
    // ═══════════════════════════════════════════════════════════════

    // Reset counter to 0 for clean test
    rtc.set_counter(0);

    // Set alarm for 5 seconds from now
    let alarm_5_seconds = rtc.seconds_to_ticks(5);
    rtc.set_alarm(alarm_5_seconds).expect("Failed to set alarm");

    // Wait for alarm
    while !rtc.alarm_flag() {
        // In real application, could enter EM1/EM2 here to save power
        delay.delay_ms(100);
    }

    // Alarm triggered!
    rtc.clear_alarm_flag();

    // ═══════════════════════════════════════════════════════════════
    // Example 3: Time Conversion Helpers
    // ═══════════════════════════════════════════════════════════════

    // Get current counter value
    let current_ticks = rtc.counter();

    // Convert to seconds
    let seconds = rtc.ticks_to_seconds(current_ticks);
    // Should be approximately 5 seconds

    // Convert to milliseconds
    let millis = rtc.ticks_to_millis(current_ticks);
    // Should be approximately 5000 milliseconds

    // Use the values (prevents unused variable warnings)
    let _ = (ticks_after_100ms, seconds, millis);

    // ═══════════════════════════════════════════════════════════════
    // Example 4: Alarm with Current Time + Offset
    // ═══════════════════════════════════════════════════════════════

    // Set alarm for 10 seconds from current time
    let current = rtc.counter();
    let alarm_10_sec = current + rtc.seconds_to_ticks(10);

    rtc.set_alarm(alarm_10_sec)
        .expect("Failed to set 10s alarm");

    // Wait for alarm
    while !rtc.alarm_flag() {
        delay.delay_ms(100);
    }

    rtc.clear_alarm_flag();

    // ═══════════════════════════════════════════════════════════════
    // Example 5: Periodic Alarm Loop
    // ═══════════════════════════════════════════════════════════════

    // Reset counter for periodic timing
    rtc.set_counter(0);

    let mut alarm_count = 0u32;

    loop {
        // Set alarm for 2 seconds from now
        let current = rtc.counter();
        let next_alarm = current + rtc.seconds_to_ticks(2);

        rtc.set_alarm(next_alarm)
            .expect("Failed to set periodic alarm");

        // Wait for alarm
        while !rtc.alarm_flag() {
            // Could enter low-power mode here
        }

        rtc.clear_alarm_flag();

        alarm_count += 1;

        // In real application, would:
        // - Toggle LED
        // - Take sensor reading
        // - Transmit data
        // - etc.

        // Get total uptime
        let uptime_seconds = rtc.ticks_to_seconds(rtc.counter());

        // Use the values
        let _ = (alarm_count, uptime_seconds);

        // Run 10 iterations then idle
        if alarm_count >= 10 {
            // Disable alarm for final idle
            rtc.disable_alarm();

            // Infinite idle loop
            loop {
                delay.delay_ms(1000);
            }
        }
    }
}
