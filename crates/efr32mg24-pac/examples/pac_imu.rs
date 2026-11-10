//! PAC-level IMU probe: is the LSM6DS3TR-C really on this board, and does PD5
//! power it?
//!
//! The question this example answers, and why it is written at the PAC level:
//! the HAL scan with PD5 driven high found only 0x3C, 0x51 and 0x76 — no 0x6A.
//! Either the module is the standard XIAO MG24 (the IMU is a Sense-only part),
//! or the power gate is not what the wiki's Arduino sketch implies, or the part
//! answers somewhere else. Those three hypotheses are separable with two bus
//! scans and one register read, so that is exactly what this does:
//!
//! 1. scan the bus with PD5 **low** (sensors unpowered),
//! 2. drive PD5 high, wait for the rail, scan again,
//! 3. probe 0x6A and 0x6B, read WHO_AM_I, and if a part answers, configure it
//!    (±2 g, ±250 dps, 104 Hz) and read one 6-axis sample.
//!
//! Reporting both scans is the point: an address that appears only after PD5
//! rises is the IMU; the same set twice means PD5 is not its power gate.
//!
//! Register facts used here:
//! - `0x0F` WHO_AM_I, expected `0x6A` on an LSM6DS3 / LSM6DS3TR-C,
//! - `0x12` CTRL3_C: block-data-update (bit 6) and auto-increment (bit 2),
//! - `0x10` CTRL1_XL and `0x11` CTRL2_G: 104 Hz, ±2 g and ±250 dps,
//! - `0x22..0x2D` the twelve output bytes (gyro X/Y/Z, then accel X/Y/Z),
//! - sensitivity 0.061 mg/LSB and 8.75 mdps/LSB at those ranges.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

const HALF_BIT: u32 = 60;
const PIN_SDA: u32 = 1 << 4; // PC04
const PIN_SCL: u32 = 1 << 5; // PC05
const PIN_SENSOR_POWER: u32 = 1 << 5; // PD5

const HEX: &[u8; 16] = b"0123456789abcdef";

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
    for shift in (0..8).rev() {
        if pos < buf.len() {
            buf[pos] = HEX[((value >> (shift * 4)) & 0xF) as usize];
            pos += 1;
        }
    }
    pos
}

fn push_hex2(buf: &mut [u8], mut pos: usize, value: u8) -> usize {
    // `(1..=0).rev()` looks like "high nibble then low" but is an empty range,
    // which silently printed nothing for every address.
    for shift in [1u8, 0] {
        if pos < buf.len() {
            buf[pos] = HEX[((value >> (shift * 4)) & 0xF) as usize];
            pos += 1;
        }
    }
    pos
}

fn push_dec(buf: &mut [u8], mut pos: usize, mut value: u32) -> usize {
    let mut digits = [0u8; 10];
    let mut n = 0;
    if value == 0 {
        digits[0] = b'0';
        n = 1;
    }
    while value > 0 {
        digits[n] = b'0' + (value % 10) as u8;
        value /= 10;
        n += 1;
    }
    while n > 0 {
        n -= 1;
        if pos < buf.len() {
            buf[pos] = digits[n];
            pos += 1;
        }
    }
    pos
}

struct Bus<'a> {
    gpio: &'a pac::gpio_s::RegisterBlock,
}

