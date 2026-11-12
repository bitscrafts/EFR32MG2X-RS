//! Read the IMU's own control registers back, through the HAL bus.
//!
//! ## Why
//!
//! Through the HAL the gyroscope reports plausible noise but the accelerometer
//! sits at 9/0/1 mg, while the PAC example on the same board reads one g. Before
//! touching the driver again, this asks the sensor what it thinks its
//! configuration is:
//!
//! - `CTRL1_XL` (0x10) must read 0x40 after the driver's write (ODR 104 Hz, ±2 g);
//! - `CTRL2_G` (0x11) must read 0x40 (ODR 104 Hz, ±250 dps);
//! - `CTRL3_C` (0x12) must read 0x44 (BDU + IF_INC).
//!
//! If `CTRL1_XL` reads 0x00 the accelerometer is still powered down and no
//! amount of reading will produce data — the fault is in the write path, not in
//! the arithmetic. If it reads 0x40, the write landed and the fault is in how the
//! twelve bytes are unpacked.
//!
//! The raw 12 bytes are printed as hex as well, so the two can be compared with
//! what the PAC example decodes.

#![no_std]
#![no_main]

use core::cell::RefCell;

use efr32mg24_hal::board::Board;
use efr32mg24_hal::bus::Bus;
use efr32mg24_hal::bus::{I2cBus, SharedBus};
use efr32mg24_hal::display::append;
use efr32mg24_hal::entry;
use efr32mg24_hal::pin::Port;
use efr32mg24_hal::sensors::Lsm6ds3trc;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

const HEX: &[u8; 16] = b"0123456789abcdef";

fn hex2(buf: &mut [u8], pos: usize, v: u8) -> usize {
    let mut pos = pos;
    if pos + 1 < buf.len() {
        buf[pos] = HEX[(v >> 4) as usize];
        buf[pos + 1] = HEX[(v & 0x0F) as usize];
        pos += 2;
    }
    pos
}

#[entry]
fn main() -> ! {
    let board = match Board::take() {
        Some(b) => b,
        None => loop {
            cortex_m::asm::nop();
        },
    };
    let led = board.output_on(Port::A, 7, true);
    board.output_on(Port::D, 5, false).on();
    cortex_m::asm::delay(4_000_000);

    let bus = RefCell::new(board.i2c_on(Bus::new(Port::B, 1 << 2, 1 << 3)));
    let mut imu = match Lsm6ds3trc::new(SharedBus::new(&bus)) {
        Ok(i) => i,
        Err(_) => {
            mailbox::fail(b"driver init failed");
            loop {
                led.toggle();
                cortex_m::asm::delay(2_000_000);
            }
        }
    };
    cortex_m::asm::delay(1_000_000);

    // What the sensor says about its own configuration, after the driver wrote it.
    let ctrl1 = bus.borrow_mut().read_reg(0x6A, 0x10).unwrap_or(0xFF);
    let ctrl2 = bus.borrow_mut().read_reg(0x6A, 0x11).unwrap_or(0xFF);
    let ctrl3 = bus.borrow_mut().read_reg(0x6A, 0x12).unwrap_or(0xFF);

    // One record only: the mailbox keeps the last message, so a multi-step report
    // would show nothing but its final line.
    let mut raw = [0u8; 12];
    let ok = bus.borrow_mut().read_regs(0x6A, 0x22, &mut raw).is_ok();

    let mut msg = [0u8; 64];
    let mut pos = append(&mut msg, 0, b"c1=");
    pos = hex2(&mut msg, pos, ctrl1);
    pos = append(&mut msg, pos, b" c2=");
    pos = hex2(&mut msg, pos, ctrl2);
    pos = append(&mut msg, pos, b" c3=");
    pos = hex2(&mut msg, pos, ctrl3);
    pos = append(&mut msg, pos, b" ac=");
    for b in &raw[6..12] {
        pos = hex2(&mut msg, pos, *b);
        pos = append(&mut msg, pos, b" ");
    }
    if !ok {
        pos = append(&mut msg, pos, b"NOREAD");
    }

    let peak = match imu.read() {
        Ok(m) => m.accel_mg[0]
            .abs()
            .max(m.accel_mg[1].abs())
            .max(m.accel_mg[2].abs()),
        Err(_) => 0,
    };
    let configured = ctrl1 == 0x40 && ctrl2 == 0x40 && ctrl3 == 0x44;
    if configured && peak > 700 {
        mailbox::pass(&msg[..pos]);
    } else {
        mailbox::fail(&msg[..pos]);
    }

    loop {
        led.toggle();
        cortex_m::asm::delay(2_000_000);
    }
}
