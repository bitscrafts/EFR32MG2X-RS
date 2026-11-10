//! Find the IMU's power enable by search, then read the sensor.
//!
//! ## What is already measured
//!
//! - The IMU bus pins are **PB3 (SDA) / PB2 (SCL)**: with every GPIO set to
//!   "input, no pull", `PORTB_DIN` reads `0x0C` — those two pins are pulled up
//!   from outside the chip, which is what an I2C bus looks like.
//! - The sensor does not answer there even with PD5 high (Seeed's driver names
//!   PD5 as `PIN_LSM6DS3TR_C_POWER`) or with PC8 high (the microphone's rail).
//! - The module is a **Sense**: the microphone works, and the microphone is
//!   Sense-only, so the IMU is fitted.
//!
//! ## What this does
//!
//! Drives each plausible GPIO high in turn and probes 0x6A on PB3/PB2 after
//! every one. Whichever pin makes the sensor answer is its power enable. Nothing
//! is written to the sensor until it answers, so this is read-only from the
//! device's point of view.
//!
//! A pin that is already an output (the LED, the mic enable) is left alone, and
//! the two bus pins are never driven as outputs.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

const HALF_BIT: u32 = 60;
const PIN_SCL: u32 = 1 << 2; // PB2
const PIN_SDA: u32 = 1 << 3; // PB3
const IMU_ADDR: u8 = 0x6A;

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

fn push_hex2(buf: &mut [u8], mut pos: usize, value: u8) -> usize {
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

fn push_signed(buf: &mut [u8], mut pos: usize, value: i32) -> usize {
    if value < 0 {
        pos = push_bytes(buf, pos, b"-");
    }
    push_dec(buf, pos, value.unsigned_abs())
}

struct Bus<'a> {
    gpio: &'a pac::gpio_s::RegisterBlock,
}

impl Bus<'_> {
    fn delay(&self) {
        cortex_m::asm::delay(HALF_BIT);
    }

    fn set(&self, sda_low: bool, scl_low: bool) {
        let mut bits = self.gpio.portb_dout().read().bits();
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
        self.gpio.portb_dout().write(|w| unsafe { w.bits(bits) });
        self.delay();
    }

    fn sda_high(&self) -> bool {
        self.gpio.portb_din().read().bits() & PIN_SDA != 0
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
}

