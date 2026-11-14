//! The smallest BSP example: one LED, one module, no addresses.
//!
//! This is the BSP's hello world, and it is deliberately shorter than the HAL or PAC
//! versions of the same thing — the point of the facade is that a reader does not have to
//! know which pin the LED is on, or that it is active low, or that the user LED is the one
//! in PA7 and not the charging indicator next to it.
//!
//! # What it proves
//!
//! - `Module::take()` is enough to get a usable board — no Expansion Base needed.
//! - The LED blinks at 1 Hz, measured, not assumed: the loop is paced by the same
//!   calibrated delay the rest of the workspace uses.
//! - The example contains **no port, pin, register or address**. If it did, the facade
//!   would be leaking and the example would be a lie about what the BSP is for.
//!
//! # Hardware
//!
//! XIAO MG24 Sense on the XIAO Expansion Base (or bare): the user LED is on PA7, active
//! low (writing 0 lights it). Everything it needs is on the board.
//!
//! # Build and flash
//!
//! ```bash
//! cargo build --release -p xiao-mg24-bsp --features rt --example 00_blink
//! bash scripts/flash.sh \
//!   /projects/_cache/targets/thumbv8m.main-none-eabihf/release/examples/00_blink --confirm
//! ```
//!
//! Flashing goes through the guard script, never through a bare `probe-rs run`: the
//! script refuses an image that would touch the Gecko bootloader.

#![no_std]
#![no_main]

use panic_halt as _;
use xiao_mg24_bsp::hal::entry;
use xiao_mg24_bsp::hal::prelude::*;
use xiao_mg24_bsp::Module;

#[entry]
fn main() -> ! {
    let mut module = Module::take().expect("the peripherals are taken twice");

    // Half a second on, half a second off. `Delay` is the HAL's busy wait over SysTick, so
    // the period is what it says rather than what the compiler happened to emit. Taken
    // before the LED because it needs the board mutably and the LED borrows it.
    let mut delay = module.delay();
    let led = module.led();

    loop {
        led.on();
        delay.delay_ms(500);
        led.off();
        delay.delay_ms(500);
    }
}
