//! Blink at the HAL's level: one pin primitive, no board.
//!
//! This is the fourth and lowest-but-one rung of the same demo, and it exists so the ladder can
//! be read side by side:
//!
//! ```text
//! pac_led_blink        registers: MODEL, DOUT, a delay loop
//! hal_led_blink        this: an Output pin and a Delay, no register
//! module_led_blink     the same through Module::take()
//! base_led_blink       the same through the carrier's facade
//! ```
//!
//! The HAL version is where the interesting part becomes visible: `Output::new` takes the port,
//! the pin **and the polarity**, because an LED wired to ground lights when the pin sinks
//! current. Writing that down once is what stops every caller from rediscovering it.
//!
//! Nothing here needs the Expansion Base: PA7 is on the module.

#![no_std]
#![no_main]

use efr32mg24_hal::board::Board;
use efr32mg24_hal::entry;
use efr32mg24_hal::pin::Port;
use efr32mg24_hal::prelude::*;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

#[entry]
fn main() -> ! {
    let board = Board::take().expect("peripherals taken twice");
    let mut delay = efr32mg24_hal::delay::Delay::with_sysclk(
        // SAFETY: SysTick is the delay's alone; nothing else in this example programs it.
        unsafe { cortex_m::Peripherals::steal().SYST },
        efr32mg24_hal::clock::DEFAULT_SYSCLK_HZ,
    );

    // PA7, active low: the same two facts `module_led_blink` hides behind the facade.
    let led = board.output_on(Port::A, 7, true);

    mailbox::write(
        mailbox::Status::Pass,
        b"hal: pin primitive drives PA7, 1 Hz",
    );

    loop {
        led.on();
        delay.delay_ms(500);
        led.off();
        delay.delay_ms(500);
    }
}