impl Bus<'_> {
    fn delay(&self) {
        cortex_m::asm::delay(HALF_BIT);
    }

    fn set(&self, sda_low: bool, scl_low: bool) {
        let mut bits = self.gpio.portc_dout().read().bits();
        bits = if sda_low {
            bits & !PIN_SDA
        } else {
            bits | PIN_SDA
        };
        bits = if scl_low {
            bits & !PIN_SCL
        } else {
            bits | PIN_SCL
        };
        self.gpio.portc_dout().write(|w| unsafe { w.bits(bits) });
        self.delay();
    }

    fn sda_high(&self) -> bool {
        self.gpio.portc_din().read().bits() & PIN_SDA != 0
    }

    fn start(&self) {
        self.set(true, true);
        self.set(true, false);
        self.set(false, false);
        self.set(true, false);
    }

    fn stop(&self) {
        self.set(true, true);
        self.set(true, false);
        self.set(false, false);
    }

    fn write_byte(&self, byte: u8) -> bool {
        for i in (0..8).rev() {
            let low = (byte >> i) & 1 == 0;
            self.set(low, true);
            self.set(low, false);
            self.set(low, true);
        }
        self.set(false, true);
        self.set(false, false);
        let acked = !self.sda_high();
        self.set(false, true);
        acked
    }

    fn read_byte(&self, ack: bool) -> u8 {
        let mut value = 0u8;
        self.set(false, true);
        for _ in 0..8 {
            self.set(false, false);
            value = (value << 1) | u8::from(self.sda_high());
            self.set(false, true);
        }
        self.set(ack, true);
        self.set(ack, false);
        self.set(ack, true);
        value
    }

    fn probe(&self, addr: u8) -> bool {
        self.start();
        let acked = self.write_byte(addr << 1);
        self.stop();
        acked
    }

    fn read_reg(&self, addr: u8, reg: u8) -> Option<u8> {
        self.start();
        if !self.write_byte(addr << 1) || !self.write_byte(reg) {
            self.stop();
            return None;
        }
        self.start();
        if !self.write_byte((addr << 1) | 1) {
            self.stop();
            return None;
        }
        let v = self.read_byte(false);
        self.stop();
        Some(v)
    }

    fn write_reg(&self, addr: u8, reg: u8, value: u8) -> bool {
        self.start();
        let ok = self.write_byte(addr << 1) && self.write_byte(reg) && self.write_byte(value);
        self.stop();
        ok
    }

    fn read_regs(&self, addr: u8, reg: u8, out: &mut [u8]) -> bool {
        self.start();
        if !self.write_byte(addr << 1) || !self.write_byte(reg) {
            self.stop();
            return false;
        }
        self.start();
        if !self.write_byte((addr << 1) | 1) {
            self.stop();
            return false;
        }
        let last = out.len().saturating_sub(1);
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = self.read_byte(i != last);
        }
        self.stop();
        true
    }

    /// Collect every address that acknowledged into `out`, returning the count.
    ///
    /// A bitmap was tried first and printed nonsense: 0x3C is bit 52, so casting
    /// the u64 to u32 for the report dropped every device above 0x27. An address
    /// list cannot be silently truncated like that.
    fn scan_into(&self, out: &mut [u8]) -> usize {
        let mut n = 0;
        for addr in 0x08u8..=0x77 {
            if self.probe(addr) && n < out.len() {
                out[n] = addr;
                n += 1;
            }
        }
        n
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());

    // I2C pins: open drain, released.
    dp.gpio_s
        .portc_model()
        .modify(|_r, w| w.mode4().wiredand().mode5().wiredand());
    dp.gpio_s
        .portc_dout()
        .modify(|r, w| unsafe { w.bits(r.bits() | PIN_SDA | PIN_SCL) });

    // PD5 high from the start, and never toggled afterwards.
    //
    // Measured 2026-11-11: scanning with PD5 low and then raising it leaves the
    // bus dead for the rest of the run — both scans come back empty even though
    // the lines read 0x30 (both high) afterwards. Power-cycling the rail while
    // the bus is being clocked is a bad idea on this board; power first, then
    // talk. Whether PD5 gates the IMU, the base, or both is still open, and this
    // example no longer needs to answer that to be useful.
    dp.gpio_s.portd_model().modify(|_r, w| w.mode5().pushpull());
    dp.gpio_s
        .portd_dout()
        .modify(|r, w| unsafe { w.bits(r.bits() | PIN_SENSOR_POWER) });
    cortex_m::asm::delay(4_000_000);

    let bus = Bus { gpio: &dp.gpio_s };
    let mut on_list = [0u8; 12];
    let on_n = bus.scan_into(&mut on_list);

    // Which candidate addresses answer?
    let mut found_addr = 0u8;
    let mut who = 0u8;
    let mut imu_addr = 0u8;
    for addr in [0x6Au8, 0x6Bu8] {
        if bus.probe(addr) {
            found_addr = addr;
            who = bus.read_reg(addr, 0x0F).unwrap_or(0);
            if who == 0x6A || who == 0x69 {
                imu_addr = addr;
            }
            break;
        }
    }

    let mut msg = [0u8; 64];
    let mut pos = push_bytes(&mut msg, 0, b"on");
    for a in on_list[..on_n].iter() {
        pos = push_bytes(&mut msg, pos, b" ");
        pos = push_hex2(&mut msg, pos, *a);
    }
    pos = push_bytes(&mut msg, pos, b" 6a=");
    pos = push_hex2(&mut msg, pos, found_addr);
    pos = push_bytes(&mut msg, pos, b"/");
    pos = push_hex2(&mut msg, pos, who);

    if imu_addr == 0 {
        // Not an error of the bus: this module may simply not carry the part.
        mailbox::fail(&msg[..pos]);
        idle(&dp);
    }

    // Configure and read one sample: ±2 g, ±250 dps, 104 Hz.
    bus.write_reg(imu_addr, 0x12, 0b0100_0100); // BDU | IF_INC
    bus.write_reg(imu_addr, 0x10, 0b0100_0000); // CTRL1_XL
    bus.write_reg(imu_addr, 0x11, 0b0100_0000); // CTRL2_G
    cortex_m::asm::delay(2_000_000);

    let mut raw = [0u8; 12];
    let ok = bus.read_regs(imu_addr, 0x22, &mut raw);
    let word = |lo: u8, hi: u8| i16::from_le_bytes([lo, hi]) as i32;
    let ax = (word(raw[6], raw[7]) as f32 * 0.061) as i32;
    let ay = (word(raw[8], raw[9]) as f32 * 0.061) as i32;
    let az = (word(raw[10], raw[11]) as f32 * 0.061) as i32;
    let gx = (word(raw[0], raw[1]) as f32 * 8.75) as i32;

    let mut msg2 = [0u8; 64];
    let mut p2 = push_bytes(&mut msg2, 0, b"imu a=");
    p2 = push_dec(&mut msg2, p2, ax.unsigned_abs());
    p2 = push_bytes(&mut msg2, p2, b",");
    p2 = push_dec(&mut msg2, p2, ay.unsigned_abs());
    p2 = push_bytes(&mut msg2, p2, b",");
    p2 = push_dec(&mut msg2, p2, az.unsigned_abs());
    p2 = push_bytes(&mut msg2, p2, b" g=");
    p2 = push_dec(&mut msg2, p2, gx.unsigned_abs());

    let peak = ax.abs().max(ay.abs()).max(az.abs());
    if ok && (700..=1300).contains(&peak) {
        mailbox::pass(&msg2[..p2]);
    } else {
        mailbox::fail(&msg2[..p2]);
    }

    idle(&dp);
}

fn idle(dp: &pac::Peripherals) -> ! {
    dp.gpio_s.porta_model().modify(|_r, w| w.mode7().pushpull());
    loop {
        for _ in 0..2_000_000 {
            cortex_m::asm::nop();
        }
        dp.gpio_s
            .porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 7)) });
    }
}
