//! GPIO Usage Example
//!
//! This example demonstrates how to use the GPIO module for digital input/output
//! with type-safe pin configuration on the EFR32MG24.
//!
//! Target: XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
//!
//! # Hardware Setup
//!
//! Built-in hardware used:
//! - PB2: Built-in LED (active high)
//!
//! Optional connections for full demonstration:
//! - PB1: Connect a button to GND (will use internal pull-up)
//! - PA0: Connect an external LED with 330Ω resistor to GND
//!
//! # Expected Behavior
//!
//! 1. Built-in LED (PB2) blinks in a pattern
//! 2. If button connected to PB1: pressing button changes blink rate
//! 3. If external LED on PA0: shows different drive strength usage
//!
//! # Learning Objectives
//!
//! - Configure GPIO pins as digital outputs
//! - Configure GPIO pins as digital inputs with pull resistors
//! - Use type-safe pin mode conversions
//! - Implement embedded-hal digital I/O traits
//! - Configure drive strength for outputs

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_hal::{
    clock::{ClockConfig, Clocks, HfxoConfig, LfxoConfig},
    delay::Delay,
    gpio::{DriveStrength, GpioExt},
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
    let (clocks, cmu) = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)),
            lfxo: Some(LfxoConfig::default()),
        },
    )
    .expect("Clock configuration failed");

    let frozen_clocks = clocks.freeze(cmu);

    // Create delay provider
    let mut delay = Delay::new(cp.SYST, &frozen_clocks);

    // ===================================================================
    // SECTION 1: Basic Digital Output
    // ===================================================================

    // Split GPIO peripheral into individual ports and pins
    // This consumes the GPIO peripheral and returns type-safe pin objects
    let gpio = dp.gpio_s.split(&frozen_clocks);

    // Configure PB2 as a push-pull output (built-in LED on XIAO MG24)
    // The type system ensures we can only use output methods on this pin
    let mut led = gpio.portb.pb2.into_push_pull_output();

    // Demonstrate basic LED control
    led.set_high().unwrap(); // Turn LED on
    delay.delay_ms(1000);
    led.set_low().unwrap(); // Turn LED off
    delay.delay_ms(1000);

    // ===================================================================
    // SECTION 2: Digital Input with Pull Resistor
    // ===================================================================

    // Configure PB1 as an input with internal pull-up resistor
    // This is suitable for active-low buttons (button connects to GND)
    // The internal pull-up keeps the pin HIGH when button is not pressed
    let mut button = gpio.portb.pb1.into_pull_up_input();

    // Read button state
    // Note: The type system prevents us from writing to this pin
    // because it's configured as an input
    let button_pressed = button.is_low().unwrap(); // Active low

    if button_pressed {
        // Blink quickly if button was pressed at startup
        for _ in 0..5 {
            led.set_high().unwrap();
            delay.delay_ms(100);
            led.set_low().unwrap();
            delay.delay_ms(100);
        }
    }

    // ===================================================================
    // SECTION 3: Drive Strength Configuration
    // ===================================================================

    // Configure PA0 with high drive strength
    // Useful when:
    // - Driving LEDs directly (without buffer)
    // - Driving capacitive loads
    // - Needing faster edge transitions
    let mut high_power_output = gpio
        .porta
        .pa0
        .into_push_pull_output_with_drive(DriveStrength::Strong);

    // Demonstrate drive strength
    high_power_output.set_high().unwrap();
    delay.delay_ms(100);
    high_power_output.set_low().unwrap();

    // ===================================================================
    // SECTION 4: Multiple Pin Control Pattern
    // ===================================================================

    // Configure additional pins for pattern demonstration
    let mut led2 = gpio.porta.pa1.into_push_pull_output();
    let mut led3 = gpio
        .porta
        .pa2
        .into_push_pull_output_with_drive(DriveStrength::Standard);

    // Create a running light pattern
    for _ in 0..3 {
        led.set_high().unwrap();
        delay.delay_ms(100);
        led.set_low().unwrap();

        led2.set_high().unwrap();
        delay.delay_ms(100);
        led2.set_low().unwrap();

        led3.set_high().unwrap();
        delay.delay_ms(100);
        led3.set_low().unwrap();
    }

    // ===================================================================
    // SECTION 5: Input-Output Interaction
    // ===================================================================

    // Create a simple button-controlled LED loop
    // This demonstrates reading inputs and controlling outputs
    loop {
        // Check button state
        if button.is_low().unwrap() {
            // Button pressed - fast blink pattern
            led.set_high().unwrap();
            delay.delay_ms(100);
            led.set_low().unwrap();
            delay.delay_ms(100);
        } else {
            // Button not pressed - slow blink pattern
            led.set_high().unwrap();
            delay.delay_ms(500);
            led.set_low().unwrap();
            delay.delay_ms(500);
        }

        // Alternate pattern on other LEDs
        led2.set_high().unwrap();
        led3.set_low().unwrap();
        delay.delay_ms(250);
        led2.set_low().unwrap();
        led3.set_high().unwrap();
        delay.delay_ms(250);
    }
}

