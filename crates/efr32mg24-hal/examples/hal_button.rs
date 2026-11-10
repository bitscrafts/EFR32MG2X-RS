//! Button + LED: press the user button on the base, the LED follows.
//!
//! The smallest example that exercises the HAL's board layer end to end:
//! `board.button()` (PC01, input with pull-up) and `board.led()` (PA7, active
//! low). No register, no pin number, no PAC type appears below.
//!
//! Verification without pressing anything: at start-up the pin must read
//! *released*, and the example reports that through the mailbox. A held button
//! at boot would read as pressed, which is a different (and visible) result.

#![no_std]
#![no_main]

use efr32mg24_hal::board::Board;
use efr32mg24_hal::entry;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

#[entry]
fn main() -> ! {
    let board = match Board::take() {
        Some(b) => b,
        None => loop {
            cortex_m::asm::nop();
        },
    };
    let button = board.button();
    let led = board.led();

    let pressed_at_boot = button.is_pressed();
    if pressed_at_boot {
        mailbox::fail(b"button held at boot (or PC01 stuck low)");
    } else {
        mailbox::pass(b"button released at boot, LED follows");
    }

    let mut last = pressed_at_boot;
    let mut presses: u32 = 0;
    loop {
        let now = button.is_pressed();
        if now {
            led.on();
        } else {
            led.off();
        }
        if now && !last {
            presses = presses.wrapping_add(1);
            // Report the count in place of the boot line, so a reader sees that
            // the edge was recognised rather than just the level.
            if presses == 1 {
                mailbox::pass(b"first press seen");
            }
        }
        last = now;
        // A short poll interval: long enough to keep the loop cheap, short
        // enough that no human press is missed.
        cortex_m::asm::delay(20_000);
    }
}
