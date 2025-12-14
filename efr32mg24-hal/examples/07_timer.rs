//! Timer and PWM Example
//!
//! This example demonstrates how to use the Timer peripheral for PWM generation
//! on the EFR32MG24 microcontroller.
//!
//! # Hardware Setup
//!
//! Connect LEDs or observe with an oscilloscope:
//! - TIMER0 CH0: PWM output at 10 kHz with 25% duty cycle
//! - TIMER0 CH1: PWM output at 10 kHz with 50% duty cycle
//! - TIMER0 CH2: PWM output at 10 kHz with 75% duty cycle
//!
//! # Features Demonstrated
//!
//! 1. Timer initialization with PWM mode
//! 2. Multiple PWM channels on a single timer
//! 3. Different duty cycles per channel
//! 4. Edge-aligned PWM mode
//!
//! # Expected Behavior
//!
//! - Three PWM signals generated at 10 kHz frequency
//! - Channel 0: 25% duty cycle (2.5 µs high, 7.5 µs low)
//! - Channel 1: 50% duty cycle (5 µs high, 5 µs low)
//! - Channel 2: 75% duty cycle (7.5 µs high, 2.5 µs low)

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

use efr32mg24_hal::{
    clock::{ClockConfig, Clocks, HfxoConfig},
    pac,
    timer::{Config, PwmChannel, PwmMode, Timer0},
};

#[entry]
fn main() -> ! {
    // Get peripheral singletons
    let _cp = cortex_m::Peripherals::take().unwrap();
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

    // Create Timer0 with PWM mode at 10 kHz
    let mut timer = Timer0::new(
        dp.timer0_s,
        Config::new(10_000).with_pwm(PwmMode::EdgeAligned),
        &frozen_clocks,
    );

    // Configure PWM channels with different duty cycles
    // Channel 0: 25% duty cycle
    timer.set_duty_cycle(PwmChannel::Channel0, 25).unwrap();
    timer.enable_channel(PwmChannel::Channel0);

    // Channel 1: 50% duty cycle
    timer.set_duty_cycle(PwmChannel::Channel1, 50).unwrap();
    timer.enable_channel(PwmChannel::Channel1);

    // Channel 2: 75% duty cycle
    timer.set_duty_cycle(PwmChannel::Channel2, 75).unwrap();
    timer.enable_channel(PwmChannel::Channel2);

    // Start the timer to begin PWM generation
    timer.start();

    // PWM signals are now being generated
    // In a real application, you might:
    // - Adjust duty cycles dynamically
    // - Use multiple timers for different frequencies
    // - Control motor speed, LED brightness, etc.

    loop {
        // PWM generation continues in hardware
        // Could add dynamic duty cycle changes here
        cortex_m::asm::wfi(); // Wait for interrupt (low power)
    }
}
