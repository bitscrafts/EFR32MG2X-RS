//! XIAO MG24 Sense pin definitions
//!
//! Maps the XIAO connector pins to EFR32MG24 GPIO ports/pins.
//!
//! # XIAO Pin Layout
//!
//! ```text
//! XIAO MG24 Sense Pin Mapping:
//!
//!        USB-C
//!     ┌─────────┐
//! D0  │ PA05    │ 5V
//! D1  │ PA06    │ GND
//! D2  │ PC00    │ 3V3
//! D3  │ PC01    │ PB02  D10
//! D4  │ PC02    │ PB03  D9
//! D5  │ PC03    │ PB04  D8
//! D6  │ PB00    │ PB01  D7
//!     └─────────┘
//!
//! Pin Aliases:
//! - LED: PA07 (built-in user LED, active low, NOT on XIAO connector)
//! - D7/BUTTON: PB01 (user button, active low)
//! - VBAT_SENSE: PA00 (battery voltage monitoring)
//!
//! NOTE: D10 (PB02) is NOT the built-in LED. The built-in LED is on PA7
//! (Arduino pin 20), confirmed by the official Silicon Labs Arduino core
//! (`variants/xiao_mg24/arduino_variant.cpp:75`: `PA7, // LED - 20`).
//! ```

use crate::hal::gpio::*;

/// XIAO MG24 Sense pin definitions
///
/// All pins start in Input<Pull> mode after initialization.
///
/// Note: `d10` and `d7` can also be used as LED and button respectively.
pub struct Pins {
    /// D0 - PA05 (GPIO, USART0 TX, I2C0 SDA)
    pub d0: Pin<'A', 5, Input<Pull>>,

    /// D1 - PA06 (GPIO, USART0 RX, I2C0 SCL)
    pub d1: Pin<'A', 6, Input<Pull>>,

    /// D2 - PC00 (GPIO, SPI0 MOSI)
    pub d2: Pin<'C', 0, Input<Pull>>,

    /// D3 - PC01 (GPIO, SPI0 MISO)
    pub d3: Pin<'C', 1, Input<Pull>>,

    /// D4 - PC02 (GPIO, SPI0 SCK)
    pub d4: Pin<'C', 2, Input<Pull>>,

    /// D5 - PC03 (GPIO, SPI0 CS)
    pub d5: Pin<'C', 3, Input<Pull>>,

    /// D6 - PB00 (GPIO, TIMER0 CC0)
    pub d6: Pin<'B', 0, Input<Pull>>,

    /// D7 - PB01 (GPIO, TIMER0 CC1, USER BUTTON)
    ///
    /// This pin has a user button connected (active low).
    pub d7: Pin<'B', 1, Input<Pull>>,

    /// D8 - PB04 (GPIO, ADC0 CH0)
    pub d8: Pin<'B', 4, Input<Pull>>,

    /// D9 - PB03 (GPIO, ADC0 CH1)
    pub d9: Pin<'B', 3, Input<Pull>>,

    /// D10 - PB02 (GPIO, PWM)
    ///
    /// NOTE: This is NOT the built-in LED. The built-in LED is on PA7
    /// (see `led` field below). PB2 is Arduino D13 (SDA1/DAC2).
    pub d10: Pin<'B', 2, Input<Pull>>,

    /// Built-in user LED (PA07, active low, NOT on XIAO connector)
    ///
    /// This is the onboard LED confirmed by the official Silicon Labs
    /// Arduino core. It is active low (clear bit = LED on, set bit = LED off).
    pub led: Pin<'A', 7, Input<Pull>>,

    /// Battery voltage sense (PA00, ADC)
    ///
    /// Note: Not on XIAO connector, internal sensing
    pub vbat_sense: Pin<'A', 0, Input<Pull>>,

    /// PC8 - Microphone power control
    ///
    /// Controls power to MSM381ACT001 analog microphone (active high).
    pub pc8: Pin<'C', 8, Input<Pull>>,

    /// PC9 - Microphone analog input
    ///
    /// Analog input from MSM381ACT001 microphone (IADC channel).
    pub pc9: Pin<'C', 9, Input<Pull>>,
}

impl Pins {
    /// Create pin definitions from GPIO Parts
    ///
    /// This takes ownership of the GPIO Parts from `GpioExt::split()`.
    pub(crate) fn new(parts: Parts) -> Self {
        Self {
            // XIAO connector pins
            d0: parts.porta.pa5,
            d1: parts.porta.pa6,
            d2: parts.portc.pc0,
            d3: parts.portc.pc1,
            d4: parts.portc.pc2,
            d5: parts.portc.pc3,
            d6: parts.portb.pb0,
            d7: parts.portb.pb1, // USER BUTTON
            d8: parts.portb.pb4,
            d9: parts.portb.pb3,
            d10: parts.portb.pb2,
            led: parts.porta.pa7, // BUILT-IN LED (PA7, active low)
            vbat_sense: parts.porta.pa0,
            pc8: parts.portc.pc8, // Microphone power
            pc9: parts.portc.pc9, // Microphone input
        }
    }
}

// Type aliases for microphone pins
pub type Pc8<MODE> = Pin<'C', 8, MODE>;
pub type Pc9<MODE> = Pin<'C', 9, MODE>;
