//! HAL bus scan: what actually answers on the Expansion Base I2C bus.
//!
//! Two jobs, on purpose:
//!
//! 1. it is the HAL twin of `pac_i2c_scan`, so a disagreement between the two is
//!    a transport bug;
//! 2. it powers the on-board sensors first (PD5), which is the only way to see
//!    whether the LSM6DS3TR-C is on this bus at all — an unpowered IMU is
//!    indistinguishable from an absent one.
//!
//! On the XIAO MG24 **Sense** the IMU should appear at 0x6A; on the standard
//! module it is not fitted. The scan therefore answers a hardware question with
//! a measurement instead of a datasheet assumption.

#![no_std]
#![no_main]

use core::cell::RefCell;

use efr32mg24_hal::board::Board;
use efr32mg24_hal::bus::I2cBus;
use efr32mg24_hal::entry;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

const HEX: &[u8; 16] = b"0123456789abcdef";

fn push_hex4(buf: &mut [u8], pos: usize, value: u8) -> usize {
    let mut p = pos;
    if p < buf.len() {
        buf[p] = HEX[(value >> 4) as usize];
        p += 1;
    }
    if p < buf.len() {
        buf[p] = HEX[(value & 0xF) as usize];
        p += 1;
    }
    p
}

#[entry]
fn main() -> ! {
    let board = match Board::take() {
        Some(b) => b,
        None => loop {
            cortex_m::asm::nop();
        },
    };
    let led = board.led();

    // Power the on-board sensors, then let the rail settle.
    board.sensors_power().enable();
    cortex_m::asm::delay(4_000_000);

    let bus = RefCell::new(board.i2c());
    let mut found: [u8; 16] = [0; 16];
    let mut count = 0usize;
    for addr in 0x08u8..=0x77 {
        if bus.borrow_mut().probe(addr) {
            if count < found.len() {
                found[count] = addr;
                count += 1;
            }
        }
    }

    // Report the addresses in hex, not as a bitmap: a bitmap needs a decoder to
    // read, and the whole point is that a human can act on this in one glance.
    let mut msg = [0u8; 64];
    let mut pos = 0usize;
    let prefix = b"scan n=";
    msg[..prefix.len()].copy_from_slice(prefix);
    pos += prefix.len();
    if pos < msg.len() {
        msg[pos] = HEX[count as usize];
        pos += 1;
    }
    for a in found[..count].iter() {
        if pos + 1 < msg.len() {
            msg[pos] = b' ';
            pos += 1;
            pos = push_hex4(&mut msg, pos, *a);
        }
    }

    // Expect at least the three base devices; the IMU at 0x6A is only present on
    // the Sense module, so it is not part of the pass condition.
    let has_panel = found[..count].contains(&0x3C);
    let has_sensor = found[..count].contains(&0x76) || found[..count].contains(&0x77);
    if has_panel && has_sensor {
        mailbox::pass(&msg[..pos]);
    } else {
        mailbox::fail(&msg[..pos]);
    }

    loop {
        led.toggle();
        cortex_m::asm::delay(2_000_000);
    }
}
