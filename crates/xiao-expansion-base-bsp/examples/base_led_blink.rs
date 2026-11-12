//! Blink through the carrier facade: the ladder's top rung.
//!
//! ```text
//! pac_led_blink        registers and a delay loop
//! hal_led_blink        an Output pin and a Delay
//! module_led_blink     Module::take(), no base needed
//! base_led_blink       this: ExpansionBase::take(), which also proves the base initialises
//! ```
//!
//! Why a blink belongs in the carrier crate when the LED is on the module: it is the cheapest
//! end-to-end check that the *facade* works. `ExpansionBase::take()` prepares two buses and
//! takes the peripherals; if it returns and the LED blinks, the base's wiring did not break
//! anything on the way. It is the first thing to flash after touching the facade.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use xiao_expansion_base_bsp::hal::prelude::*;
use xiao_expansion_base_bsp::ExpansionBase;

#[path = "common/mailbox.rs"]
mod mailbox;

#[entry]
fn main() -> ! {
    let mut base = ExpansionBase::take().expect("peripherals taken twice");
    let mut delay = base.delay();
    let led = base.led();

    let found = base.probe();
    mailbox::write(
        if found.oled && found.environment && found.rtc {
            mailbox::Status::Pass
        } else {
            mailbox::Status::Fail
        },
        if found.oled && found.environment && found.rtc {
            b"base: 3 devices answered, LED blinking"
        } else {
            b"base: not every device answered"
        },
    );

    loop {
        led.on();
        delay.delay_ms(500);
        led.off();
        delay.delay_ms(500);
    }
}
