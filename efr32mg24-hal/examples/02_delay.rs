//! Delay Usage Example
//!
//! This example demonstrates how to use the blocking delay functions
//! provided by the HAL's Delay module, which uses the ARM Cortex-M SysTick timer.
//!
//! Target: XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
//!
//! # Hardware Setup
//!
//! No external connections required for this example.
//!
//! In a real application, you might connect an LED to see the timing:
//! - LED on PB2 (built-in LED on XIAO MG24)
//!
//! # Expected Behavior
//!
//! The example performs various delay operations with different time scales.
//! With an LED connected, you would see periodic blinking.
//! With a logic analyzer, you can measure the actual delay timing.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_hal::{
    clock::{ClockConfig, Clocks, HfxoConfig, LfxoConfig},
    delay::Delay,
    pac,
    prelude::*,
};
use panic_halt as _;

#[entry]
fn main() -> ! {
    // Get peripheral access
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks with external crystals (XIAO MG24)
    // Using 39 MHz HFXO provides accurate timing for delays
    let clocks = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)),
            lfxo: Some(LfxoConfig::default()),
        }
    ).freeze();

    // Create delay provider using SysTick timer
    let mut delay = Delay::new(cp.SYST, &clocks);

    // Example 1: Millisecond delays
    // Suitable for LED blinking, debouncing, etc.
    delay.delay_ms(1000); // 1 second delay
    delay.delay_ms(500);  // 500 milliseconds

    // Example 2: Microsecond delays
    // Suitable for protocol timing, sensor communication
    delay.delay_us(1000); // 1 millisecond (1000 microseconds)
    delay.delay_us(100);  // 100 microseconds
    delay.delay_us(10);   // 10 microseconds

    // Example 3: Nanosecond delays
    // Suitable for precise timing, bit-banging protocols
    // Note: Very short delays use approximate NOP loop
    delay.delay_ns(1_000_000); // 1 millisecond (1,000,000 nanoseconds)
    delay.delay_ns(10_000);    // 10 microseconds
    delay.delay_ns(500);       // 500 nanoseconds

    // Example 4: Using in a loop (LED blink pattern)
    // In a real application with GPIO:
    // let mut led = gpio.pb2.into_push_pull_output();
    loop {
        // Toggle LED on
        // led.set_high().unwrap();
        delay.delay_ms(200);

        // Toggle LED off
        // led.set_low().unwrap();
        delay.delay_ms(200);

        // Short pulse pattern
        for _ in 0..3 {
            // led.set_high().unwrap();
            delay.delay_ms(50);
            // led.set_low().unwrap();
            delay.delay_ms(50);
        }

        // Long pause
        delay.delay_ms(1000);
    }
}

// Notes on Delay Usage:
//
// 1. Blocking Behavior:
//    - All delays are blocking - CPU waits during delay
//    - Not suitable for low-power applications
//    - Interrupts can still fire during delays (if enabled)
//
// 2. Timing Accuracy:
//    - Millisecond delays: Very accurate (depends on clock)
//    - Microsecond delays: Accurate for delays > 10 us
//    - Nanosecond delays: Approximate for very short delays (< 1 us)
//
// 3. Best Practices:
//    - Use delay_ms() for general timing (debouncing, LED blinking)
//    - Use delay_us() for protocol timing (I2C, SPI bit delays)
//    - Use delay_ns() only when precise sub-microsecond timing needed
//    - Consider using timers for non-blocking timing
//    - Consider sleep modes for long delays in battery-powered devices
//
// 4. Maximum Delays:
//    - No practical limit due to automatic chunking
//    - For very long delays, consider:
//      * Using sleep modes for power savings
//      * Using RTC for accurate long-term timing
//      * Using timers for non-blocking operation
//
// 5. Alternative Approaches:
//    - For non-blocking: Use Timer peripherals with interrupts
//    - For low power: Use RTC with sleep modes (EM2/EM3)
//    - For multiple timings: Use hardware timers (TIMER0-4)
//    - For watchdog: Use WDOG peripheral
//
// 6. Common Use Cases:
//    - Sensor initialization delays
//    - Communication protocol timing
//    - LED patterns and indicators
//    - Button debouncing
//    - Power-up sequence timing
//    - Display refresh timing