// =======================================================================
// NOTES AND BEST PRACTICES
// =======================================================================

// 1. Type Safety:
//    - Pins start as Input by default
//    - Must explicitly convert to Output before writing
//    - Compiler prevents using pins in wrong mode
//
//    let input = gpio.pb1.into_floating_input();
//    // input.set_high();  // ERROR: won't compile!
//
//    let mut output = input.into_push_pull_output();
//    output.set_high();  // OK now

// 2. Ownership:
//    - Each pin can only exist once
//    - Consumed when converted to different mode
//    - Prevents accidental double-configuration
//
//    let led = gpio.pb2.into_push_pull_output();
//    // let led2 = gpio.pb2.into_push_pull_output();  // ERROR: pb2 already consumed

// 3. Pull Resistors:
//    - Internal pull-up/down: 20-50 kΩ (typical 30 kΩ)
//    - Good for buttons and switches
//    - May need external pull-ups for I2C or high-speed signals
//
//    // Active-low button (button connects to GND)
//    let button = gpio.pb1.into_pull_up_input();
//    if button.is_low().unwrap() { /* pressed */ }
//
//    // Active-high button (button connects to VDD)
//    let button = gpio.pb1.into_pull_down_input();
//    if button.is_high().unwrap() { /* pressed */ }

// 4. Drive Strength:
//    - Weak (1 mA): Low-power indicators
//    - Standard (6 mA): General purpose, most LEDs
//    - Strong (10 mA): High-current LEDs, capacitive loads
//
//    // Standard LED with resistor
//    let mut led = gpio.pb2.into_push_pull_output();
//
//    // High-power LED or long wire
//    let mut led = gpio.pb2.into_push_pull_output_with_drive(DriveStrength::Strong);

// 5. Current Limiting:
//    Always use resistors with LEDs to limit current:
//
//    GPIO ---[330Ω]---[LED]--- GND
//
//    For 3.3V supply and red LED (Vf ≈ 2.0V):
//    I = (3.3V - 2.0V) / 330Ω ≈ 4 mA (safe for Standard drive)

// 6. embedded-hal Compatibility:
//    This module implements embedded-hal v1.0 traits:
//    - OutputPin: set_high(), set_low()
//    - InputPin: is_high(), is_low()
//
//    This makes it compatible with generic embedded-hal drivers

// 7. Pin Conversion Examples:
//
//    // Start as input
//    let pin = gpio.pb2;
//
//    // Convert to output
//    let mut output = pin.into_push_pull_output();
//    output.set_high();
//
//    // Convert back to input
//    let input = output.into_floating_input();
//    let state = input.is_high();
//
//    // Convert to input with pull-up
//    let input_pulled = input.into_pull_up_input();

// 8. Compile-Time Guarantees:
//    The type system provides these guarantees at compile time:
//    - Can't write to inputs
//    - Can't read outputs (without changing mode)
//    - Can't use the same pin twice
//    - Can't use pins that don't exist on the chip
//    - No runtime overhead for these safety checks

// 9. Common Patterns:
//
//    // LED blink
//    loop {
//        led.set_high().unwrap();
//        delay.delay_ms(500);
//        led.set_low().unwrap();
//        delay.delay_ms(500);
//    }
//
//    // Toggle pattern
//    led.set_high().unwrap();
//    led.set_low().unwrap();
//
//    // Button debouncing (basic)
//    if button.is_low().unwrap() {
//        delay.delay_ms(50);  // Debounce delay
//        if button.is_low().unwrap() {
//            // Button definitely pressed
//        }
//    }

// 10. Future Features (not yet implemented):
//     - GPIO interrupts (EXTI)
//     - Alternate function configuration
//     - Output speed/slew rate control
//     - Open-drain output mode
//     - Analog mode for ADC
