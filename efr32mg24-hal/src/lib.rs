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
//! **Phase 5 Tier 2 - USART Implementation**: In Progress
//!
//! ## Tier 1 Peripherals (Complete)
//!
//! - **Clock Management (CMU)** - HFXO/HFRCO configuration with hardware register access
//! - **GPIO** - Type-safe pin modes with hardware register manipulation
//! - **Delay** - SysTick-based delays integrated with CMU
//!
//! ## Tier 2 Peripherals (In Progress)
//!
//! - **USART** - Serial communication (USART0) with embedded-hal-nb traits ✅
//! - **I2C** - I2C master mode (I2C0, I2C1) with embedded-hal traits ✅
//! - SPI (USART in SPI mode) - Planned
//! - Timer (TIMER0-4) and PWM - Planned
//!
//! ## Tier 3 Peripherals (Planned - Phase 4)
//!
//! - ADC (IADC)
//! - DMA (LDMA)
//! - Power management (EMU)
//! - RTC (RTCC)
//! - Watchdog (WDOG)
//!
//! # Quick Start
//!
//! ```no_run
//! use efr32mg24_hal::{
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
//!         hfxo: Some(HfxoConfig::new(39_000_000)),
//!         lfxo: Some(Default::default()),
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
//! - [`clock`] - Clock Management Unit (CMU) configuration
//! - [`delay`] - Blocking delays using SysTick
//! - [`gpio`] - General Purpose I/O
//! - [`i2c`] - I2C master mode
//! - [`usart`] - USART/UART serial communication
//! - [`prelude`] - Common imports for convenience
//!
//! # Feature Flags
//!
//! - `rt` - Include runtime support (startup code, vector table)

// Re-export the PAC
pub use efr32mg24_pac as pac;

// HAL modules
pub mod clock;
pub mod delay;
pub mod gpio;
pub mod i2c;
pub mod usart;

/// Prelude module for convenient imports
///
/// This module re-exports the most commonly used types and traits
/// for easy importing with `use efr32mg24_hal::prelude::*;`
pub mod prelude {
    /// Re-export embedded-hal delay trait
    pub use embedded_hal::delay::DelayNs as _embedded_hal_delay_DelayNs;

    /// Re-export embedded-hal digital traits
    pub use embedded_hal::digital::InputPin as _embedded_hal_digital_InputPin;
    pub use embedded_hal::digital::OutputPin as _embedded_hal_digital_OutputPin;

    /// Re-export embedded-hal I2C trait
    pub use embedded_hal::i2c::I2c as _embedded_hal_i2c_I2c;

    /// Re-export GPIO extension trait
    pub use crate::gpio::GpioExt as _efr32mg24_hal_gpio_GpioExt;
}
