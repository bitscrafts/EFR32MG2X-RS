//! VU meter through the HAL: microphone level drives the LED brightness.
//!
//! The HAL twin of `pac_mic_vu`. What the example does not mention is the whole
//! analog chain: no IADC register, no timebase, no PC9, no clock selection. It
//! asks the board for the converter and the microphone driver for a level.
//!
//! Verification: the mailbox reports RMS, peak and sample count. A working chain
//! always produces samples; silence is a small level, not zero samples. Zero
//! samples means the converter never delivered and the report says so.

#![no_std]
#![no_main]

use efr32mg24_hal::board::Board;
use efr32mg24_hal::bus::Bus;
use efr32mg24_hal::display::format_fixed;
use efr32mg24_hal::entry;
use efr32mg24_hal::microphone::Microphone;
use efr32mg24_hal::pin::Port;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Software PWM steps per refresh.
const PWM_STEPS: u32 = 100;

#[entry]
fn main() -> ! {
    let board = match Board::take() {
        Some(b) => b,
        None => loop {
            cortex_m::asm::nop();
        },
    };
    let led = board.output_on(Port::A, 7, true);

    // Board wiring: microphone powered, analog pin handed to the ADC, IADCCLK on.
    board.output_on(Port::C, 8, false).on();
    let adc = board.converter();

    let mic = match Microphone::new(adc) {
        Ok(m) => m,
        Err(e) => {
            let msg: &[u8] = match e {
                efr32mg24_hal::microphone::MicError::StuckDisabling => {
                    b"mic: adc stuck in DISABLING (power cycle needed)"
                }
                efr32mg24_hal::microphone::MicError::ConversionsWontStop => {
                    b"mic: conversions will not stop"
                }
                efr32mg24_hal::microphone::MicError::WriteNotPublished => {
                    b"mic: configuration write never published"
                }
            };
            mailbox::fail(msg);
            loop {
                led.toggle();
                cortex_m::asm::delay(2_000_000);
            }
        }
    };

    let mut reported = false;
    let mut peak_seen = 0u32;

    loop {
        let level = mic.level(128);
        if level.peak > peak_seen {
            peak_seen = level.peak;
        }

        // Brightness follows the level: quiet room -> almost dark, clap -> full.
        let duty = core::cmp::min(PWM_STEPS, (level.rms * PWM_STEPS) / 200);
        for step in 0..PWM_STEPS {
            if step < duty {
                led.on();
            } else {
                led.off();
            }
            cortex_m::asm::delay(300);
        }

        if !reported {
            let mut msg = [0u8; 64];
            let mut pos = 0usize;
            let head = b"hal mic rms=";
            msg[..head.len()].copy_from_slice(head);
            pos += head.len();
            pos += format_fixed(&mut msg[pos..], level.rms as i32 * 100, 0);
            let mid = b" pk=";
            msg[pos..pos + mid.len()].copy_from_slice(mid);
            pos += mid.len();
            pos += format_fixed(&mut msg[pos..], peak_seen as i32 * 100, 0);
            let tail = b" n=";
            msg[pos..pos + tail.len()].copy_from_slice(tail);
            pos += tail.len();
            pos += format_fixed(&mut msg[pos..], level.samples as i32 * 100, 0);

            if level.samples > 0 {
                mailbox::pass(&msg[..pos]);
            } else {
                mailbox::fail(&msg[..pos]);
            }
            reported = true;
        }
    }
}