/// Drive one pin high, leaving everything else alone.
///
/// The `modify` closure hands back a writer whose `bits()` needs the new value;
/// the current value comes from the reader, not from the writer.
fn drive_high(gpio: &pac::gpio_s::RegisterBlock, port: u8, pin: u32) {
    let mask = 1u32 << pin;
    let shift = pin * 4;
    let set_mode = |cur: u32| (cur & !(0xF << shift)) | (4 << shift); // mode 4 = push-pull
    match port {
        0 => {
            gpio.porta_model()
                .modify(|r, w| unsafe { w.bits(set_mode(r.bits())) });
            gpio.porta_dout()
                .modify(|r, w| unsafe { w.bits(r.bits() | mask) });
        }
        1 => {
            gpio.portb_model()
                .modify(|r, w| unsafe { w.bits(set_mode(r.bits())) });
            gpio.portb_dout()
                .modify(|r, w| unsafe { w.bits(r.bits() | mask) });
        }
        2 => {
            // Pins 8..15 live in MODEH; the candidates below are all below 8.
            gpio.portc_model()
                .modify(|r, w| unsafe { w.bits(set_mode(r.bits())) });
            gpio.portc_dout()
                .modify(|r, w| unsafe { w.bits(r.bits() | mask) });
        }
        _ => {
            gpio.portd_model()
                .modify(|r, w| unsafe { w.bits(set_mode(r.bits())) });
            gpio.portd_dout()
                .modify(|r, w| unsafe { w.bits(r.bits() | mask) });
        }
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());

    // The IMU bus: PB3/PB2 open drain, released.
    dp.gpio_s
        .portb_model()
        .modify(|_r, w| w.mode2().wiredand().mode3().wiredand());
    dp.gpio_s
        .portb_dout()
        .modify(|r, w| unsafe { w.bits(r.bits() | PIN_SCL | PIN_SDA) });

    let bus = Bus { gpio: &dp.gpio_s };

    // Candidate enables: every pin that is not one of ours (PA7 = LED,
    // PC8 = mic power, PC4/PC5 = base bus, PB2/PB3 = this bus) and not obviously
    // a header pin in use. The list is ordered by plausibility.
    let candidates: [(u8, u32); 22] = [
        (3, 5), // PD5 - Seeed's named pin, first because it is documented
        (0, 1),
        (0, 2),
        (0, 3),
        (0, 4),
        (0, 5),
        (0, 6),
        (1, 0),
        (1, 1),
        (1, 4),
        (1, 5),
        (2, 0),
        (2, 1),
        (2, 2),
        (2, 3),
        (2, 6),
        (2, 7),
        (3, 0),
        (3, 1),
        (3, 2),
        (3, 3),
        (3, 4),
    ];

    let mut found_port = 0xFFu8;
    let mut found_pin = 0xFFu32;
    let mut tried = 0u32;
    let mut found_addr = IMU_ADDR;

    for (port, pin) in candidates.iter().copied() {
        drive_high(&dp.gpio_s, port, pin);
        cortex_m::asm::delay(60_000);
        tried += 1;
        // Both strapping options: SA0 low gives 0x6A, SA0 high gives 0x6B. The
        // first search only tried 0x6A, which is one way to miss a fitted part.
        if bus.probe(0x6A) {
            found_port = port;
            found_pin = pin;
            found_addr = 0x6A;
            break;
        }
        if bus.probe(0x6B) {
            found_port = port;
            found_pin = pin;
            found_addr = 0x6B;
            break;
        }
    }

    let mut msg = [0u8; 64];
    let mut pos = push_bytes(&mut msg, 0, b"tried=");
    pos = push_dec(&mut msg, pos, tried);
    pos = push_bytes(&mut msg, pos, b" en=p");
    pos = push_dec(&mut msg, pos, found_port as u32);
    pos = push_bytes(&mut msg, pos, b".");
    pos = push_dec(&mut msg, pos, found_pin);

    if found_port == 0xFF {
        mailbox::fail(&msg[..pos]);
        idle(&dp);
    }

    // The sensor answered: read its identity and one sample.
    let id = bus.read_reg(found_addr, 0x0F).unwrap_or(0);
    pos = push_bytes(&mut msg, pos, b" a=");
    pos = push_hex2(&mut msg, pos, found_addr);
    pos = push_bytes(&mut msg, pos, b" id=");
    pos = push_hex2(&mut msg, pos, id);

    bus.write_reg(found_addr, 0x12, 0b0100_0100);
    bus.write_reg(found_addr, 0x10, 0b0100_0000);
    bus.write_reg(found_addr, 0x11, 0b0100_0000);
    cortex_m::asm::delay(200_000);

    let mut raw = [0u8; 12];
    let ok = bus.read_regs(found_addr, 0x22, &mut raw);
    let word = |lo: u8, hi: u8| i16::from_le_bytes([lo, hi]) as i32;
    let ax = (word(raw[6], raw[7]) as f32 * 0.061) as i32;
    let ay = (word(raw[8], raw[9]) as f32 * 0.061) as i32;
    let az = (word(raw[10], raw[11]) as f32 * 0.061) as i32;

    let mut m2 = [0u8; 64];
    let mut p2 = push_bytes(&mut m2, 0, b"imu p");
    p2 = push_dec(&mut m2, p2, found_port as u32);
    p2 = push_bytes(&mut m2, p2, b".");
    p2 = push_dec(&mut m2, p2, found_pin);
    p2 = push_bytes(&mut m2, p2, b" id=");
    p2 = push_hex2(&mut m2, p2, id);
    p2 = push_bytes(&mut m2, p2, b" a=");
    p2 = push_signed(&mut m2, p2, ax);
    p2 = push_bytes(&mut m2, p2, b",");
    p2 = push_signed(&mut m2, p2, ay);
    p2 = push_bytes(&mut m2, p2, b",");
    p2 = push_signed(&mut m2, p2, az);

    let peak = ax.abs().max(ay.abs()).max(az.abs());
    if ok && (700..=1300).contains(&peak) {
        mailbox::pass(&m2[..p2]);
    } else {
        mailbox::fail(&m2[..p2]);
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
