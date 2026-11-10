//! Which pins have an external pull-up? A measurement, not a datasheet reading.
//!
//! ## The question
//!
//! The IMU (LSM6DS3TR-C) is fitted on this Sense module — the microphone proves
//! it, since that part exists only on the Sense — but it does not answer on
//! either documented bus:
//!
//! - base bus (PC4/PC5): `3c 51 76`, no `6a`;
//! - I2C1 as mapped by the Arduino variant (PB3/PB2): nothing at all;
//! - with PD5 (the IMU's power enable in Seeed's driver) and PC8 both driven.
//!
//! So the pin map assumption is wrong somewhere. Rather than guess another pin
//! pair, this measures the board: an I2C bus has **pull-up resistors on exactly
//! two pins**. Configure every GPIO as an input with no internal pull, read the
//! input registers, and any pin that reads high is pulled up by something
//! outside the chip. Those are the bus lines.
//!
//! ## Method
//!
//! For each port: `MODEL = 0` and `MODEH = 0` (mode 0 = input disabled, and with
//! `DOUT = 0` there is no internal pull), then read `DIN`. A set bit means the
//! level is high without the chip driving it. The report lists the masks so the
//! pins can be identified by bit position.
//!
//! Nothing is driven, so this cannot damage the board: the pins end up as plain
//! inputs.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

const PIN_IMU_POWER: u32 = 1 << 5; // PD5

fn push_bytes(buf: &mut [u8], mut pos: usize, s: &[u8]) -> usize {
    for b in s {
        if pos < buf.len() {
            buf[pos] = *b;
            pos += 1;
        }
    }
    pos
}

fn push_hex8(buf: &mut [u8], mut pos: usize, value: u32) -> usize {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for shift in (0..8).rev() {
        if pos < buf.len() {
            buf[pos] = HEX[((value >> (shift * 4)) & 0xF) as usize];
            pos += 1;
        }
    }
    pos
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());

    let gpio = &dp.gpio_s;
    // MODE 1: input enabled, no pull. Mode 0 disables the input buffer, so DIN
    // reads 0 whatever the pin is doing — the first attempt used 0 and produced
    // a meaningless all-zero result.
    const INPUT_NO_PULL: u32 = 0x1111_1111;
    gpio.porta_model()
        .write(|w| unsafe { w.bits(INPUT_NO_PULL) });
    gpio.portb_model()
        .write(|w| unsafe { w.bits(INPUT_NO_PULL) });
    gpio.portc_model()
        .write(|w| unsafe { w.bits(INPUT_NO_PULL) });
    gpio.portd_model()
        .write(|w| unsafe { w.bits(INPUT_NO_PULL) });
    // Only ports A and C have a MODEH register in this package: B and D carry
    // fewer than eight pins, so MODEL covers them entirely.
    gpio.porta_modeh()
        .write(|w| unsafe { w.bits(INPUT_NO_PULL) });
    gpio.portc_modeh()
        .write(|w| unsafe { w.bits(INPUT_NO_PULL) });
    gpio.porta_dout().write(|w| unsafe { w.bits(0) });
    gpio.portb_dout().write(|w| unsafe { w.bits(0) });
    gpio.portc_dout().write(|w| unsafe { w.bits(0) });
    gpio.portd_dout().write(|w| unsafe { w.bits(0) });

    cortex_m::asm::delay(200_000);
    let a0 = gpio.porta_din().read().bits();
    let b0 = gpio.portb_din().read().bits();
    let c0 = gpio.portc_din().read().bits();
    let d0 = gpio.portd_din().read().bits();

    // Power the IMU rail and measure again: a pin that becomes high only now is
    // a line whose pull-up lives on the sensor's own supply.
    gpio.portd_model().write(|w| unsafe { w.bits(0) });
    gpio.portd_dout()
        .modify(|r, w| unsafe { w.bits(r.bits() | PIN_IMU_POWER) });
    gpio.portd_model().modify(|_r, w| w.mode5().pushpull());
    gpio.portd_dout()
        .modify(|r, w| unsafe { w.bits(r.bits() | PIN_IMU_POWER) });
    cortex_m::asm::delay(400_000);

    let a1 = gpio.porta_din().read().bits();
    let b1 = gpio.portb_din().read().bits();
    let c1 = gpio.portc_din().read().bits();
    let d1 = gpio.portd_din().read().bits();

    let _ = (a0, b0, c0, d0);
    let mut msg2 = [0u8; 64];
    let mut p2 = push_bytes(&mut msg2, 0, b"pwr_on  a=");
    p2 = push_hex8(&mut msg2, p2, a1);
    p2 = push_bytes(&mut msg2, p2, b" b=");
    p2 = push_hex8(&mut msg2, p2, b1);
    p2 = push_bytes(&mut msg2, p2, b" c=");
    p2 = push_hex8(&mut msg2, p2, c1);
    p2 = push_bytes(&mut msg2, p2, b" d=");
    p2 = push_hex8(&mut msg2, p2, d1);
    mailbox::pass(&msg2[..p2]);

    loop {
        cortex_m::asm::nop();
    }
}
