#![doc = include_str!("../docs/crate-doc.md")]
#![no_std]
#![deny(missing_docs)]

//! Hardware Abstraction Layer for EFR32MG24 microcontrollers
//!
//! This crate provides high-level, safe abstractions over EFR32MG24 peripherals
//! with full embedded-hal v1.0 trait implementations.
//!
//! # Target Hardware
//!
//! Primary development target: **Seeed Studio XIAO MG24 Sense**
//! - Chip: EFR32MG24B220F1536IM48-B
//! - Core: ARM Cortex-M33 @ 78 MHz
//! - Memory: 1536 KB Flash, 256 KB RAM
//! - Radio: 2.4 GHz (Matter, Thread, Zigbee, BLE 5.3)
//!
//! # Status
//!
//! A **minimal working version**: every peripheral family that has a driver is exercised by an example, and the
//! examples that make a claim carry a record read back off the board. What is proven and what is not is written where
//! the evidence lives — the crate `README.md` next to this one — and the assessment that would precede a publication
//! is `docs/04-reference/HAL-MATURITY-2026-11-25.md` in the repository.
//!
//! - **Proven on the part**: GPIO and routing, delays, the bit-banged bus, I2C master (blocking and async), LDMA,
//!   TIMER and the SYSRTC compare channel, WDOG, EMU voltage scaling, the IADC with the analogue microphone, the IMU
//!   over I2C, and the HFXO (39 MHz, with a 1000 ms delay measured at 1006 ms by an independent clock).
//! - **Compiled, and waiting for a measured example**: SPI and USART. Both drivers exist and build; neither has been
//!   watched moving a byte on the board yet, and this crate does not claim otherwise.
//! - **Not implemented**: GPIO interrupts, I2C slave mode, SPI slave mode, DMA linked descriptors.
//!
//! Two notes that belong with the status rather than in a bug tracker: the `dwt` module is an **instrument**, not a
//! time source — its counter only runs while a debug session is attached, and [`dwt::start`] returns an error when it
//! does not; and `embassy` support lives in its own crate, not here.
//!
//! # Quick Start
//!
//! ```no_run
//! use efr32mg2x_hal::{
//!     clock::{Clocks, ClockConfig, HfxoConfig},
//!     delay::Delay,
//!     pac,
//!     prelude::*,
//! };
//!
//! // Get peripheral singletons
//! let cp = cortex_m::Peripherals::take().unwrap();
//! let dp = pac::Peripherals::take().unwrap();
//!
//! // Configure clocks with XIAO MG24's 39 MHz crystal
//! let clocks = Clocks::new(
//!     dp.cmu_s,
//!     ClockConfig {
//!         hfxo: None,
//!         lfxo: None,
//!     }
//! ).freeze();
//!
//! // Create delay provider
//! let mut delay = Delay::new(cp.SYST, &clocks);
//!
//! loop {
//!     // Your application code here
//!     delay.delay_ms(1000);
//! }
//! ```
//!
//! # Module Organization
//!
//! - [`adc`] - Analog-to-Digital Converter (IADC)
//! - [`clock`] - Clock Management Unit (CMU) configuration
//! - [`delay`] - Blocking delays using SysTick
//! - [`dma`] - Direct Memory Access (LDMA) controller
//! - [`emu`] - Energy Management Unit (power modes)
//! - [`gpio`] - General Purpose I/O
//! - [`i2c`] - I2C master mode
//! - [`rtc`] - Real-Time Clock (SYSRTC0)
//! - [`spi`] - SPI master mode (USART in SPI mode)
//! - [`timer`] - Timer and PWM functionality (TIMER0-4)
//! - [`usart`] - USART/UART serial communication
//! - [`wdog`] - Watchdog Timer (WDOG0)
//! - [`prelude`] - Common imports for convenience
//!
//! # Radio/Wireless Features
//!
//! Radio features (BLE, Thread, Zigbee, Matter) are provided by separate protocol
//! stack crates that sit above the HAL layer:
//!
//! - `efr32mg24-bluetooth-sys` - Bluetooth Low Energy 5.3 (BGAPI stack)
//! - `efr32mg24-rail-sys` - Radio Abstraction Interface Layer
//!
//! See those crates for wireless examples and documentation.
//!
//! # Feature Flags
//!
//! - `rt` - Include runtime support (startup code, vector table)

// Re-export the PAC
pub use efr32mg2x_pac as pac;

// HAL modules
pub mod adc;
mod alias;
#[cfg(feature = "rt")]
pub use cortex_m_rt::entry;

pub mod board;
pub mod bus;
pub mod buzzer;
pub mod clock;
pub mod cores;
pub mod delay;
pub mod display;
pub mod dma;
pub mod dwt;
pub mod emu;
pub mod i2c;
pub mod microphone;
pub mod mvp;
pub mod pin;
pub mod pwm;
pub mod route;
pub mod rtc;
pub mod sensors;
pub mod sleep;
pub mod spi;
pub mod sysrtc;
pub mod timer;
pub mod usart;
pub mod wdog;

/// Prelude module for convenient imports
///
/// This module re-exports the most commonly used types and traits
/// for easy importing with `use efr32mg2x_hal::prelude::*;`
pub mod prelude {
    /// Re-export embedded-hal delay trait
    pub use embedded_hal::delay::DelayNs as _embedded_hal_delay_DelayNs;

    /// Re-export embedded-hal digital traits
    pub use embedded_hal::digital::InputPin as _embedded_hal_digital_InputPin;
    pub use embedded_hal::digital::OutputPin as _embedded_hal_digital_OutputPin;

    /// Re-export embedded-hal I2C trait
    pub use embedded_hal::i2c::I2c as _embedded_hal_i2c_I2c;

    /// Re-export embedded-hal SPI trait
    pub use embedded_hal::spi::SpiBus as _embedded_hal_spi_SpiBus;
}
