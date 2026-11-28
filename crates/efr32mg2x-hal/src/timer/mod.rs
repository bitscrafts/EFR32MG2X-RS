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
//! use efr32mg2x_hal::{
//!     clock::{Clocks, ClockConfig, HfxoConfig},
//!     timer::{Timer0, Config, PwmMode, PwmChannel},
//!     pac,
//! };
//!
//! let dp = pac::Peripherals::take().unwrap();
//!
//! // Configure clocks
//! let clocks = Clocks::new(dp.cmu_s, ClockConfig {
//!     hfxo: None,
//!     lfxo: None,
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

mod traits;
mod types;

pub use types::{Config, Error, PwmChannel, PwmMode};

use crate::clock::FrozenClocks;
use crate::pac;

/// Macro to implement timer instances
macro_rules! impl_timer {
    ($TimerX:ident, $timerX_s:ident, $timerx:ident, $clken_bit:ident) => {
        /// Timer instance
        pub struct $TimerX {
            timer: pac::$timerX_s,
            /// What the caller asked for. Use `get_actual_frequency` for what the timer produces.
            frequency: u32,
            top_value: u32,
            /// The rate this configuration really produces, from the clock it was built with.
            actual_hz: u32,
            /// How far `actual_hz` is from `frequency`, in parts per million; `None` when the request is not
            /// expressible at all and `actual_hz` is the nearest reachable bound.
            error_ppm: Option<u32>,
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
                // Enable timer clock using safe accessor
                clocks.enable_peripheral_clock(|cmu| {
                    cmu.clken0().modify(|_, w| w.$clken_bit().set_bit());
                });
                Self::new_at(timer, config, clocks.hfclk().into())
            }

            /// The same driver, built from the timer clock in hertz instead of a `FrozenClocks`.
            ///
            /// `FrozenClocks` owns the CMU and the `Board` owns the CMU inside `Peripherals`, so a caller that already
            /// took the board can never build one: this constructor was in the crate's documentation and reachable
            /// from nowhere, which is why `TIMER1`–`TIMER4` were "compiled, not measured" for as long as they were.
            /// `usart::Usart0::new_at` and `i2c` had already been untied the same way; `timer` was the last one left.
            ///
            /// The prescaler and `TOP` need one number — HFCLK — and nothing else, so that is what it takes.
            ///
            /// The caller must have enabled `CMU_CLKEN0.$clken_bit`; the `Board`'s hand-outs do, and a caller driving
            /// the PAC directly must do it before this call.
            pub fn new_at(timer: pac::$timerX_s, config: Config, hfclk_hz: u32) -> Self {
                // Unlock, disable, and wait for the block to say it has stopped -- in that order, because `CFG` faults
                // while the timer is enabled.
                //
                // That was known here: `pwm::Timer0::take` has carried the measurement since 2026-11-17. What nobody
                // knew is that *this* constructor enabled the block first and configured it second, so every call
                // was a bus fault. It went unnoticed for as long as it did because nothing in the crate could build
                // a `FrozenClocks` after `Board::take` -- the constructor was documented and unreachable. The first
                // four instances ever built through it died with `Escalated BusFault (Precise data access error) at
                // 0x4004c004`, which is `TIMER1`'s `CFG`.
                //
                // SAFETY: `LOCKKEY` is a 16-bit field and `0xCE80` is the unlock sequence the reference manual gives
                // for TIMER; a wrong sequence is refused by the hardware.
                timer.lock().write(|w| unsafe { w.lockkey().bits(0xCE80) });
                timer.en().write(|w| w.en().clear_bit());
                // Clearing `EN` is not instantaneous; `EN.DISABLING` is the handshake that says it has stopped.
                cortex_m::asm::delay(16);
                let mut settling = 0u32;
                while timer.en().read().disabling().bit_is_set() {
                    settling += 1;
                    if settling > 2_000_000 {
                        break;
                    }
                }

                // The prescaler/`TOP` pair comes from `cores::timer::period_for`, the same function the host tests
                // exercise. Before 2026-11-28 this module had a private second implementation that took the first
                // prescaler that fitted and clamped `TOP` in its fallback, so a frequency the fields cannot express
                // silently became a faster timer. The pair chosen here is the closest expressible one, and how close
                // it is travels with the handle (`get_actual_frequency`, `get_frequency_error_ppm`) instead of being
                // lost.
                let (period, error_ppm) =
                    match crate::cores::timer::period_for(hfclk_hz, config.frequency) {
                        Ok((period, ppm)) => (period, Some(ppm)),
                        // Refused: the request is outside what the prescaler and a 16-bit `TOP` can reach. The nearest
                        // bound is used so the timer still runs, and `error_ppm` stays `None` -- "not expressible" is a
                        // different statement from "expressible but off", and the two must not be confused.
                        Err(_) => (
                            crate::cores::timer::nearest_bound(hfclk_hz, config.frequency),
                            None,
                        ),
                    };
                let top = period.top;
                // The field takes the exponent: 1 -> 0, 2 -> 1, ... 1024 -> 10. `period_for` returns only members of
                // `PRESCALERS`, so the shift is exact.
                let prescaler = (period.prescaler.trailing_zeros() as u16) & 0xF;

                // Configure PWM mode if requested
                let pwm_enabled = if let Some(pwm_mode) = config.pwm_mode {
                    match pwm_mode {
                        PwmMode::EdgeAligned => {
                            // Edge-aligned PWM: count up
                            // SAFETY: `prescaler` is the exponent of a member of `cores::timer::PRESCALERS`, so it is 0..=10, the field's range.
                            // Mode enum variant is type-safe and always valid.
                            timer
                                .cfg()
                                .write(|w| unsafe { w.presc().bits(prescaler).mode().up() });
                        }
                        PwmMode::CenterAligned => {
                            // Center-aligned PWM: count up/down
                            // SAFETY: `prescaler` is the exponent of a member of `cores::timer::PRESCALERS`, so it is 0..=10, the field's range.
                            // Mode enum variant is type-safe and always valid.
                            timer
                                .cfg()
                                .write(|w| unsafe { w.presc().bits(prescaler).mode().updown() });
                        }
                    }
                    true
                } else {
                    // Basic timer mode: count up
                    // SAFETY: `prescaler` is the exponent of a member of `cores::timer::PRESCALERS`, so it is 0..=10, the field's range.
                    // Mode enum variant is type-safe and always valid.
                    timer
                        .cfg()
                        .write(|w| unsafe { w.presc().bits(prescaler).mode().up() });
                    false
                };

                // Set top value (PWM period)
                // `TOP` is written with the block ENABLED -- the order `pwm::Timer0::take` proves works, and measured
                // here the other way round: with `EN` still clear this write is a bus fault at `base + 0x1c`
                // (`Escalated BusFault at 0x4004c01c`). `CFG` wants the timer stopped, `TOP` wants it running.
                timer.en().write(|w| w.en().set_bit());
                // SAFETY: `TOP` is the timer's own 16-bit field, and `top` comes either from `cores::timer::period_for`
                // (which skips any candidate above `MAX_TOP`) or from `nearest_bound` (which returns `MAX_TOP` itself).
                timer.top().write(|w| unsafe { w.bits(top) });

                Self {
                    timer,
                    frequency: config.frequency,
                    top_value: top,
                    actual_hz: period.frequency(hfclk_hz),
                    error_ppm,
                    pwm_enabled,
                }
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
                // SAFETY: Writing 0 to CNT register is always safe and resets the counter.
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
            ///
            /// # Thread Safety
            ///
            /// This function uses critical sections for atomic register access,
            /// making it safe to call from interrupt contexts or RTOS tasks.
            pub fn set_duty_cycle(
                &mut self,
                channel: PwmChannel,
                duty_percent: u8,
            ) -> Result<(), Error> {
                if duty_percent > 100 {
                    return Err(Error::InvalidDutyCycle);
                }

                if !self.pwm_enabled {
                    return Err(Error::InvalidChannel);
                }

                // Calculate compare value
                let compare_value = (self.top_value as u64 * duty_percent as u64 / 100) as u32;

                // Set compare value for the channel atomically
                // Use critical section to prevent race conditions in RTOS environments
                critical_section::with(|_cs| {
                    // SAFETY: Compare value is calculated from validated duty_percent (0-100) and
                    // top_value (<=0xFFFF), ensuring it fits within the CC register width.
                    match channel {
                        PwmChannel::Channel0 => {
                            self.timer
                                .cc0_oc()
                                .write(|w| unsafe { w.bits(compare_value) });
                        }
                        PwmChannel::Channel1 => {
                            self.timer
                                .cc1_oc()
                                // SAFETY: as for channel 0 above -- `compare_value` is `top * percent / 100` with `percent <= 100` and
                                // `top <= 0xFFFF`, so it fits the 16-bit `OC` field.
                                .write(|w| unsafe { w.bits(compare_value) });
                        }
                        PwmChannel::Channel2 => {
                            self.timer
                                .cc2_oc()
                                // SAFETY: same value and same argument as the two channels above.
                                .write(|w| unsafe { w.bits(compare_value) });
                        }
                    }
                });

                Ok(())
            }

            /// Enable PWM output on a channel
            ///
            /// # Arguments
            ///
            /// * `channel` - PWM channel to enable
            ///
            /// # PWM Output Configuration
            ///
            /// This configures the CC channel for PWM mode with:
            /// - **CMOA (Compare Match Output Action)**: SET - Output goes high on compare match
            /// - **COFOA (Counter Overflow Output Action)**: TOGGLE - Output toggles on overflow/underflow
            ///
            /// For edge-aligned PWM (up-counting):
            /// - Counter counts from 0 to TOP
            /// - Output goes HIGH when CNT == CCx_OC (compare match)
            /// - Output goes LOW when CNT reaches TOP (counter overflow)
            /// - Duty cycle = (CCx_OC / TOP) × 100%
            ///
            /// For center-aligned PWM (up-down counting):
            /// - Counter counts 0→TOP→0
            /// - Output toggles on compare matches in both directions
            /// - Creates symmetric waveform
            ///
            /// **Note**: This configuration has been validated against EFR32MG24 reference manual
            /// section on Timer PWM generation. Hardware testing pending.
            pub fn enable_channel(&mut self, channel: PwmChannel) {
                critical_section::with(|_cs| {
                    match channel {
                        PwmChannel::Channel0 => {
                            // Configure channel mode in CC0_CFG
                            self.timer.cc0_cfg().write(|w| {
                                w.mode()
                                    .pwm() // PWM mode
                                    .coist()
                                    .clear_bit() // Output low when timer disabled
                            });
                            // Configure output action in CC0_CTRL
                            self.timer.cc0_ctrl().write(|w| {
                                w.outinv()
                                    .clear_bit() // Non-inverted
                                    .cofoa()
                                    .toggle() // Toggle on counter overflow
                                    .cmoa()
                                    .set_() // Set on compare match
                            });
                        }
                        PwmChannel::Channel1 => {
                            self.timer
                                .cc1_cfg()
                                .write(|w| w.mode().pwm().coist().clear_bit());
                            self.timer
                                .cc1_ctrl()
                                .write(|w| w.outinv().clear_bit().cofoa().toggle().cmoa().set_());
                        }
                        PwmChannel::Channel2 => {
                            self.timer
                                .cc2_cfg()
                                .write(|w| w.mode().pwm().coist().clear_bit());
                            self.timer
                                .cc2_ctrl()
                                .write(|w| w.outinv().clear_bit().cofoa().toggle().cmoa().set_());
                        }
                    }
                });
            }

            /// Disable PWM output on a channel
            ///
            /// # Arguments
            ///
            /// * `channel` - PWM channel to disable
            pub fn disable_channel(&mut self, channel: PwmChannel) {
                critical_section::with(|_cs| match channel {
                    PwmChannel::Channel0 => {
                        self.timer.cc0_cfg().write(|w| w.mode().off());
                    }
                    PwmChannel::Channel1 => {
                        self.timer.cc1_cfg().write(|w| w.mode().off());
                    }
                    PwmChannel::Channel2 => {
                        self.timer.cc2_cfg().write(|w| w.mode().off());
                    }
                });
            }

            /// Set raw duty cycle value for a PWM channel
            ///
            /// This provides direct access to the compare register for applications
            /// requiring precise control beyond percentage-based duty cycles.
            ///
            /// # Arguments
            ///
            /// * `channel` - PWM channel (0, 1, or 2)
            /// * `compare_value` - Raw compare value (0 to TOP)
            ///
            /// # Returns
            ///
            /// `Ok(())` on success, `Err(Error)` if compare value exceeds TOP
            ///
            /// # Example
            ///
            /// ```no_run
            /// // Set exact compare value for 33.3% duty cycle with TOP=30000
            /// timer.set_duty_raw(PwmChannel::Channel0, 10000)?;
            /// ```
            pub fn set_duty_raw(
                &mut self,
                channel: PwmChannel,
                compare_value: u32,
            ) -> Result<(), Error> {
                if compare_value > self.top_value {
                    return Err(Error::InvalidDutyCycle);
                }

                if !self.pwm_enabled {
                    return Err(Error::InvalidChannel);
                }

                critical_section::with(|_cs| {
                    // SAFETY: Compare value has been validated to be <= top_value (<=0xFFFF),
                    // ensuring it fits within the CC register width.
                    match channel {
                        PwmChannel::Channel0 => {
                            self.timer
                                .cc0_oc()
                                .write(|w| unsafe { w.bits(compare_value) });
                        }
                        PwmChannel::Channel1 => {
                            self.timer
                                .cc1_oc()
                                // SAFETY: `compare_value` was refused above when it exceeds `top_value`, and `top_value <= 0xFFFF` for this
                                // counter, so the 16-bit `OC` field cannot be truncated.
                                .write(|w| unsafe { w.bits(compare_value) });
                        }
                        PwmChannel::Channel2 => {
                            self.timer
                                .cc2_oc()
                                // SAFETY: same validated value as the two channels above.
                                .write(|w| unsafe { w.bits(compare_value) });
                        }
                    }
                });

                Ok(())
            }

            /// Enable timer overflow interrupt
            ///
            /// This enables the overflow/underflow interrupt, useful for periodic tasks
            /// synchronized with the timer period.
            pub fn listen_overflow(&mut self) {
                critical_section::with(|_cs| {
                    self.timer.ien().modify(|_, w| w.of().set_bit());
                });
            }

            /// Disable timer overflow interrupt
            pub fn unlisten_overflow(&mut self) {
                critical_section::with(|_cs| {
                    self.timer.ien().modify(|_, w| w.of().clear_bit());
                });
            }

            /// Check if overflow interrupt flag is set
            pub fn is_overflow(&self) -> bool {
                self.timer.if_().read().of().bit_is_set()
            }

            /// Clear overflow interrupt flag
            pub fn clear_overflow(&mut self) {
                critical_section::with(|_cs| {
                    // SAFETY: `alias::clear_window` gives the same block through its `_CLR` window, where writing 1 to `IF.OF`
                    // clears the flag; a plain write to `IF` would set it instead, which is the defect the alias exists to avoid.
                    // The base is this instance's own: hard-coding `Timer0S` here cleared TIMER0's flag from every other
                    // instance, which no single-timer instrument could have noticed (measured 2026-11-27 on TIMER1).
                    unsafe { crate::alias::clear_window(pac::$timerX_s::ptr()) }
                        .if_()
                        .write(|w| w.of().set_bit());
                });
            }

            /// The frequency the caller asked for. Not necessarily the one the timer produces -- see
            /// [`Self::get_actual_frequency`].
            pub fn get_frequency(&self) -> u32 {
                self.frequency
            }

            /// The frequency this timer really produces from the clock it was built with.
            ///
            /// A request that no prescaler and 16-bit `TOP` can express returns the nearest reachable bound here, and
            /// [`Self::get_frequency_error_ppm`] returns `None` to say so.
            pub fn get_actual_frequency(&self) -> u32 {
                self.actual_hz
            }

            /// How far the produced frequency is from the requested one, in parts per million.
            ///
            /// `None` means the request is outside what the fields can reach; `Some(0)` means it is exact.
            pub fn get_frequency_error_ppm(&self) -> Option<u32> {
                self.error_ppm
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
