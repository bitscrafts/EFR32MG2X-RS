#![no_std]
#![doc = include_str!("../README.md")]

//! Board Support Package for Seeed Studio XIAO MG24 Sense
//!
//! Provides a high-level interface to the XIAO MG24 Sense board peripherals.
//!
//! # Features
//!
//! - Pin definitions for all 19 XIAO GPIOs
//! - Board initialization (clocks, peripherals)
//! - Pre-configured peripheral instances (USART, I2C, SPI)
//! - Radio protocol implementations (BLE 5.3 available, Zigbee/Thread future)
//!
//! # Quick Start
//!
//! ```no_run
//! use xiao_mg24_bsp::Board;
//! use cortex_m_rt::entry;
//!
//! #[entry]
//! fn main() -> ! {
//!     // Initialize board with default configuration
//!     let board = Board::take().unwrap();
//!
//!     // Use pre-configured GPIO pins
//!     let mut led = board.pins.d0.into_output();
//!
//!     loop {
//!         led.set_high();
//!         cortex_m::asm::delay(1_000_000);
//!         led.set_low();
//!         cortex_m::asm::delay(1_000_000);
//!     }
//! }
//! ```
//!
//! # Board Configuration
//!
//! - **MCU**: EFR32MG24B220F1536IM48-B
//! - **CPU**: ARM Cortex-M33 @ 78 MHz with FPU and TrustZone-M
//! - **Flash**: 1536 KB
//! - **RAM**: 256 KB
//! - **Radio**: 2.4 GHz (Bluetooth 5.3, Zigbee, Thread, Matter)
//! - **Package**: 48-pin QFN

pub mod board;
pub mod pins;
#[cfg(feature = "ble")]
pub mod radio;
#[cfg(feature = "microphone")]
pub mod sensors;

// Re-export for convenience
pub use board::Board;
pub use pins::Pins;

// Re-export HAL types
pub use efr32mg24_hal as hal;
pub use efr32mg24_pac as pac;
