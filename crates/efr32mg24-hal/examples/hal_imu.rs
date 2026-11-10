//! IMU test: the on-board LSM6DS3TR-C 6-axis sensor, through the HAL only.
//!
//! What this example is really demonstrating is the split between the two
//! layers:
//!
//! - the **power gate** (PD5) is board wiring, so it comes from
//!   `board.sensors_power()`;
//! - the **protocol** (WHO_AM_I, control registers, 12-byte burst) is the HAL
//!   driver's job.
//!
//! Two pieces of board wiring have to be right before the driver can work, and
//! both were measured rather than assumed:
//!
//! - **the bus**: the IMU is on its own pair, PB2 = SDA / PB3 = SCL, not on the
//!   Expansion Base bus where the panel, RTC and BME68x live;
//! - **the power gate**: PD5 high, then a few milliseconds before it answers.
//!
//! Hence two buses in this example: `board.imu_i2c()` for the sensor and
//! `board.i2c()` for the panel.
//!
//! Note on units: `display::format_fixed` takes a value multiplied by 100, so an
//! integer milligram count is passed already scaled. Passing it raw prints the
//! reading divided by 100 — which looks exactly like a dead accelerometer
//! (975 mg became "9"), and cost a round of debugging that the raw-byte dump in
//! `hal_imu_regs` settled.
//!
//! Verification: the report carries WHO_AM_I, the acceleration and the angular
//! rate. A board sitting still must read about 1000 mg on the axis pointing at
//! the floor and near zero on the other two, with the gyro near zero — that is a
//! physical check that needs no instrument.

#![no_std]
#![no_main]

use core::cell::RefCell;

use efr32mg24_hal::board::Board;
use efr32mg24_hal::bus::{I2cBus, SharedBus};
use efr32mg24_hal::display::{append, format_fixed, Ssd1306};
use efr32mg24_hal::entry;
use efr32mg24_hal::sensors::Lsm6ds3trc;
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
    let led = board.led();

    // Wiring first: without this the IMU is invisible on the bus, and it needs
    // time to come up after the rail settles (Seeed's sketch does not show a
    // delay, but the datasheet's power-up time is ~10 ms and the bus tolerates
    // waiting far better than a retry loop here).
    board.sensors_power().enable();
    cortex_m::asm::delay(4_000_000);

    let bus = RefCell::new(board.imu_i2c());

    // Report what the bus actually says, instead of collapsing every failure
    // into one number: "no ACK" and "wrong id" are completely different bugs.
    if !bus.borrow_mut().probe(0x6A) {
        mailbox::fail(b"imu no ack at 6a (pd5 high, bus ok?)");
        loop {
            led.toggle();
            cortex_m::asm::delay(2_000_000);
        }
    }
    let who = bus.borrow_mut().read_reg(0x6A, 0x0F).unwrap_or(0);
    if who != 0x6A {
        let mut msg = [0u8; 48];
        let mut pos = append(&mut msg, 0, b"imu who_am_i=");
        let hex = b"0123456789abcdef";
        msg[pos] = hex[(who >> 4) as usize];
        pos += 1;
        msg[pos] = hex[(who & 0x0F) as usize];
        pos += 1;
        pos = append(&mut msg, pos, b" (expected 6a)");
        mailbox::fail(&msg[..pos]);
        loop {
            led.toggle();
            cortex_m::asm::delay(2_000_000);
        }
    }

    let mut imu = match Lsm6ds3trc::new(SharedBus::new(&bus)) {
        Ok(i) => i,
        Err(_) => {
            mailbox::fail(b"imu driver init failed after id check");
            loop {
                led.toggle();
                cortex_m::asm::delay(2_000_000);
            }
        }
    };

    // Give the sensor a few samples to settle after power-up.
    for _ in 0..5 {
        let _ = imu.read();
        cortex_m::asm::delay(500_000);
    }

    // A second bus for the panel: different pins, same transport code. Sharing
    // one master between the two would mean reconfiguring pins behind the
    // driver's back.
    let base_bus = RefCell::new(board.i2c());
    let mut oled = Ssd1306::new(SharedBus::new(&base_bus));

    loop {
        match imu.read() {
            Ok(m) => {
                let mut msg = [0u8; 64];
                let mut pos = append(&mut msg, 0, b"imu id=6a a=");
                pos += format_fixed(&mut msg[pos..], m.accel_mg[0] * 100, 0);
                pos = append(&mut msg, pos, b",");
                pos += format_fixed(&mut msg[pos..], m.accel_mg[1] * 100, 0);
                pos = append(&mut msg, pos, b",");
                pos += format_fixed(&mut msg[pos..], m.accel_mg[2] * 100, 0);
                pos = append(&mut msg, pos, b" g=");
                pos += format_fixed(&mut msg[pos..], m.gyro_mdps[0] * 100, 0);
                pos = append(&mut msg, pos, b",");
                pos += format_fixed(&mut msg[pos..], m.gyro_mdps[1] * 100, 0);
                pos = append(&mut msg, pos, b",");
                pos += format_fixed(&mut msg[pos..], m.gyro_mdps[2] * 100, 0);

                // A board at rest reads ~1000 mg on one axis; anything in that
                // ballpark on at least one axis is evidence the accelerometer
                // is really measuring, not just answering on the bus.
                let peak = m.accel_mg[0]
                    .abs()
                    .max(m.accel_mg[1].abs())
                    .max(m.accel_mg[2].abs());
                if (700..=1300).contains(&peak) {
                    mailbox::pass(&msg[..pos]);
                } else {
                    mailbox::fail(&msg[..pos]);
                }

                if let Some(ref mut d) = oled {
                    d.clear();
                    d.text(0, 0, b"IMU ACC MG");
                    let mut line = [0u8; 20];
                    let n = format_fixed(&mut line, m.accel_mg[2] * 100, 0);
                    d.text(0, 2, &line[..n]);
                    d.flush();
                }
            }
            Err(_) => mailbox::fail(b"imu read failed"),
        }

        for _ in 0..2 {
            led.toggle();
            cortex_m::asm::delay(1_500_000);
        }
    }
}
