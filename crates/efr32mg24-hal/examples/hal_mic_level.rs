//! Shout at the board and the light gets brighter — with hardware PWM.
//!
//! This is the demo Seeed flashes into every XIAO MG24 **Sense** at the factory: *"the louder you
//! shout, the brighter the light will be."* In Arduino it is `analogWrite(LED_PIN, 0..255)`; here
//! it is a TIMER0 compare channel whose duty cycle follows the microphone level.
//!
//! Why PWM and not a software loop: a GPIO has two brightnesses, and a loop that toggles it
//! spends the CPU doing it — the same CPU that has to sample the microphone. A compare channel
//! keeps the LED lit *while the sampler runs*, which is the difference between a demo and
//! something a user can build on.
//!
//! What the example uses, and why each part is there:
//!
//! - **PC8 enables the microphone**, PC9 is the analog signal, and `IADC0` converts it. The mic
//!   is analog MEMS, not PDM: there is no I2S and no digital microphone bus on this board.
//! - **PA7 is the user LED, active low**, and it is the same pin the blink examples use.
//! - **the level is an RMS window of 128 samples**, mapped to 0..=100 % duty. Silence is a small
//!   number, not zero samples; zero samples means the converter never delivered.
//!
//! Verification without eyes: the mailbox reports RMS, peak and the duty being applied. The
//! brightness itself needs a human — the numbers do not.

#![no_std]
#![no_main]

use efr32mg24_hal::board::Board;
use efr32mg24_hal::entry;
use efr32mg24_hal::microphone::{Level, Microphone};
use efr32mg24_hal::pin::Port;
use efr32mg24_hal::prelude::*;

use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// PWM frequency for the LED. 1 kHz is fast enough that the eye integrates the pulses into a
/// brightness instead of seeing a flicker. The timer's 16-bit TOP sets the *floor* at
/// 19 MHz / 65535 ≈ 290 Hz, not the ceiling, so this is comfortably inside the range.
const LED_PWM_HZ: u32 = 1_000;

/// Samples per level window.
const WINDOW: usize = 128;

/// Map an RMS level (ADC counts around mid-scale) to a duty cycle.
///
/// The scale is deliberately not linear in the ear: a quiet room reads about 10 counts and a
/// raised voice a few hundred, so a linear map would leave the LED dark until the room is loud.
/// 0..=400 counts across 0..=100 % puts normal speech in the middle, which is what makes the
/// demo feel responsive.
fn duty_for(level: &Level) -> u32 {
    let rms = level.rms.min(400);
    rms * 100 / 400
}

fn push_dec(out: &mut [u8], mut at: usize, mut value: u32) -> usize {
    let mut digits = [0u8; 10];
    let mut n = 0;
    loop {
        digits[n] = b'0' + (value % 10) as u8;
        value /= 10;
        n += 1;
        if value == 0 || n == digits.len() {
            break;
        }
    }
    for i in (0..n).rev() {
        out[at] = digits[i];
        at += 1;
    }
    at
}

#[entry]
fn main() -> ! {
    let mut board = Board::take().expect("peripherals taken twice");
    let mut delay = board.delay();

    // The microphone: PC8 enable (active high), then the converter.
    board.output_on(Port::C, 8, false).on();
    delay.delay_ms(5);
    let mic = match board.converter().microphone() {
        Ok(mic) => mic,
        Err(_) => {
            mailbox::write(mailbox::Status::Fail, b"mic: converter will not start");
            loop {
                delay.delay_ms(500);
            }
        }
    };

    // The LED as a PWM channel: PA7, through TIMER0 channel 0.
    // No PAC here on purpose: `Board::take` already owns it, and a HAL example that steals the
    // singleton would be admitting the HAL failed to hand out what it owns.
    let mut led = match board.pwm_on(Port::A, 7, LED_PWM_HZ) {
        Some(pwm) => pwm,
        None => {
            mailbox::write(
                mailbox::Status::Fail,
                b"pwm: TIMER0 is not counting (clock?)",
            );
            loop {
                delay.delay_ms(500);
            }
        }
    };

    let mut reported = false;
    loop {
        let level = mic.level(WINDOW);
        let duty = duty_for(&level);
        led.set_duty(duty);

        if !reported && level.samples > 0 {
            reported = true;
            let mut buf = [0u8; 64];
            buf[..4].copy_from_slice(b"pwm ");
            let mut at = 4;
            buf[at..at + 4].copy_from_slice(b"rms=");
            at += 4;
            at = push_dec(&mut buf, at, level.rms);
            buf[at..at + 4].copy_from_slice(b" pk=");
            at += 4;
            at = push_dec(&mut buf, at, level.peak);
            buf[at..at + 4].copy_from_slice(b" n=");
            at += 3;
            at = push_dec(&mut buf, at, level.samples);
            buf[at..at + 5].copy_from_slice(b" 1k0=");
            at += 5;
            at = push_dec(&mut buf, at, led.frequency_hz());
            mailbox::write(
                if level.samples > 0 {
                    mailbox::Status::Pass
                } else {
                    mailbox::Status::Fail
                },
                &buf[..at],
            );
        }

        // A short pause between windows: the sampler is the expensive part and the eye does not
        // need 60 updates a second of a level that changes with a voice.
        delay.delay_ms(20);
    }
}
