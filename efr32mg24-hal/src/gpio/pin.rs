//! GPIO Pin implementation
//!
//! This module contains the Pin struct and its mode conversion methods.
//! The Pin type uses Rust's type system to enforce compile-time safety
//! for GPIO operations, ensuring pins can only be used in their configured mode.

use core::marker::PhantomData;

use super::types::{Alternate, Analog, DriveStrength, Input, Output, Pull};

/// Generic pin type
pub struct Pin<const PORT: char, const PIN: u8, MODE = Input<Pull>> {
    pub(crate) _mode: PhantomData<MODE>,
}

impl<const PORT: char, const PIN: u8, MODE> Pin<PORT, PIN, MODE> {
    /// Create a new pin in the specified mode
    ///
    /// This is safe because pins can only be created by consuming the GPIO peripheral,
    /// which ensures each pin exists exactly once.
    pub(crate) fn new() -> Self {
        Self { _mode: PhantomData }
    }

    /// Convert pin to push-pull output mode with standard drive strength
    pub fn into_push_pull_output(self) -> Pin<PORT, PIN, Output<0>> {
        // Configure pin as push-pull output
        critical_section::with(|_cs| {
            let gpio = unsafe { &(*crate::pac::GpioS::ptr()) };

            // Configure the pin mode based on port and pin number
            match (PORT, PIN) {
                ('A', 0..=7) => {
                    gpio.porta_model().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (PIN * 4))) | (0x4 << (PIN * 4)))
                    });
                }
                ('A', 8..=15) => {
                    let pin_offset = PIN - 8;
                    gpio.porta_modeh().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (pin_offset * 4))) | (0x4 << (pin_offset * 4)))
                    });
                }
                ('B', 0..=7) => {
                    gpio.portb_model().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (PIN * 4))) | (0x4 << (PIN * 4)))
                    });
                }
                ('C', 0..=7) => {
                    gpio.portc_model().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (PIN * 4))) | (0x4 << (PIN * 4)))
                    });
                }
                ('C', 8..=15) => {
                    let pin_offset = PIN - 8;
                    gpio.portc_modeh().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (pin_offset * 4))) | (0x4 << (pin_offset * 4)))
                    });
                }
                ('D', 0..=7) => {
                    gpio.portd_model().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (PIN * 4))) | (0x4 << (PIN * 4)))
                    });
                }
                _ => panic!("Invalid port/pin combination"),
            }
        });

        Pin { _mode: PhantomData }
    }

    /// Convert pin to push-pull output mode with specified drive strength
    pub fn into_push_pull_output_with_drive(
        self,
        _drive: DriveStrength,
    ) -> Pin<PORT, PIN, Output<0>> {
        // TODO: Configure actual hardware registers with specified drive strength
        Pin { _mode: PhantomData }
    }

    /// Convert pin to input mode with no pull resistor
    pub fn into_floating_input(self) -> Pin<PORT, PIN, Input<Pull>> {
        // TODO: Configure actual hardware registers
        // - Set pin mode to input
        // - Disable pull-up/pull-down
        Pin { _mode: PhantomData }
    }

    /// Convert pin to input mode with pull-up resistor
    pub fn into_pull_up_input(self) -> Pin<PORT, PIN, Input<Pull>> {
        // Configure pin as input with pull-up
        critical_section::with(|_cs| {
            let gpio = unsafe { &(*crate::pac::GpioS::ptr()) };

            // First, set DOUT high to enable pull-up (when mode is INPUTPULL)
            match PORT {
                'A' => {
                    gpio.porta_dout().modify(|r, w| unsafe {
                        w.bits(r.bits() | (1 << PIN))
                    });
                }
                'B' => {
                    gpio.portb_dout().modify(|r, w| unsafe {
                        w.bits(r.bits() | (1 << PIN))
                    });
                }
                'C' => {
                    gpio.portc_dout().modify(|r, w| unsafe {
                        w.bits(r.bits() | (1 << PIN))
                    });
                }
                'D' => {
                    gpio.portd_dout().modify(|r, w| unsafe {
                        w.bits(r.bits() | (1 << PIN))
                    });
                }
                _ => panic!("Invalid port"),
            }

            // Then configure mode as INPUTPULL (mode 2)
            match (PORT, PIN) {
                ('A', 0..=7) => {
                    gpio.porta_model().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (PIN * 4))) | (0x2 << (PIN * 4)))
                    });
                }
                ('A', 8..=15) => {
                    let pin_offset = PIN - 8;
                    gpio.porta_modeh().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (pin_offset * 4))) | (0x2 << (pin_offset * 4)))
                    });
                }
                ('B', 0..=7) => {
                    gpio.portb_model().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (PIN * 4))) | (0x2 << (PIN * 4)))
                    });
                }
                ('C', 0..=7) => {
                    gpio.portc_model().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (PIN * 4))) | (0x2 << (PIN * 4)))
                    });
                }
                ('C', 8..=15) => {
                    let pin_offset = PIN - 8;
                    gpio.portc_modeh().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (pin_offset * 4))) | (0x2 << (pin_offset * 4)))
                    });
                }
                ('D', 0..=7) => {
                    gpio.portd_model().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (PIN * 4))) | (0x2 << (PIN * 4)))
                    });
                }
                _ => panic!("Invalid port/pin combination"),
            }
        });

        Pin { _mode: PhantomData }
    }

    /// Convert pin to input mode with pull-down resistor
    pub fn into_pull_down_input(self) -> Pin<PORT, PIN, Input<Pull>> {
        // Configure pin as input with pull-down
        critical_section::with(|_cs| {
            let gpio = unsafe { &(*crate::pac::GpioS::ptr()) };

            // First, set DOUT low to enable pull-down (when mode is INPUTPULL)
            match PORT {
                'A' => {
                    gpio.porta_dout().modify(|r, w| unsafe {
                        w.bits(r.bits() & !(1 << PIN))
                    });
                }
                'B' => {
                    gpio.portb_dout().modify(|r, w| unsafe {
                        w.bits(r.bits() & !(1 << PIN))
                    });
                }
                'C' => {
                    gpio.portc_dout().modify(|r, w| unsafe {
                        w.bits(r.bits() & !(1 << PIN))
                    });
                }
                'D' => {
                    gpio.portd_dout().modify(|r, w| unsafe {
                        w.bits(r.bits() & !(1 << PIN))
                    });
                }
                _ => panic!("Invalid port"),
            }

            // Then configure mode as INPUTPULL (mode 2)
            match (PORT, PIN) {
                ('A', 0..=7) => {
                    gpio.porta_model().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (PIN * 4))) | (0x2 << (PIN * 4)))
                    });
                }
                ('A', 8..=15) => {
                    let pin_offset = PIN - 8;
                    gpio.porta_modeh().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (pin_offset * 4))) | (0x2 << (pin_offset * 4)))
                    });
                }
                ('B', 0..=7) => {
                    gpio.portb_model().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (PIN * 4))) | (0x2 << (PIN * 4)))
                    });
                }
                ('C', 0..=7) => {
                    gpio.portc_model().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (PIN * 4))) | (0x2 << (PIN * 4)))
                    });
                }
                ('C', 8..=15) => {
                    let pin_offset = PIN - 8;
                    gpio.portc_modeh().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (pin_offset * 4))) | (0x2 << (pin_offset * 4)))
                    });
                }
                ('D', 0..=7) => {
                    gpio.portd_model().modify(|r, w| unsafe {
                        w.bits((r.bits() & !(0xF << (PIN * 4))) | (0x2 << (PIN * 4)))
                    });
                }
                _ => panic!("Invalid port/pin combination"),
            }
        });

        Pin { _mode: PhantomData }
    }

    /// Convert pin to analog mode (disables digital functions)
    pub fn into_analog(self) -> Pin<PORT, PIN, Analog> {
        // TODO: Configure actual hardware registers
        // - Disable digital input/output
        // - Prepare for analog use (ADC)
        Pin { _mode: PhantomData }
    }

    /// Convert pin to alternate function mode
    pub fn into_alternate<const AF: u8>(self) -> Pin<PORT, PIN, Alternate<AF>> {
        // TODO: Configure actual hardware registers
        // - Set pin to alternate function mode
        // - Route alternate function AF to this pin
        Pin { _mode: PhantomData }
    }
}
