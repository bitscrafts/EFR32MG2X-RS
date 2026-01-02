//! Blink LED example for XIAO MG24 Sense
//!
//! Demonstrates basic board initialization and GPIO usage.
//!
//! # Hardware Setup
//!
//! - Built-in LED (PB02, D10) will blink at 1 Hz
//! - No external hardware required
//!
//! # Expected Behavior
//!
//! - LED blinks ON for 500ms, OFF for 500ms
//! - LED is active-low (LOW = ON, HIGH = OFF)
//!
//! # Build & Flash
//!
//! ```bash
//! cargo build --example blink --release
//! probe-rs run --chip EFR32MG24B220F1536IM48 target/thumbv8m.main-none-eabihf/release/examples/blink
//! ```

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use panic_halt as _;
use xiao_mg24_bsp::{hal, Board};

#[entry]
fn main() -> ! {
    // Initialize board with default configuration
    // - HFXO @ 39 MHz
    // - SYSCLK @ 39 MHz
    let board = Board::take().unwrap();

    // Split board into components
    let (pins, clocks, core) = board.split();

    // Configure built-in LED pin (D10/PB02, active low)
    let mut led = pins.d10.into_push_pull_output();

    // Create delay provider
    let mut delay = hal::delay::Delay::new(core.SYST, &clocks);

    loop {
        // LED ON (active low)
        led.set_low().ok();
        delay.delay_ms(500);

        // LED OFF
        led.set_high().ok();
        delay.delay_ms(500);
    }
}
