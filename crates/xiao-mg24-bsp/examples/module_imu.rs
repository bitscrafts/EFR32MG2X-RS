//! The Sense variant's IMU, read through the module crate alone.
//!
//! This example exists to answer one question with the compiler: **does the module crate
//! stand on its own?** It depends on `xiao-mg24-bsp` with the `sense` feature and nothing
//! else — no Expansion Base, no OLED, no buzzer. Put the XIAO on the desk and this runs.
//!
//! What it proves on the hardware:
//!
//! - the IMU's bus is `SDA = PB2`, `SCL = PB3` — the opposite of the vendor's Arduino
//!   variant, and the reason a scan of the "documented" pins returns an empty bus;
//! - `PD5` must be driven high before the sensor answers at all, because an unpowered IMU is
//!   indistinguishable from an absent one;
//! - the chip identifier reads `0x6A`, and the accelerometer magnitude at rest is about
//!   1000 mg — gravity is a free check on the bus, the scale factors and the byte order.
//!
//! Report goes to the SWD mailbox (see `common/mailbox.rs`), so the result can be read
//! without eyes on the board.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_hal::delay::Delay;
use efr32mg24_hal::prelude::*;
use efr32mg24_hal::sensors::lsm6ds3trc::{AccelOdr, AccelRange, Config, Lsm6ds3trc};
use efr32mg24_hal::sensors::ImuError;
use panic_halt as _;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Report one verdict and stop touching the bus; the mailbox keeps the last message.
fn report(status: mailbox::Status, parts: &[&[u8]]) {
    let mut msg = [0u8; 200];
    let mut pos = 0;
    for part in parts {
        let end = pos + part.len();
        if end > msg.len() {
            break;
        }
        msg[pos..end].copy_from_slice(part);
        pos = end;
    }
    mailbox::write(status, &msg[..pos]);
}

fn push_dec(out: &mut [u8], mut at: usize, mut value: i32) -> usize {
    if value < 0 {
        out[at] = b'-';
        at += 1;
        value = -value;
    }
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
    let mut delay: Delay = module.delay();

    // Power first, talk after: the gate is not optional.
    let power = module.imu_power();
    power.on();
    delay.delay_ms(4);

    let bus = module.imu_bus();
    let mut imu = match Lsm6ds3trc::with_config(
        bus,
        Config {
            range: AccelRange::G8,
            odr: AccelOdr::Hz416,
        },
    ) {
        Ok(imu) => imu,
        Err(ImuError::NotAnLsm6ds3(id)) => {
            let mut buf = [0u8; 40];
            buf[..10].copy_from_slice(b"imu id=0x");
            let at = push_dec(&mut buf, 9, i32::from(id));
            buf[at] = b' ';
            buf[at + 1] = b'?';
            report(mailbox::Status::Fail, &[&buf[..at + 2]]);
            loop {
                delay.delay_ms(1000);
            }
        }
        Err(_) => {
            report(mailbox::Status::Fail, &[b"imu no ack (power? bus?)"]);
            loop {
                delay.delay_ms(1000);
            }
        }
    };

    // Settle, then average a few reads: a single sample of a MEMS accelerometer is noisy.
    delay.delay_ms(50);
    let mut sum = [0i32; 3];
    let mut n = 0i32;
    for _ in 0..8 {
        if let Ok(motion) = imu.read() {
            for axis in 0..3 {
                sum[axis] += motion.accel_mg[axis];
            }
            n += 1;
        }
        delay.delay_ms(5);
    }
    if n == 0 {
        report(mailbox::Status::Fail, &[b"imu read failed"]);
        loop {
            delay.delay_ms(1000);
        }
    }
    let avg = [sum[0] / n, sum[1] / n, sum[2] / n];

    // |a| without floats: the integer square root of the squared magnitude.
    let sq = avg[0] * avg[0] + avg[1] * avg[1] + avg[2] * avg[2];
    let mut mag = 0i32;
    while (mag + 1) * (mag + 1) <= sq {
        mag += 1;
    }

    let mut buf = [0u8; 96];
    let mut at = 0;
    buf[..8].copy_from_slice(b"imu 6a ");
    at = 8;
    buf[at] = b'x';
    at = push_dec(&mut buf, at + 1, avg[0]);
    buf[at] = b' ';
    at = push_dec(&mut buf, at + 1, avg[1]);
    buf[at] = b' ';
    at = push_dec(&mut buf, at + 1, avg[2]);
    buf[at] = b' ';
    at += 1;
    buf[at..at + 4].copy_from_slice(b"|a|=");
    at += 4;
    at = push_dec(&mut buf, at, mag);

    // Gravity is the invariant: a bus that returns plausible noise still fails this.
    let ok = (700..=1300).contains(&mag);
    report(
        if ok {
            mailbox::Status::Pass
        } else {
            mailbox::Status::Fail
        },
        &[&buf[..at]],
    );

    loop {
        delay.delay_ms(1000);
    }
}
