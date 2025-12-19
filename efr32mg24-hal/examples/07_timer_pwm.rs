//! Timer PWM LED Brightness Control Example
//!
//! This example demonstrates PWM generation using TIMER0 on the EFR32MG24.
//! It uses PWM to control LED brightness, cycling through different duty cycles.
//!
//! # Hardware Setup
//!
//! - Target: Seeed Studio XIAO MG24 Sense
//! - LED: PB2 (onboard LED) connected to TIMER0 CC0 output
//! - Crystal: 39 MHz HFXO
//!
//! # Pin Routing
//!
//! To use PWM output, you must configure GPIO pin routing:
//! - Configure PB2 as TIMER0_CC0 output via GPIO alternate function
//! - See GPIO module documentation for pin routing configuration
//!
//! # Expected Behavior
//!
//! - LED brightness smoothly cycles from 0% to 100% and back
//! - PWM frequency: 10 kHz (imperceptible flicker to human eye)
//! - Cycle period: ~5 seconds per full brightness sweep
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
    timer::{Config, PwmChannel, PwmMode, Timer0},
};
use panic_halt as _;

#[entry]
fn main() -> ! {
    // Get peripheral singletons
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks with XIAO MG24's 39 MHz crystal
    let clocks = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)),
            lfxo: Some(Default::default()),
        },
    )
    .freeze();

    // Create delay provider for timing control
    let mut delay = Delay::new(cp.SYST, &clocks);

    // Configure TIMER0 for PWM at 10 kHz
    // PWM frequency = 10 kHz provides smooth LED dimming without visible flicker
    let mut timer = Timer0::new(
        dp.timer0_s,
        Config::new(10_000).with_pwm(PwmMode::EdgeAligned),
        &clocks,
    );

    // Enable PWM output on channel 0
    // Note: GPIO pin routing must be configured separately to connect
    // TIMER0_CC0 output to PB2 (LED pin)
    timer.enable_channel(PwmChannel::Channel0);

    // Start the timer
    timer.start();

    // Main loop: cycle LED brightness using PWM duty cycle
    loop {
        // Fade in: 0% ’ 100% brightness
        for duty in 0..=100u8 {
            timer
                .set_duty_cycle(PwmChannel::Channel0, duty)
                .unwrap();
            delay.delay_ms(25); // 25ms per step = 2.5s total fade-in
        }

        // Fade out: 100% ’ 0% brightness
        for duty in (0..=100u8).rev() {
            timer
                .set_duty_cycle(PwmChannel::Channel0, duty)
                .unwrap();
            delay.delay_ms(25); // 25ms per step = 2.5s total fade-out
        }
    }
}

// Example of raw duty cycle control for precise values
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

// Example of multiple PWM channels
#[allow(dead_code)]
fn demonstrate_multi_channel_pwm(timer: &mut Timer0) {
    // Enable all 3 PWM channels
    timer.enable_channel(PwmChannel::Channel0);
    timer.enable_channel(PwmChannel::Channel1);
    timer.enable_channel(PwmChannel::Channel2);

    // Set different duty cycles for RGB LED control
    timer.set_duty_cycle(PwmChannel::Channel0, 50).unwrap(); // Red: 50%
    timer.set_duty_cycle(PwmChannel::Channel1, 75).unwrap(); // Green: 75%
    timer.set_duty_cycle(PwmChannel::Channel2, 25).unwrap(); // Blue: 25%
}
