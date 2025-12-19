//! Timer PWM Comprehensive Example
//!
//! This example demonstrates PWM generation using TIMER0 on the EFR32MG24.
//! It showcases multiple PWM features including multi-channel output, dynamic duty
//! cycle control, and LED brightness animation.
//!
//! # Hardware Setup
//!
//! - Target: Seeed Studio XIAO MG24 Sense
//! - Crystal: 39 MHz HFXO
//! - PWM Outputs (configure GPIO routing):
//!   - CH0: PB2 (onboard LED) - animated brightness fade
//!   - CH1: Optional external LED - 50% static duty cycle
//!   - CH2: Optional external LED - 75% static duty cycle
//!
//! # Pin Routing
//!
//! To use PWM outputs, configure GPIO pin routing:
//! - PB2 as TIMER0_CC0 output via GPIO alternate function
//! - See GPIO module documentation for pin routing configuration
//!
//! # Features Demonstrated
//!
//! 1. Timer initialization with PWM mode (edge-aligned)
//! 2. Multiple PWM channels on a single timer
//! 3. Dynamic duty cycle updates (Channel 0 fade animation)
//! 4. Static duty cycles (Channels 1-2)
//! 5. Helper functions for advanced usage patterns
//!
//! # Expected Behavior
//!
//! - Channel 0: LED brightness smoothly cycles 0% -> 100% -> 0%
//!   - PWM frequency: 10 kHz (imperceptible flicker)
//!   - Cycle period: ~5 seconds per full sweep
//! - Channel 1: Static 50% duty cycle (5 µs high, 5 µs low)
//! - Channel 2: Static 75% duty cycle (7.5 µs high, 2.5 µs low)
//!
//! # Build & Run
//!
//! ```bash
//! cargo build --example 07_timer_pwm --features rt --release
//! # Flash to device with probe-rs
//! cargo run --example 07_timer_pwm --features rt --release
//! ```

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_hal::{
    clock::{ClockConfig, Clocks, HfxoConfig},
    delay::Delay,
    pac,
    prelude::*,
    timer::{Config, PwmChannel, PwmMode, Timer0},
};
use panic_halt as _;

#[entry]
fn main() -> ! {
    // Get peripheral singletons
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks with XIAO MG24's 39 MHz crystal
    let (clocks, cmu) = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)),
            lfxo: Some(Default::default()),
        },
    )
    .expect("Clock configuration failed");

    let frozen_clocks = clocks.freeze(cmu);

    // Create delay provider for timing control
    let mut delay = Delay::new(cp.SYST, &frozen_clocks);

    // Configure TIMER0 for PWM at 10 kHz
    // PWM frequency = 10 kHz provides smooth LED dimming without visible flicker
    let mut timer = Timer0::new(
        dp.timer0_s,
        Config::new(10_000).with_pwm(PwmMode::EdgeAligned),
        &frozen_clocks,
    );

    // Configure Channel 0: Dynamic duty cycle (animated)
    timer.enable_channel(PwmChannel::Channel0);

    // Configure Channel 1: Static 50% duty cycle
    timer.set_duty_cycle(PwmChannel::Channel1, 50).unwrap();
    timer.enable_channel(PwmChannel::Channel1);

    // Configure Channel 2: Static 75% duty cycle
    timer.set_duty_cycle(PwmChannel::Channel2, 75).unwrap();
    timer.enable_channel(PwmChannel::Channel2);

    // Start the timer
    timer.start();

    // Main loop: animate Channel 0 brightness while Channels 1-2 stay static
    loop {
        // Fade in: 0% -> 100% brightness on Channel 0
        for duty in 0..=100u8 {
            timer.set_duty_cycle(PwmChannel::Channel0, duty).unwrap();
            delay.delay_ms(25); // 25ms per step = 2.5s total fade-in
        }

        // Fade out: 100% -> 0% brightness on Channel 0
        for duty in (0..=100u8).rev() {
            timer.set_duty_cycle(PwmChannel::Channel0, duty).unwrap();
            delay.delay_ms(25); // 25ms per step = 2.5s total fade-out
        }

        // Channels 1 and 2 continue at their static duty cycles
    }
}

// Example of raw duty cycle control for precise values
// Useful when you need exact timing beyond 1% resolution
#[allow(dead_code)]
fn demonstrate_raw_duty_control(timer: &mut Timer0) {
    let top = timer.get_top_value();

    // Set exact 33.3% duty cycle using raw compare value
    let duty_33_percent = top / 3;
    timer
        .set_duty_raw(PwmChannel::Channel0, duty_33_percent)
        .unwrap();

    // Set exact 66.6% duty cycle
    let duty_66_percent = (top * 2) / 3;
    timer
        .set_duty_raw(PwmChannel::Channel0, duty_66_percent)
        .unwrap();
}

// Example of interrupt-driven PWM updates
// Useful for updating PWM in background without blocking
#[allow(dead_code)]
fn demonstrate_interrupt_usage(timer: &mut Timer0) {
    // Enable overflow interrupt
    timer.listen_overflow();

    // In interrupt handler (TIMER0_IRQHandler):
    // if timer.is_overflow() {
    //     timer.clear_overflow();
    //     // Update PWM duty cycle or perform periodic task
    // }
}

// Example of RGB LED control with three PWM channels
// Each channel can independently control red, green, or blue intensity
#[allow(dead_code)]
fn demonstrate_rgb_led_control(timer: &mut Timer0) {
    // Enable all 3 PWM channels
    timer.enable_channel(PwmChannel::Channel0);
    timer.enable_channel(PwmChannel::Channel1);
    timer.enable_channel(PwmChannel::Channel2);

    // Set different duty cycles for RGB LED control
    timer.set_duty_cycle(PwmChannel::Channel0, 50).unwrap(); // Red: 50%
    timer.set_duty_cycle(PwmChannel::Channel1, 75).unwrap(); // Green: 75%
    timer.set_duty_cycle(PwmChannel::Channel2, 25).unwrap(); // Blue: 25%
                                                             // Result: Cyan-ish color (more green, some red, less blue)
}

// Example of smooth color transitions for RGB LED
// Demonstrates coordinated multi-channel duty cycle updates
#[allow(dead_code)]
fn demonstrate_rgb_fade(timer: &mut Timer0, delay: &mut Delay) {
    // Fade from red to green
    for step in 0..=100u8 {
        timer
            .set_duty_cycle(PwmChannel::Channel0, 100 - step)
            .unwrap(); // Red decreases
        timer.set_duty_cycle(PwmChannel::Channel1, step).unwrap(); // Green increases
        timer.set_duty_cycle(PwmChannel::Channel2, 0).unwrap(); // Blue stays off
        delay.delay_ms(20); // 2 second transition
    }
}

// Example of motor speed control with PWM
// Shows how to map RPM to duty cycle
#[allow(dead_code)]
fn demonstrate_motor_control(timer: &mut Timer0) {
    // Motor specifications: 1000 RPM at 50% duty, 2000 RPM at 100%
    let target_rpm: u16 = 1500; // Target: 1500 RPM

    // Map RPM to duty cycle: 1000 RPM = 50%, 2000 RPM = 100%
    // Linear interpolation: duty = 50 + (rpm - 1000) / 20
    let duty = if target_rpm <= 1000 {
        50 // Minimum speed
    } else if target_rpm >= 2000 {
        100 // Maximum speed
    } else {
        50 + ((target_rpm - 1000) / 20) as u8
    };

    timer.set_duty_cycle(PwmChannel::Channel0, duty).unwrap();
}
