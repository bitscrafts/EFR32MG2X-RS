//! Clock Management Unit (CMU) configuration
//!
//! This module provides type-safe configuration of the EFR32MG24's clocking system.
//!
//! # Overview
//!
//! The EFR32MG24 has multiple clock sources:
//! - **HFXO**: High Frequency Crystal Oscillator (external crystal, typically 39 MHz)
//! - **HFRCO**: High Frequency RC Oscillator (internal, 19 MHz default)
//! - **LFXO**: Low Frequency Crystal Oscillator (external 32.768 kHz crystal)
//! - **LFRCO**: Low Frequency RC Oscillator (internal, 32.768 kHz)
//!
//! # Phase 2 Implementation Status
//!
//! Hardware register access is partially implemented:
//! - HFXO clock source selection via SYSCLKCTRL register
//! - Clock frequency tracking
//! - Safe peripheral consumption pattern
//!
//! # Examples
//!
//! ## Using External Crystal (XIAO MG24 Default)
//!
//! ```no_run
//! use efr32mg24_hal::{
//!     clock::{Clocks, ClockConfig, HfxoConfig, LfxoConfig},
//!     pac,
//! };
//!
//! let dp = pac::Peripherals::take().unwrap();
//!
//! // Configure clocks with external 39 MHz crystal (XIAO MG24 default)
//! let clocks = Clocks::new(
//!     dp.cmu_s,
//!     ClockConfig {
//!         hfxo: Some(HfxoConfig::new(39_000_000)),
//!         lfxo: Some(LfxoConfig::default()),  // 32.768 kHz
//!     }
//! );
//!
//! let frozen_clocks = clocks.freeze();
//!
//! // Use frozen clocks with peripherals
//! let sysclk_freq = frozen_clocks.sysclk().0;
//! ```
//!
//! ## Using Internal RC Oscillators
//!
//! ```no_run
//! use efr32mg24_hal::{clock::{Clocks, ClockConfig}, pac};
//!
//! let dp = pac::Peripherals::take().unwrap();
//!
//! // Use default internal oscillators (HFRCO @ 19 MHz, LFRCO @ 32.768 kHz)
//! let clocks = Clocks::new(dp.cmu_s, ClockConfig::default());
//! let frozen = clocks.freeze();
//! ```
//!
//! # Module Structure
//!
//! - `types.rs` - Type definitions (Hertz, HfxoConfig, LfxoConfig, ClockConfig)
//! - `clocks.rs` - Clocks implementation with hardware register access
//! - `frozen.rs` - FrozenClocks wrapper for immutable clock reference
//! - `mod.rs` - Module coordinator and public API

mod clocks;
mod frozen;
mod types;

// Re-export public types
pub use clocks::Clocks;
pub use frozen::FrozenClocks;
pub use types::{ClockConfig, Hertz, HfxoConfig, LfxoConfig};
