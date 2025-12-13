//! Timer and PWM Driver
//!
//! This module provides Timer and PWM functionality using TIMER0-4 peripherals
//! for the EFR32MG24 microcontroller.
//!
//! # Hardware Registers
//!
//! - **EN**: Enable register
//! - **CTRL**: Control register (mode, counting direction)
//! - **CFG**: Configuration register
//! - **CMD**: Command register (start, stop)
//! - **CNT**: Counter value
//! - **TOP**: Top value (period for PWM)
//! - **CC0/CC1/CC2_OC**: Output Compare registers (duty cycle)
//! - **CC0/CC1/CC2_CTRL**: Channel control registers
//! - **STATUS**: Status flags
//!
//! # Features
//!
//! - 5 Timer instances (Timer0-4)
//! - Each timer has 3 PWM channels
//! - Edge-aligned and center-aligned PWM modes
//! - Configurable frequency and duty cycle
//! - 16-bit or 32-bit resolution
//! - Hardware register access
//!
//! # Example
//!
//! ```no_run
//! use efr32mg24_hal::{
//!     clock::{Clocks, ClockConfig, HfxoConfig},
//!     timer::{Timer0, Config, PwmMode, PwmChannel},
//!     pac,
//! };
//!
//! let dp = pac::Peripherals::take().unwrap();
//!
//! // Configure clocks
//! let clocks = Clocks::new(dp.cmu_s, ClockConfig {
//!     hfxo: Some(HfxoConfig::new(39_000_000)),
//!     lfxo: Some(Default::default()),
//! }).freeze();
//!
//! // Create Timer0 with PWM at 10 kHz
//! let mut timer = Timer0::new(
//!     dp.timer0_s,
//!     Config::new(10_000).with_pwm(PwmMode::EdgeAligned),
//!     &clocks
//! );
//!
//! // Set 50% duty cycle on channel 0
//! timer.set_duty_cycle(PwmChannel::Channel0, 50).unwrap();
//! timer.enable_channel(PwmChannel::Channel0);
//! ```

mod types;
mod traits;

pub use types::{Config, Error, PwmChannel, PwmMode};

use crate::clock::FrozenClocks;
use crate::pac;

