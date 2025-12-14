//! Embedded-HAL trait implementations for GPIO pins
//!
//! This module implements the embedded-hal digital I/O traits for GPIO pins,
//! providing a standard interface for digital input and output operations.

use embedded_hal::digital::{ErrorType, InputPin, OutputPin};

use super::pin::Pin;
use super::types::{GpioError, Input, Output};

// Implement embedded-hal traits for Output mode
impl<const PORT: char, const PIN: u8, const DRIVE: u8> ErrorType for Pin<PORT, PIN, Output<DRIVE>> {
    type Error = GpioError;
}

impl<const PORT: char, const PIN: u8, const DRIVE: u8> OutputPin for Pin<PORT, PIN, Output<DRIVE>> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        // Clear the output bit
        critical_section::with(|_cs| {
            let gpio = unsafe { &(*crate::pac::GpioS::ptr()) };

            match PORT {
                'A' => {
                    gpio.porta_dout()
                        .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << PIN)) });
                }
                'B' => {
                    gpio.portb_dout()
                        .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << PIN)) });
                }
                'C' => {
                    gpio.portc_dout()
                        .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << PIN)) });
                }
                'D' => {
                    gpio.portd_dout()
                        .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << PIN)) });
                }
                _ => return Err(GpioError::InvalidMode),
            }
            Ok(())
        })
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        // Set the output bit
        critical_section::with(|_cs| {
            let gpio = unsafe { &(*crate::pac::GpioS::ptr()) };

            match PORT {
                'A' => {
                    gpio.porta_dout()
                        .modify(|r, w| unsafe { w.bits(r.bits() | (1 << PIN)) });
                }
                'B' => {
                    gpio.portb_dout()
                        .modify(|r, w| unsafe { w.bits(r.bits() | (1 << PIN)) });
                }
                'C' => {
                    gpio.portc_dout()
                        .modify(|r, w| unsafe { w.bits(r.bits() | (1 << PIN)) });
                }
                'D' => {
                    gpio.portd_dout()
                        .modify(|r, w| unsafe { w.bits(r.bits() | (1 << PIN)) });
                }
                _ => return Err(GpioError::InvalidMode),
            }
            Ok(())
        })
    }
}

// Implement embedded-hal traits for Input mode
impl<const PORT: char, const PIN: u8, PULL> ErrorType for Pin<PORT, PIN, Input<PULL>> {
    type Error = GpioError;
}

impl<const PORT: char, const PIN: u8, PULL> InputPin for Pin<PORT, PIN, Input<PULL>> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        // Read the input bit
        let result = critical_section::with(|_cs| {
            let gpio = unsafe { &(*crate::pac::GpioS::ptr()) };

            let din_value = match PORT {
                'A' => gpio.porta_din().read().bits(),
                'B' => gpio.portb_din().read().bits(),
                'C' => gpio.portc_din().read().bits(),
                'D' => gpio.portd_din().read().bits(),
                _ => return Err(GpioError::InvalidMode),
            };

            Ok((din_value & (1 << PIN)) != 0)
        });

        result
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.is_high().map(|v| !v)
    }
}
