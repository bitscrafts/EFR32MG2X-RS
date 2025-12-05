//! General Purpose Input/Output (GPIO) Module
//!
//! This module provides type-safe, zero-cost abstractions for GPIO pins on the EFR32MG24.
//!
//! # Overview
//!
//! The EFR32MG24 provides multiple GPIO ports (A, B, C, D) with various capabilities:
//! - Digital input/output with configurable drive strength
//! - Internal pull-up/pull-down resistors
//! - Interrupt capability on pin state changes
//! - Alternative function routing for peripherals
//!
//! # Type-Safe Pin Management
//!
//! This module uses Rust's type system to ensure compile-time safety:
//! - Pins can only be used in their configured mode
//! - Mode transitions are enforced at compile time
//! - Prevents accidental misuse of unconfigured pins
//!
//! # Examples
//!
//! ## Basic Digital Output
//!
//! ```no_run
//! use efr32mg24_hal::{gpio::GpioExt, pac};
//! use embedded_hal::digital::OutputPin;
//!
//! let dp = pac::Peripherals::take().unwrap();
//! let gpio = dp.gpio_s.split();
//!
//! // Configure PB2 as push-pull output (LED on XIAO MG24)
//! let mut led = gpio.portb.pb2.into_push_pull_output();
//!
//! // Turn LED on
//! led.set_high().unwrap();
//!
//! // Turn LED off
//! led.set_low().unwrap();
//! ```
//!
//! ## Digital Input with Pull-up
//!
//! ```no_run
//! use efr32mg24_hal::{gpio::GpioExt, pac};
//! use embedded_hal::digital::InputPin;
//!
//! let dp = pac::Peripherals::take().unwrap();
//! let gpio = dp.gpio_s.split();
//!
//! // Configure PB1 as input with internal pull-up (button)
//! let button = gpio.portb.pb1.into_pull_up_input();
//!
//! // Read button state (active low)
//! if button.is_low().unwrap() {
//!     // Button pressed
//! }
//! ```
//!
//! ## Drive Strength Configuration
//!
//! ```no_run
//! use efr32mg24_hal::{gpio::{GpioExt, DriveStrength}, pac};
//!
//! let dp = pac::Peripherals::take().unwrap();
//! let gpio = dp.gpio_s.split();
//!
//! // Configure output with high drive strength for LED
//! let mut led = gpio.portb.pb2.into_push_pull_output_with_drive(DriveStrength::Strong);
//! ```

mod pin;
mod traits;
mod types;

// Re-export public types
pub use pin::Pin;
pub use types::{
    Alternate, Analog, DriveStrength, Floating, GpioError, Input, Output, Parts, PinMode, PortA,
    PortB, PortC, PortD, Pull, PullDown, PullUp,
};

/// Extension trait to split GPIO peripheral into independent pins
pub trait GpioExt {
    /// Split the GPIO peripheral into independent ports
    fn split(self) -> Parts;
}

// Implement GpioExt for the GPIO_S peripheral (Secure GPIO)
impl GpioExt for crate::pac::GpioS {
    fn split(self) -> Parts {
        // Enable GPIO clock in CMU
        critical_section::with(|_cs| {
            let cmu = unsafe { &(*crate::pac::CmuS::ptr()) };
            cmu.clken0().modify(|_r, w| w.gpio().set_bit());
        });

        Parts {
            porta: types::PortA::new(),
            portb: types::PortB::new(),
            portc: types::PortC::new(),
            portd: types::PortD::new(),
        }
    }
}
