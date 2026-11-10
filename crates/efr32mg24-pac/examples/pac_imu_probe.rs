//! Find the IMU by trying pin pairs and both SDA/SCL orders.
//!
//! ## Why this exists
//!
//! Seeed's documentation is unambiguous about the device — `LSM6DS3 myIMU(I2C_MODE,
//! 0x6A)` with `pinMode(PD5); digitalWrite(PD5, HIGH)` — and their driver switches
//! the bus with `#define Wire Wire1` for `ARDUINO_XIAO_MG24`. Everything about the
//! address and the enable is therefore known. What is *not* proven is which two
//! pins carry Wire1 on this module, and in which order.
//!
//! A swapped pair is invisible to every other test: the addresses are right, the
//! enable is right, the pull-ups are there, and the sensor simply never sees a
//! clock edge in the right place. This example treats the pin pair as the unknown
//! and tries each candidate with both assignments.
//!
//! ## Method
//!
//! For every candidate pair: configure both pins as open drain (`wire[d]and`,
//! which on this die is mode 8), release them, and probe 0x6A and 0x6B. The first
//! pair that answers is reported along with its `WHO_AM_I`. The report names the
//! pins as `p<port>.<bit>` so the answer can be turned into a pin map.
//!
//! PD5 (the documented gyro enable) and PC8 (the microphone rail) are both held
//! high throughout, so a missing enable cannot hide the answer.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

const HALF_BIT: u32 = 60;
const HEX: &[u8; 16] = b"0123456789abcdef";

#[derive(Clone, Copy)]
struct Pin {
    port: u8,
    bit: u32,
}

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

fn push_pin(buf: &mut [u8], pos: usize, pin: Pin) -> usize {
    let pos = push_bytes(buf, pos, b"p");
    let pos = push_dec(buf, pos, pin.port as u32);
    let pos = push_bytes(buf, pos, b".");
    push_dec(buf, pos, pin.bit)
}

struct Bus<'a> {
    gpio: &'a pac::gpio_s::RegisterBlock,
    sda: Pin,
    scl: Pin,
}