/// Macro to implement timer instances
macro_rules! impl_timer {
    ($TimerX:ident, $timerX_s:ident, $timerx:ident, $clken_bit:ident) => {
        /// Timer instance
        pub struct $TimerX {
            timer: pac::$timerX_s,
            frequency: u32,
            top_value: u32,
            pwm_enabled: bool,
        }

        impl $TimerX {
            /// Creates a new timer instance
            ///
            /// # Arguments
            ///
            /// * `timer` - Timer peripheral instance
            /// * `config` - Timer configuration
            /// * `clocks` - Frozen clock configuration
            pub fn new(timer: pac::$timerX_s, config: Config, clocks: &FrozenClocks) -> Self {
                // Enable timer clock
                critical_section::with(|_cs| {
                    let cmu = unsafe { &(*pac::CmuS::ptr()) };
                    cmu.clken0().modify(|_, w| w.$clken_bit().set_bit());
                });

                // Enable timer peripheral
                timer.en().write(|w| w.en().set_bit());

                // Calculate prescaler and top value for desired frequency
                let (prescaler, top) = Self::calculate_prescaler_and_top(
                    clocks.hfclk().into(),
                    config.frequency
                );

                // Configure PWM mode if requested
                let pwm_enabled = if let Some(pwm_mode) = config.pwm_mode {
                    match pwm_mode {
                        PwmMode::EdgeAligned => {
                            // Edge-aligned PWM: count up
                            timer.cfg().write(|w| unsafe {
                                w.presc().bits(prescaler)
                                    .mode().up()
                            });
                        }
                        PwmMode::CenterAligned => {
                            // Center-aligned PWM: count up/down
                            timer.cfg().write(|w| unsafe {
                                w.presc().bits(prescaler)
                                    .mode().updown()
                            });
                        }
                    }
                    true
                } else {
                    // Basic timer mode: count up
                    timer.cfg().write(|w| unsafe {
                        w.presc().bits(prescaler)
                            .mode().up()
                    });
                    false
                };

                // Set top value (PWM period)
                timer.top().write(|w| unsafe { w.bits(top) });

                Self {
                    timer,
                    frequency: config.frequency,
                    top_value: top,
                    pwm_enabled,
                }
            }

            /// Calculate prescaler and top value for desired frequency
            ///
            /// Formula: Timer_freq = HFCLK / (prescaler * (TOP + 1))
            fn calculate_prescaler_and_top(hfclk: u32, target_freq: u32) -> (u16, u32) {
                // Try prescalers from 0 to 10 (dividers: 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024)
                for prescaler in 0..=10u16 {
                    let divider = 1u32 << prescaler;
                    let timer_clk = hfclk / divider;

                    // Calculate TOP value
                    if timer_clk > target_freq {
                        let top = (timer_clk / target_freq).saturating_sub(1);
                        if top <= 0xFFFF {  // 16-bit limit
                            return (prescaler, top);
                        }
                    }
                }

                // Fallback: maximum prescaler with best TOP
                let divider = 1024u32;
                let timer_clk = hfclk / divider;
                let top = (timer_clk / target_freq).saturating_sub(1).min(0xFFFF);
                (10, top)
            }

            /// Start the timer
            pub fn start(&mut self) {
                self.timer.cmd().write(|w| w.start().set_bit());
            }

            /// Stop the timer
            pub fn stop(&mut self) {
                self.timer.cmd().write(|w| w.stop().set_bit());
            }

            /// Get current counter value
            pub fn get_counter(&self) -> u32 {
                self.timer.cnt().read().bits()
            }

            /// Reset counter to zero
            pub fn reset_counter(&mut self) {
                self.timer.cnt().write(|w| unsafe { w.bits(0) });
            }

            /// Set duty cycle for a PWM channel
            ///
            /// # Arguments
            ///
            /// * `channel` - PWM channel (0, 1, or 2)
            /// * `duty_percent` - Duty cycle percentage (0-100)
            ///
            /// # Returns
            ///
            /// `Ok(())` on success, `Err(Error)` if duty cycle is invalid
            pub fn set_duty_cycle(&mut self, channel: PwmChannel, duty_percent: u8) -> Result<(), Error> {
                if duty_percent > 100 {
                    return Err(Error::InvalidDutyCycle);
                }

                if !self.pwm_enabled {
                    return Err(Error::InvalidChannel);
                }

                // Calculate compare value
                let compare_value = (self.top_value as u64 * duty_percent as u64 / 100) as u32;

                // Set compare value for the channel
                match channel {
                    PwmChannel::Channel0 => {
                        self.timer.cc0_oc().write(|w| unsafe { w.bits(compare_value) });
                    }
                    PwmChannel::Channel1 => {
                        self.timer.cc1_oc().write(|w| unsafe { w.bits(compare_value) });
                    }
                    PwmChannel::Channel2 => {
                        self.timer.cc2_oc().write(|w| unsafe { w.bits(compare_value) });
                    }
                }

                Ok(())
            }

            /// Enable PWM output on a channel
            ///
            /// # Arguments
            ///
            /// * `channel` - PWM channel to enable
            pub fn enable_channel(&mut self, channel: PwmChannel) {
                match channel {
                    PwmChannel::Channel0 => {
                        // Configure channel mode in CC0_CFG
                        self.timer.cc0_cfg().write(|w| {
                            w.mode().pwm() // PWM mode
                                .coist().clear_bit() // Output low when timer disabled
                        });
                        // Configure output action in CC0_CTRL
                        self.timer.cc0_ctrl().write(|w| {
                            w.outinv().clear_bit() // Non-inverted
                                .cofoa().toggle() // Toggle on counter overflow
                                .cmoa().set_() // Set on compare match
                        });
                    }
                    PwmChannel::Channel1 => {
                        self.timer.cc1_cfg().write(|w| {
                            w.mode().pwm()
                                .coist().clear_bit()
                        });
                        self.timer.cc1_ctrl().write(|w| {
                            w.outinv().clear_bit()
                                .cofoa().toggle()
                                .cmoa().set_()
                        });
                    }
                    PwmChannel::Channel2 => {
                        self.timer.cc2_cfg().write(|w| {
                            w.mode().pwm()
                                .coist().clear_bit()
                        });
                        self.timer.cc2_ctrl().write(|w| {
                            w.outinv().clear_bit()
                                .cofoa().toggle()
                                .cmoa().set_()
                        });
                    }
                }
            }

            /// Disable PWM output on a channel
            ///
            /// # Arguments
            ///
            /// * `channel` - PWM channel to disable
            pub fn disable_channel(&mut self, channel: PwmChannel) {
                match channel {
                    PwmChannel::Channel0 => {
                        self.timer.cc0_cfg().write(|w| w.mode().off());
                    }
                    PwmChannel::Channel1 => {
                        self.timer.cc1_cfg().write(|w| w.mode().off());
                    }
                    PwmChannel::Channel2 => {
                        self.timer.cc2_cfg().write(|w| w.mode().off());
                    }
                }
            }

            /// Get the configured PWM frequency
            pub fn get_frequency(&self) -> u32 {
                self.frequency
            }

            /// Get the TOP value (period)
            pub fn get_top_value(&self) -> u32 {
                self.top_value
            }
        }
    };
}

// Implement all 5 timer instances
impl_timer!(Timer0, Timer0S, timer0, timer0);
impl_timer!(Timer1, Timer1S, timer1, timer1);
impl_timer!(Timer2, Timer2S, timer2, timer2);
impl_timer!(Timer3, Timer3S, timer3, timer3);
impl_timer!(Timer4, Timer4S, timer4, timer4);
