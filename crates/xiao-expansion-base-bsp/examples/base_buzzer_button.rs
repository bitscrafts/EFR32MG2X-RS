//! The base's buzzer and button, through the facade: the two devices with no protocol.
//!
//! Every other device on the Expansion Base is an I2C conversation. These two are not, and
//! they are the ones where an abstraction earns its keep differently: the buzzer is a timer
//! channel routed to a pin, and the button is a pin with a pull-up — both of which are easy
//! to get subtly wrong and impossible to notice without a measurement.
//!
//! Measured facts this example relies on, and which the facade encodes:
//!
//! - the buzzer is on `PC03`, driven by `TIMER0` `CC0`, and the tone comes from the timer's
//!   period: 2700 Hz needs `TOP = 3518` after calibrating against the CPU clock;
//! - silence is `CC0_CTRL.CMOA = None`, not stopping the timer — stopping it and restarting
//!   it left the first cycle at the wrong level, which sounds like a click;
//! - the button is on `PC01`, shorts to ground, and reads high at rest.
//!
//! Turn your back on it: the LED follows the button and the buzzer answers it, so the board
//! says which of the two is broken.

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
    let button = base.button();

    // The buzzer is optional by type: `None` means the timer never reported a running
    // counter, which is a clock problem and not a missing buzzer.
    let mut buzzer = base.buzzer();

    let pressed_at_boot = button.is_low();
    let tone_ok = buzzer.is_some();
    mailbox::write(
        if tone_ok {
            mailbox::Status::Pass
        } else {
            mailbox::Status::Fail
        },
        if pressed_at_boot {
            b"base: button held at boot, buzzer ok"
        } else {
            b"base: button released at boot, buzzer ok"
        },
    );

    let mut was_pressed = pressed_at_boot;
    let mut beeps: u32 = 0;

    loop {
        let pressed = button.is_low();
        if pressed != was_pressed {
            was_pressed = pressed;
            if pressed {
                // A short beep per press: proof both devices answer without a terminal.
                beeps = beeps.wrapping_add(1);
                if let Some(b) = buzzer.as_mut() {
                    b.sound();
                    delay.delay_ms(80);
                    b.silence();
                }
            }
        }
        // The LED follows the button: on while held.
        if pressed {
            led.on();
        } else {
            led.off();
        }
        delay.delay_ms(10);
    }
}
