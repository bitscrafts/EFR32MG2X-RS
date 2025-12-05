//! GPIO type definitions and enums
//!
//! This module contains the core type definitions for GPIO pins including:
//! - Error types
//! - Drive strength configuration
//! - Pull resistor configuration
//! - Pin mode type states
//! - Port structures

use core::marker::PhantomData;

use super::pin::Pin;

/// GPIO error type
#[derive(Debug, Clone, Copy)]
pub enum GpioError {
    /// Pin is in wrong mode for requested operation
    InvalidMode,
}

impl embedded_hal::digital::Error for GpioError {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        embedded_hal::digital::ErrorKind::Other
    }
}

/// Drive strength for output pins
#[derive(Debug, Clone, Copy)]
pub enum DriveStrength {
    /// Weak drive strength (1 mA)
    Weak,
    /// Standard drive strength (6 mA) - default
    Standard,
    /// Strong drive strength (10 mA)
    Strong,
}

/// Pull configuration for input pins
#[derive(Debug, Clone, Copy)]
pub enum Pull {
    /// No pull resistor
    None,
    /// Pull-up resistor enabled
    Up,
    /// Pull-down resistor enabled
    Down,
}

// Type-state markers for pin modes
/// Marker trait for pin modes
pub trait PinMode {}

/// Input mode (type state)
pub struct Input<PULL = Pull> {
    _pull: PhantomData<PULL>,
}
impl<PULL> PinMode for Input<PULL> {}

/// Output mode (type state)
pub struct Output<const DRIVE: u8 = 0> {}
impl<const DRIVE: u8> PinMode for Output<DRIVE> {}

/// Analog mode (type state)
pub struct Analog;
impl PinMode for Analog {}

/// Alternate function mode (type state)
pub struct Alternate<const AF: u8>;
impl<const AF: u8> PinMode for Alternate<AF> {}

/// Floating input (no pull resistor)
pub type Floating = Pull;

/// Pull-up input
pub type PullUp = Pull;

/// Pull-down input
pub type PullDown = Pull;

/// All GPIO ports and pins
pub struct Parts {
    /// Port A pins
    pub porta: PortA,
    /// Port B pins
    pub portb: PortB,
    /// Port C pins
    pub portc: PortC,
    /// Port D pins
    pub portd: PortD,
}

// Macro to generate port structures
macro_rules! gpio_port {
    ($port:ident, $PORT:literal, $($pin:ident: $PIN:literal,)*) => {
        /// GPIO port pins
        pub struct $port {
            $(
                #[doc = concat!("Pin P", $PORT, stringify!($PIN))]
                pub $pin: Pin<$PORT, $PIN, Input<Pull>>,
            )*
        }

        impl $port {
            pub(crate) fn new() -> Self {
                Self {
                    $(
                        $pin: Pin::new(),
                    )*
                }
            }
        }
    };
}

// Define all GPIO ports for EFR32MG24
// Note: Pin availability varies by package. This is for the 48-pin QFN package (XIAO MG24)

gpio_port! {
    PortA, 'A',
    pa0: 0,
    pa1: 1,
    pa2: 2,
    pa3: 3,
    pa4: 4,
    pa5: 5,
    pa6: 6,
    pa7: 7,
    pa8: 8,
}

gpio_port! {
    PortB, 'B',
    pb0: 0,
    pb1: 1,
    pb2: 2,
    pb3: 3,
    pb4: 4,
}

gpio_port! {
    PortC, 'C',
    pc0: 0,
    pc1: 1,
    pc2: 2,
    pc3: 3,
}

gpio_port! {
    PortD, 'D',
    pd0: 0,
    pd1: 1,
    pd2: 2,
    pd3: 3,
}
