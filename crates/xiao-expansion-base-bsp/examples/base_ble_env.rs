//! The product shape: a carrier reading a sensor and advertising the reading.
//!
//! Every BLE example before this one advertises a constant — a name, a frame with an uptime. This one
//! closes the loop: the BME68x on the carrier is read, and the numbers go out in the advertising
//! payload. It is the smallest thing that is genuinely a *product*: a sensor node that a phone can read
//! without connecting, without pairing and without an app.
//!
//! ## The payload, and the budget
//!
//! ```text
//! flags                                 3
//! manufacturer header + company id      4
//! frame: version 1 + temp 2 + press 2 + rh 2 + sequence 2   9
//! ----------------------------------------------------------
//! total                                16 of 31 bytes
//! ```
//!
//! Sixteen bytes is comfortable, which is why a sensor beacon fits: the values are fixed-point
//! integers, not text. `-1234` centi-°C is two bytes; `"-12.34"` is six, and the receiver would have to
//! parse it.
//!
//! ## What each layer contributes
//!
//! ```text
//! ExpansionBase::environment()   which device, which bus, which address
//! Bme68x::read_with()            the calibration maths and the conversion
//! Bluetooth::set_manufacturer_data()   the radio, which knows nothing about sensors
//! ```
//!
//! ## The rate is not the sensor's rate
//!
//! The BME68x can be read far faster than a phone needs advertised. This example reads once every two
//! seconds: faster only burns power and fills the advertising channels with nearly identical values.

#![no_std]
#![no_main]

use panic_halt as _;
use xiao_expansion_base_bsp::hal::prelude::*;
use xiao_expansion_base_bsp::{entry, ExpansionBase};

#[path = "common/mailbox.rs"]
mod mailbox;

/// Seeed's company identifier is not ours to use; this is the SIG's test range, and a product takes
/// its own from the assigned-numbers list.
const COMPANY_ID: u16 = 0x02E5;

/// Frame version, so a receiver can reject what it does not understand.
const FRAME_VERSION: u8 = 0x01;

const READ_PERIOD_MS: u32 = 2_000;

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
    let mut board = ExpansionBase::take().expect("peripherals taken twice");
    // Delay first: it needs the board mutably, and the LED and the facade borrow it afterwards.
    let mut delay = board.delay();
    let led = board.led();

    // The sensor first: if the carrier is not there, there is nothing to advertise.
    let mut environment = match board.environment() {
        Ok(sensor) => sensor,
        Err(_) => {
            mailbox::write(
                mailbox::Status::Fail,
                b"base ble: no BME68x (is the carrier fitted?)",
            );
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // Then the radio: the stack boots after the sensor is known good, so a failure is attributable.
    let mut ble = match efr32mg24_radio::Bluetooth::new(&mut delay) {
        Ok(ble) => ble,
        Err(_) => {
            mailbox::write(mailbox::Status::Fail, b"base ble: stack did not boot");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let mut sequence: u16 = 0;
    let mut advertised: u32 = 0;

    loop {
        // The BME68x needs a delay hook while it performs its measurement; the board's delay is the
        // one that is calibrated.
        let reading = match environment.read_with(|| delay.delay_ms(1)) {
            Ok(reading) => reading,
            Err(_) => {
                mailbox::write(mailbox::Status::Fail, b"base ble: BME68x read failed");
                delay.delay_ms(READ_PERIOD_MS);
                continue;
            }
        };

        // Fixed point, little endian, nine bytes: the receiver multiplies by 0.01.
        let mut frame = [0u8; 9];
        frame[0] = FRAME_VERSION;
        frame[1..3].copy_from_slice(&(reading.temperature_centi_c as i16).to_le_bytes());
        frame[3..5].copy_from_slice(&(reading.pressure_hpa as i16).to_le_bytes());
        // Humidity can be negative in the sensor's own units on a cold start; it is clamped here
        // rather than wrapped, because 655.35 % is not a humidity.
        let humidity = reading.humidity_centi_pct.clamp(0, 10_000) as u16;
        frame[5..7].copy_from_slice(&humidity.to_le_bytes());
        frame[7..9].copy_from_slice(&sequence.to_le_bytes());

        if ble.set_manufacturer_data(COMPANY_ID, &frame).is_ok()
            && ble.start_advertising(500, 0).is_ok()
        {
            advertised += 1;
            sequence = sequence.wrapping_add(1);
            let mut msg = [0u8; 64];
            let mut at = 0;
            msg[at..at + b"base ble: t=".len()].copy_from_slice(b"base ble: t=");
            at += 12;
            at = push_dec(&mut msg, at, reading.temperature_centi_c.unsigned_abs());
            msg[at..at + b"c p=".len()].copy_from_slice(b"c p=");
            at += 5;
            at = push_dec(&mut msg, at, reading.pressure_hpa.unsigned_abs());
            msg[at..at + b"hpa n=".len()].copy_from_slice(b"hpa n=");
            at += 6;
            at = push_dec(&mut msg, at, advertised);
            mailbox::write(mailbox::Status::Pass, &msg[..at]);
        } else {
            mailbox::write(
                mailbox::Status::Fail,
                b"base ble: payload refused (31-byte budget)",
            );
        }

        led.on();
        delay.delay_ms(20);
        led.off();
        delay.delay_ms(READ_PERIOD_MS - 20);
    }
}
