//! BLE from the board crate: a module user adds one dependency and gets a beacon.
//!
//! This is the rung of the ladder that was missing. The radio crate's examples prove the stack; this
//! one proves the *combination* — a user who has already taken `Module` for the LED and the sensors can
//! add Bluetooth without learning anything about buses, pins or the radio's own clock requirements.
//!
//! ## Why the radio crate is a dev-dependency
//!
//! `xiao-mg24-bsp` must not depend on `efr32mg24-radio`: that would pull three MSLA-licensed binary
//! blobs into every user of a GPIO. It is a **dev**-dependency, so the dependency exists for this
//! example and for nothing else — which is also a statement about the cost: a product that wants BLE
//! opts in, and a product that does not never links a megabyte of vendor code.
//!
//! ## What each line owns
//!
//! ```text
//! Module::take()        boot, clocks, the PAC singleton
//! module.delay()        the calibrated delay the stack asks for while it boots
//! Bluetooth::beacon()   the stack: HFXO, controller reset, advertising data
//! module.led()          the proof that the program is still in its loop
//! ```
//!
//! Nothing here names a pin, an address or a register — which is what the module crate is for.

#![no_std]
#![no_main]

use panic_halt as _;
use xiao_mg24_bsp::hal::prelude::*;
use xiao_mg24_bsp::{entry, Module};

#[path = "common/mailbox.rs"]
mod mailbox;

#[entry]
fn main() -> ! {
    let mut module = Module::take().expect("peripherals taken twice");
    let mut delay = module.delay();
    let led = module.led();

    let mut ble = match efr32mg24_radio::Bluetooth::beacon(&mut delay, "xiao-mg24-bsp") {
        Ok(ble) => ble,
        Err(_) => {
            mailbox::write(
                mailbox::Status::Fail,
                b"module ble: stack did not boot (crystal or blobs)",
            );
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    mailbox::write(
        mailbox::Status::Pass,
        b"module ble: advertising as xiao-mg24-bsp",
    );

    // One blink per event: a phone that connects or reads is visible on the board without a host.
    loop {
        if ble.poll_event().is_some() {
            led.on();
            delay.delay_ms(30);
            led.off();
        } else {
            cortex_m::asm::wfi();
        }
    }
}
