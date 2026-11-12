//! PAC-level IMU: the on-board LSM6DS3TR-C at I2C address 0x6A.
//!
//! ## The three facts this example needed
//!
//! 1. **Bus:** the IMU is not on the Expansion Base bus (PC4/PC5, where the panel,
//!    the RTC and the BME68x live). It is on its own pair, **PB2 = SDA, PB3 = SCL**.
//! 2. **Line order:** that order is the measured one. The Arduino variant declares
//!    `PIN_WIRE_SDA1 = PB3` / `PIN_WIRE_SCL1 = PB2`, i.e. the opposite, and with
//!    the lines swapped the sensor never answers — which is why every earlier scan
//!    of "the right pins" came back empty. `pac_imu_probe` found this by trying
//!    both assignments.
//! 3. **Power:** PD5 must be driven high (Seeed: "turn on the gyro enable pin"),
//!    then the part needs a few milliseconds before it answers.
//!
//! With those three right, `WHO_AM_I` (0x0F) returns **0x6A**, the identity the
//! documentation states for the LSM6DS3TR-C.
//!
//! ## What it reports
//!
//! Acceleration on three axes in milli-g and angular rate in milli-degrees per
//! second, through the mailbox, then keeps the LED blinking so the board is
//! visibly alive while the values are read over SWD.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

const HALF_BIT: u32 = 60;

// Measured wiring, not the one in the Arduino variant.
const PIN_SDA: u32 = 1 << 2; // PB2
const PIN_SCL: u32 = 1 << 3; // PB3
const PIN_IMU_POWER: u32 = 1 << 5; // PD5
const PIN_LED: u32 = 1 << 7; // PA7, active low

const IMU_ADDR: u8 = 0x6A;

const REG_WHO_AM_I: u8 = 0x0F;
const REG_CTRL1_XL: u8 = 0x10;
const REG_CTRL2_G: u8 = 0x11;
const REG_CTRL3_C: u8 = 0x12;
const REG_OUTX_L_G: u8 = 0x22;

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
    fn new(gpio: &pac::gpio_s::RegisterBlock) {
        // Open drain: mode 8 is `wire[d]and` on this die, which is what an I2C
        // master needs so the pull-ups can win.
        gpio.portb_model()
            .modify(|_r, w| w.mode2().wiredand().mode3().wiredand());
        gpio.portb_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() | PIN_SDA | PIN_SCL) });
    }

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

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());

    // LED first so the board is visibly alive even if the sensor is not.
    dp.gpio_s.porta_model().modify(|_r, w| w.mode7().pushpull());
    dp.gpio_s
        .porta_dout()
        .modify(|r, w| unsafe { w.bits(r.bits() | PIN_LED) });

    // Power the IMU: Seeed's own sketch drives PD5 high before talking to it.
    dp.gpio_s.portd_model().modify(|_r, w| w.mode5().pushpull());
    dp.gpio_s
        .portd_dout()
        .modify(|r, w| unsafe { w.bits(r.bits() | PIN_IMU_POWER) });
    cortex_m::asm::delay(400_000);

    Bus::new(&dp.gpio_s);
    let bus = Bus { gpio: &dp.gpio_s };
    cortex_m::asm::delay(40_000);

    let Some(id) = bus.read_reg(IMU_ADDR, REG_WHO_AM_I) else {
        idle(&dp);
    };

    // BDU + IF_INC, 104 Hz on both sensors, +/-2 g and +/-250 dps.
    bus.write_reg(IMU_ADDR, REG_CTRL3_C, 0b0100_0100);
    bus.write_reg(IMU_ADDR, REG_CTRL1_XL, 0b0100_0000);
    bus.write_reg(IMU_ADDR, REG_CTRL2_G, 0b0100_0000);
    cortex_m::asm::delay(400_000);

    let mut raw = [0u8; 12];
    let mut reported = false;

    loop {
        if !reported {
            reported = true;
            if bus.read_regs(IMU_ADDR, REG_OUTX_L_G, &mut raw) {
                let word = |lo: u8, hi: u8| i16::from_le_bytes([lo, hi]) as i32;
                // 0.061 mg/LSB at +/-2 g; 8.75 mdps/LSB at +/-250 dps.
                let ax = word(raw[6], raw[7]) * 61 / 1000;
                let ay = word(raw[8], raw[9]) * 61 / 1000;
                let az = word(raw[10], raw[11]) * 61 / 1000;
                let gx = word(raw[0], raw[1]) * 875 / 100;
                let gy = word(raw[2], raw[3]) * 875 / 100;
                let gz = word(raw[4], raw[5]) * 875 / 100;

                let mut msg = [0u8; 64];
                let mut pos = push_bytes(&mut msg, 0, b"id=");
                pos = push_hex2(&mut msg, pos, id);
                pos = push_bytes(&mut msg, pos, b" a=");
                pos = push_signed(&mut msg, pos, ax);
                pos = push_bytes(&mut msg, pos, b",");
                pos = push_signed(&mut msg, pos, ay);
                pos = push_bytes(&mut msg, pos, b",");
                pos = push_signed(&mut msg, pos, az);
                pos = push_bytes(&mut msg, pos, b"mg g=");
                pos = push_signed(&mut msg, pos, gx);
                pos = push_bytes(&mut msg, pos, b",");
                pos = push_signed(&mut msg, pos, gy);
                pos = push_bytes(&mut msg, pos, b",");
                pos = push_signed(&mut msg, pos, gz);

                // A part lying still on a desk must report ~1000 mg on one axis.
                // Squared magnitude, compared in mg^2: 800..1200 mg becomes
                // 640_000..1_440_000, and no floating point (or libm) is needed.
                let squared = ax * ax + ay * ay + az * az;
                if id == 0x6A && (640_000..=1_440_000).contains(&squared) {
                    mailbox::pass(&msg[..pos]);
                } else {
                    mailbox::fail(&msg[..pos]);
                }
            } else {
                let mut msg = [0u8; 64];
                let mut pos = push_bytes(&mut msg, 0, b"id=");
                pos = push_hex2(&mut msg, pos, id);
                pos = push_bytes(&mut msg, pos, b" but output read failed");
                mailbox::fail(&msg[..pos]);
            }
        }

        // Slow blink, so the board can be seen running while values are read.
        for _ in 0..3_000_000 {
            cortex_m::asm::nop();
        }
        dp.gpio_s
            .porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ PIN_LED) });
    }
}

fn idle(dp: &pac::Peripherals) -> ! {
    mailbox::fail(b"imu no ack at 0x6a on PB2/PB3");
    loop {
        for _ in 0..2_000_000 {
            cortex_m::asm::nop();
        }
        dp.gpio_s
            .porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ PIN_LED) });
    }
}
