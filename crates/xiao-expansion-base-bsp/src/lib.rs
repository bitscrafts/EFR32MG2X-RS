//! The Seeed XIAO Expansion Base, as a board.
//!
//! This crate describes the *carrier* that stacks underneath a XIAO module. It exists
//! separately from `xiao-mg24-bsp` for one reason: a user may own the module and not the
//! base, and in that case none of this code should exist for them — not as an `Option`, not
//! as a runtime check, but as a crate they do not depend on.
//!
//! # What is on the base
//!
//! ```text
//! OLED             SSD1306, 128x64, address 0x3C
//! environmental    BME680/688, address 0x76 (or 0x77)
//! real-time clock  PCF8563, address 0x51
//! buzzer           PC03, piezo, driven by TIMER0 CC0
//! user button      PC01, shorts to ground, pulled up
//! Grove            UART (PC06/PC07), A0 (PC00), and the same I2C bus as the panel
//! microSD          CS PC02, SPI on PA03/PA04/PA05        (not implemented yet)
//! ```
//!
//! Those three I2C devices share one bus on `PC04`/`PC05`, with pull-ups on the base.
//!
//! # What is *not* here
//!
//! The IMU and the microphone on the Sense module, the LED, and the clock tree: those belong
//! to the module crate. This crate depends on it, never the other way round.
//!
//! # Example
//!
//! ```no_run
//! use xiao_expansion_base_bsp::ExpansionBase;
//!
//! let mut base = ExpansionBase::take().expect("the board is taken twice");
//! let mut oled = base.oled().expect("panel");
//! oled.clear();
//! oled.text(0, 0, b"hello");
//! oled.flush();
//! ```

#![no_std]

pub mod expansion_base;

pub use expansion_base::{Devices, ExpansionBase};

pub use efr32mg24_hal as hal;
pub use efr32mg24_pac as pac;
pub use xiao_mg24_bsp as module;