impl Bus<'_> {
    fn mode_open_drain(&self) {
        for pin in [self.sda, self.scl] {
            let shift = pin.bit * 4;
            let to_od = |cur: u32| (cur & !(0xF << shift)) | (8 << shift); // 8 = wiredand
            match pin.port {
                0 => {
                    self.gpio
                        .porta_model()
                        .modify(|r, w| unsafe { w.bits(to_od(r.bits())) });
                }
                1 => {
                    self.gpio
                        .portb_model()
                        .modify(|r, w| unsafe { w.bits(to_od(r.bits())) });
                }
                2 => {
                    self.gpio
                        .portc_model()
                        .modify(|r, w| unsafe { w.bits(to_od(r.bits())) });
                }
                _ => {
                    self.gpio
                        .portd_model()
                        .modify(|r, w| unsafe { w.bits(to_od(r.bits())) });
                }
            }
        }
        // Release both.
        let mask = (1 << self.sda.bit) | (1 << self.scl.bit);
        match self.sda.port {
            0 => {
                self.gpio
                    .porta_dout()
                    .modify(|r, w| unsafe { w.bits(r.bits() | mask) });
            }
            1 => {
                self.gpio
                    .portb_dout()
                    .modify(|r, w| unsafe { w.bits(r.bits() | mask) });
            }
            2 => {
                self.gpio
                    .portc_dout()
                    .modify(|r, w| unsafe { w.bits(r.bits() | mask) });
            }
            _ => {
                self.gpio
                    .portd_dout()
                    .modify(|r, w| unsafe { w.bits(r.bits() | mask) });
            }
        }
    }

    fn delay(&self) {
        cortex_m::asm::delay(HALF_BIT);
    }

    fn set(&self, sda_low: bool, scl_low: bool) {
        let mut bits = match self.sda.port {
            0 => self.gpio.porta_dout().read().bits(),
            1 => self.gpio.portb_dout().read().bits(),
            2 => self.gpio.portc_dout().read().bits(),
            _ => self.gpio.portd_dout().read().bits(),
        };
        let sda = 1u32 << self.sda.bit;
        let scl = 1u32 << self.scl.bit;
        bits = if sda_low { bits & !sda } else { bits | sda };
        bits = if scl_low { bits & !scl } else { bits | scl };
        match self.sda.port {
            0 => {
                self.gpio.porta_dout().write(|w| unsafe { w.bits(bits) });
            }
            1 => {
                self.gpio.portb_dout().write(|w| unsafe { w.bits(bits) });
            }
            2 => {
                self.gpio.portc_dout().write(|w| unsafe { w.bits(bits) });
            }
            _ => {
                self.gpio.portd_dout().write(|w| unsafe { w.bits(bits) });
            }
        }
        self.delay();
    }

    fn sda_high(&self) -> bool {
        let din = match self.sda.port {
            0 => self.gpio.porta_din().read().bits(),
            1 => self.gpio.portb_din().read().bits(),
            2 => self.gpio.portc_din().read().bits(),
            _ => self.gpio.portd_din().read().bits(),
        };
        din & (1 << self.sda.bit) != 0
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
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());

    // Documented enable (PD5) plus the microphone rail: both high, so a power
    // gate can never be the reason for silence.
    dp.gpio_s.portd_model().modify(|_r, w| w.mode5().pushpull());
    dp.gpio_s
        .portd_dout()
        .modify(|r, w| unsafe { w.bits(r.bits() | (1 << 5)) });
    dp.gpio_s.portc_modeh().modify(|_r, w| w.mode0().pushpull());
    dp.gpio_s
        .portc_dout()
        .modify(|r, w| unsafe { w.bits(r.bits() | (1 << 8)) });
    cortex_m::asm::delay(600_000);

    // Candidate wire pairs, both orders. PB3/PB2 is the variant's Wire1; the
    // others are pins that measurably carry external pull-ups or that the module
    // could plausibly use for a private bus.
    let pairs: [(Pin, Pin); 8] = [
        (Pin { port: 1, bit: 3 }, Pin { port: 1, bit: 2 }),
        (Pin { port: 1, bit: 2 }, Pin { port: 1, bit: 3 }),
        (Pin { port: 0, bit: 1 }, Pin { port: 0, bit: 2 }),
        (Pin { port: 0, bit: 2 }, Pin { port: 0, bit: 1 }),
        (Pin { port: 0, bit: 4 }, Pin { port: 0, bit: 5 }),
        (Pin { port: 0, bit: 5 }, Pin { port: 0, bit: 4 }),
        (Pin { port: 2, bit: 4 }, Pin { port: 2, bit: 5 }),
        (Pin { port: 2, bit: 5 }, Pin { port: 2, bit: 4 }),
    ];

    let mut winner: Option<(Bus, u8)> = None;
    for (sda, scl) in pairs.iter().copied() {
        let bus = Bus {
            gpio: &dp.gpio_s,
            sda,
            scl,
        };
        bus.mode_open_drain();
        bus.set(false, false);
        cortex_m::asm::delay(20_000);
        for addr in [0x6Au8, 0x6Bu8] {
            if bus.probe(addr) {
                winner = Some((
                    Bus {
                        gpio: &dp.gpio_s,
                        sda,
                        scl,
                    },
                    addr,
                ));
                break;
            }
        }
        if winner.is_some() {
            break;
        }
    }

    let mut msg = [0u8; 64];
    match winner {
        None => {
            let pos = push_bytes(&mut msg, 0, b"no ack on 8 pairs x2 addrs");
            mailbox::fail(&msg[..pos]);
        }
        Some((bus, addr)) => {
            bus.mode_open_drain();
            bus.set(false, false);
            cortex_m::asm::delay(20_000);
            let id = bus.read_reg(addr, 0x0F).unwrap_or(0);
            let mut pos = push_bytes(&mut msg, 0, b"imu sda=");
            pos = push_pin(&mut msg, pos, bus.sda);
            pos = push_bytes(&mut msg, pos, b" scl=");
            pos = push_pin(&mut msg, pos, bus.scl);
            pos = push_bytes(&mut msg, pos, b" a=");
            pos = push_hex2(&mut msg, pos, addr);
            pos = push_bytes(&mut msg, pos, b" id=");
            pos = push_hex2(&mut msg, pos, id);
            // 0x69 = LSM6DS3, 0x6A = LSM6DS3TR-C, 0x6C = LSM6DSO family.
            if id == 0x69 || id == 0x6A || id == 0x6C {
                mailbox::pass(&msg[..pos]);
            } else {
                mailbox::fail(&msg[..pos]);
            }
        }
    }

    loop {
        cortex_m::asm::nop();
    }
}
