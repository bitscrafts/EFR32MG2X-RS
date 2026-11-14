//! The IMU, on the air: motion broadcast so a phone can watch it without connecting.
//!
//! This is the sensor node in its most useful form for debugging: every 100 ms the accelerometer is
//! read, and the three axes plus a motion flag go out in the advertising payload. A phone with nRF
//! Connect shows the numbers as a hex blob; a second board running `ble_scanner` shows the same frame
//! with an RSSI. No pairing, no GATT database, no crypto — which is why this example exists **now**,
//! while the connectable path waits for the PSA library.
//!
//! ## The frame, and why it is integers
//!
//! ```text
//! byte 0      frame version (0x01)
//! byte 1      flags: bit0 = moving, bit1..7 reserved
//! bytes 2-3   x, milli-g, little endian, i16
//! bytes 4-5   y
//! bytes 6-7   z
//! bytes 8-9   |a|, milli-g (1000 at rest on a desk)
//! ------------------------------------------------
//! 10 bytes of payload, 17 of the 31 with flags and the manufacturer header
//! ```
//!
//! Milli-g as an `i16` covers ±32 g, which is everything the sensor's ±8 g range can produce. Sending
//! text would cost three times as much and force the receiver to parse.
//!
//! ## Why advertising and not a notification
//!
//! A notification requires a connection, a connection requires a GATT database, and a GATT database in
//! this SDK's connectable path pulls the bonding code — which pulls a crypto library that is not
//! vendored. Advertising carries the same numbers with none of that, at the cost of being one-way and
//! at most 31 bytes. For a sensor you want to *watch*, that is the right trade.
//!
//! ## What to expect when it runs
//!
//! At rest, `|a|` sits near 1000 milli-g on a still desk, and x/y/z rotate as you tilt the board. Shake
//! it and the flag changes and the magnitude swings past 2000. A phone in nRF Connect shows the frame
//! updating roughly ten times a second; that rate is set below, not by the sensor, which can do 416 Hz.

#![no_std]
#![no_main]

use panic_halt as _;
use xiao_mg24_bsp::hal::prelude::*;
use xiao_mg24_bsp::hal::sensors::lsm6ds3trc::{AccelOdr, AccelRange, Config, ImuError, Lsm6ds3trc};
use xiao_mg24_bsp::{entry, Module};

#[path = "common/mailbox.rs"]
mod mailbox;

/// A company identifier from the SIG's test range; a product takes its own.
const COMPANY_ID: u16 = 0x02E5;

const FRAME_VERSION: u8 = 0x01;
const FLAG_MOVING: u8 = 0x01;

/// Above this departure from 1 g the board is considered to be moving.
///
/// 350 milli-g is well above the sensor's noise (a few mg at rest) and below a deliberate tap, which
/// this project measured at 1800 milli-g in the fall detector's calibration.
const MOTION_THRESHOLD_MG: i32 = 350;

/// How often a sample is advertised. Faster than a human can watch, slower than the sensor can
/// measure — and every advertisement costs radio time in all three advertising channels.
const PERIOD_MS: u32 = 100;

fn magnitude_mg(accel: [i32; 3]) -> i32 {
    // Integer square root: no floats, no libm, and a milli-g is all the resolution this frame carries.
    let sq = accel[0] * accel[0] + accel[1] * accel[1] + accel[2] * accel[2];
    let mut mag = 0i32;
    while (mag + 1) * (mag + 1) <= sq {
        mag += 1;
    }
    mag
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
    let mut module = Module::take().expect("peripherals taken twice");
    // Delay first: it borrows the board mutably; everything else borrows it afterwards.
    let mut delay = module.delay();
    let led = module.led();

    // The IMU's power gate, then the bus the module already knows.
    module.imu_power().on();
    delay.delay_ms(4);

    let mut imu = match Lsm6ds3trc::with_config(
        module.imu_bus(),
        Config {
            range: AccelRange::G8,
            odr: AccelOdr::Hz416,
        },
    ) {
        Ok(imu) => imu,
        Err(ImuError::NotAnLsm6ds3(id)) => {
            let mut buf = [0u8; 40];
            buf[..12].copy_from_slice(b"motion: id=0x");
            let at = push_dec(&mut buf, 11, u32::from(id));
            mailbox::write(mailbox::Status::Fail, &buf[..at]);
            loop {
                cortex_m::asm::wfi();
            }
        }
        Err(_) => {
            mailbox::write(mailbox::Status::Fail, b"motion: no ack (gate? bus?)");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // The stack boots after the sensor is known good, so a failure is attributable to one of them.
    let mut ble = match efr32mg24_radio::Bluetooth::new(&mut delay) {
        Ok(ble) => ble,
        Err(_) => {
            mailbox::write(mailbox::Status::Fail, b"motion: stack did not boot");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let mut last_moving = false;
    let mut frames: u32 = 0;

    loop {
        if let Ok(motion) = imu.read() {
            let accel = motion.accel_mg;
            let mag = magnitude_mg(accel);
            let moving = (mag - 1000).abs() > MOTION_THRESHOLD_MG;

            let mut frame = [0u8; 10];
            frame[0] = FRAME_VERSION;
            frame[1] = if moving { FLAG_MOVING } else { 0 };
            // Each axis is clamped to the frame's range rather than wrapped: a value that saturates is
            // a value the receiver can trust as "at least this much", and a wrapped one is a lie.
            let clamp = |v: i32| v.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
            frame[2..4].copy_from_slice(&clamp(accel[0]).to_le_bytes());
            frame[4..6].copy_from_slice(&clamp(accel[1]).to_le_bytes());
            frame[6..8].copy_from_slice(&clamp(accel[2]).to_le_bytes());
            frame[8..10].copy_from_slice(&clamp(mag).to_le_bytes());

            // The payload is replaced and advertising restarted: the stack does not re-read the
            // advertising data while a set is running.
            if ble.set_manufacturer_data(COMPANY_ID, &frame).is_ok()
                && ble.start_advertising(100, 0).is_ok()
            {
                frames += 1;
            }

            // The LED tracks motion, which makes the board readable across a room without a phone.
            if moving != last_moving {
                last_moving = moving;
                let mut msg = [0u8; 64];
                let mut at = 0;
                msg[at..at + b"motion: |a|=".len()].copy_from_slice(b"motion: |a|=");
                at += 12;
                at = push_dec(&mut msg, at, mag.unsigned_abs());
                msg[at..at + b" mg ".len()].copy_from_slice(b" mg ");
                at += 4;
                msg[at..at + 7].copy_from_slice(if moving { b"moving " } else { b"still  " });
                at += 7;
                msg[at..at + b"n=".len()].copy_from_slice(b"n=");
                at += 2;
                at = push_dec(&mut msg, at, frames);
                mailbox::write(
                    if moving {
                        mailbox::Status::Info
                    } else {
                        mailbox::Status::Pass
                    },
                    &msg[..at],
                );
            }
        }

        if last_moving {
            led.on();
        } else {
            led.off();
        }
        delay.delay_ms(PERIOD_MS);
    }
}
