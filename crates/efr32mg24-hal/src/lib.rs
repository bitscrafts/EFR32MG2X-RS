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
//! **Phase C Stage 1 Complete**: All essential, communication, and advanced peripherals (Stage 1) production-ready
//!
//! ## Phase A - Essential Peripherals (Complete)
//!
//! - **Clock Management (CMU)** - HFXO/HFRCO configuration with hardware register access
//! - **GPIO** - Type-safe pin modes with hardware register manipulation
//! - **Delay** - SysTick-based delays integrated with CMU
//!
//! ## Phase B - Communication Peripherals (Complete)
//!
//! - **USART** - Serial communication (USART0) with embedded-hal-nb traits ✅
//! - **I2C** - I2C master mode (I2C0, I2C1) with embedded-hal traits ✅
//! - **SPI** - SPI master mode (USART in SPI mode) with embedded-hal traits ✅
//! - **Timer** - Timer and PWM (TIMER0-4) with 3 channels each ✅
//!
//! ## Phase C - Advanced Peripherals (Stage 1 Complete)
//!
//! - **ADC (IADC)** - Analog-to-digital converter ✅
//! - **DMA (LDMA)** - Direct Memory Access controller (Stage 1) ✅
//! - **EMU** - Energy Management Unit (EM0-EM2 transitions) ✅
//! - **RTC** - Real-Time Clock (SYSRTC0) ✅
//! - **WDOG** - Watchdog Timer (WDOG0) ✅
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
pub use efr32mg24_pac as pac;

// HAL modules
pub mod adc;
#[cfg(feature = "rt")]
pub use cortex_m_rt::entry;

pub mod board;
pub mod bus;
pub mod buzzer;
pub mod clock;
pub mod delay;
pub mod display;
pub mod dma;
pub mod emu;
pub mod i2c;
pub mod microphone;
pub mod pin;
pub mod pwm;
pub mod rtc;
pub mod sensors;
pub mod spi;
pub mod timer;
pub mod usart;
pub mod wdog;

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

    /// Re-export embedded-hal SPI trait
    pub use embedded_hal::spi::SpiBus as _embedded_hal_spi_SpiBus;
}
