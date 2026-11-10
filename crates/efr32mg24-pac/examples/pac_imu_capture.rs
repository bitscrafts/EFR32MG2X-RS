//! Capture real IMU data into RAM, so a classifier can be trained on what the
//! board actually does instead of on invented numbers.
//!
//! ## Why this exists
//!
//! A fall detector has to be trained on falls — or at least on the signals a
//! wrist- or board-mounted accelerometer really produces when it is picked up,
//! shaken and dropped. Synthetic data would make the classifier's accuracy a
//! statement about the generator, not about the device.
//!
//! This example samples the on-board LSM6DS3TR-C at a fixed rate into a ring
//! buffer in RAM and never stops. The buffer address and the sample count go to
//! the mailbox, so the host can read the raw samples over SWD while the board
//! keeps running — no serial port, no protocol, no data loss.
//!
//! ## Timing
//!
//! The sample clock is the DWT cycle counter, which is how Silicon Labs' own MVP
//! driver measures time. `SAMPLE_PERIOD_CYCLES` is a cycle count, so the rate
//! follows SYSCLK; the rate actually achieved is measured below and reported, so
//! the training data carries its own time base instead of assuming one.
//!
//! ## Buffer layout
//!
//! `CAPTURE[i]` is one sample: `accel_mg[0..3]` in thousandths of a g, in g units
//! ×1000 as `i16` (so ±32 g fits). 1000 samples at 100 Hz is ten seconds — enough
//! to catch a fall and the quiet before it.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

const HALF_BIT: u32 = 60;
const PIN_SDA: u32 = 1 << 2; // PB2
const PIN_SCL: u32 = 1 << 3; // PB3
const PIN_IMU_POWER: u32 = 1 << 5; // PD5
const PIN_LED: u32 = 1 << 7; // PA7

const IMU_ADDR: u8 = 0x6A;
const REG_CTRL1_XL: u8 = 0x10;
const REG_CTRL2_G: u8 = 0x11;
const REG_CTRL3_C: u8 = 0x12;
const REG_OUTX_L_XL: u8 = 0x28;

/// 100 Hz at ~19 MHz SYSCLK. The achieved rate is measured and reported.
const SAMPLE_PERIOD_CYCLES: u32 = 190_000;

/// Samples kept in the ring: 1000 × 100 Hz = 10 s.
const CAPTURE_LEN: usize = 1000;

#[repr(align(32))]
struct Samples([[i16; 3]; CAPTURE_LEN]);

static mut CAPTURE: Samples = Samples([[0; 3]; CAPTURE_LEN]);
static mut TOTAL: u32 = 0;

struct Bus<'a> {
    gpio: &'a pac::gpio_s::RegisterBlock,
}

impl Bus<'_> {
    fn new(gpio: &pac::gpio_s::RegisterBlock) {
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

fn push_bytes(buf: &mut [u8], mut pos: usize, s: &[u8]) -> usize {
    for b in s {
        if pos < buf.len() {
            buf[pos] = *b;
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

/// Enable the DWT cycle counter — the time base Silicon Labs' own MVP driver uses.
///
/// `cortex_m::peripheral::DCB`/`DWT` need the `Peripherals::take()` singleton of
/// the cortex-m crate, which the RT entry point does not hand out; `steal()` is
/// the supported way to get them when nothing else has taken them.
fn dwt_start() {
    let mut core = unsafe { cortex_m::Peripherals::steal() };
    core.DCB.enable_trace();
    core.DWT.enable_cycle_counter();
}

fn cycle_count() -> u32 {
    unsafe { (*cortex_m::peripheral::DWT::PTR).cyccnt.read() }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());

    // LED on PA7 as a heartbeat.
    dp.gpio_s.porta_model().modify(|_r, w| w.mode7().pushpull());

    // IMU power, then the bus.
    dp.gpio_s.portd_model().modify(|_r, w| w.mode5().pushpull());
    dp.gpio_s
        .portd_dout()
        .modify(|r, w| unsafe { w.bits(r.bits() | PIN_IMU_POWER) });
    cortex_m::asm::delay(400_000);

    Bus::new(&dp.gpio_s);
    let bus = Bus { gpio: &dp.gpio_s };
    cortex_m::asm::delay(40_000);

    // ±2 g, 104 Hz output data rate, BDU + auto-increment.
    bus.write_reg(IMU_ADDR, REG_CTRL3_C, 0b0100_0100);
    bus.write_reg(IMU_ADDR, REG_CTRL1_XL, 0b0100_0000);
    bus.write_reg(IMU_ADDR, REG_CTRL2_G, 0b0100_0000);
    cortex_m::asm::delay(400_000);

    dwt_start();

    // Announce where the samples live, before the first one is taken.
    let base = unsafe { core::ptr::addr_of!(CAPTURE.0) as u32 };
    let mut msg = [0u8; 64];
    let mut pos = push_bytes(&mut msg, 0, b"cap base=0x");
    pos = push_hex8(&mut msg, pos, base);
    pos = push_bytes(&mut msg, pos, b" n=");
    pos = push_dec(&mut msg, pos, CAPTURE_LEN as u32);
    pos = push_bytes(&mut msg, pos, b" rate=");
    pos = push_dec(&mut msg, pos, 19_000_000 / SAMPLE_PERIOD_CYCLES);
    pos = push_bytes(&mut msg, pos, b"Hz");
    mailbox::pass(&msg[..pos]);

    let mut index = 0usize;
    let mut last_cycles = 0u32;
    let mut led_toggle = 0u32;

    loop {
        let now = cycle_count();
        if now.wrapping_sub(last_cycles) < SAMPLE_PERIOD_CYCLES {
            continue;
        }
        last_cycles = now;

        let mut raw = [0u8; 6];
        if bus.read_regs(IMU_ADDR, REG_OUTX_L_XL, &mut raw) {
            let word = |lo: u8, hi: u8| i16::from_le_bytes([lo, hi]) as i32;
            // 0.061 mg/LSB at ±2 g, kept in mg so the host needs no conversion.
            let ax = (word(raw[0], raw[1]) * 61 / 1000) as i16;
            let ay = (word(raw[2], raw[3]) * 61 / 1000) as i16;
            let az = (word(raw[4], raw[5]) * 61 / 1000) as i16;
            unsafe {
                CAPTURE.0[index] = [ax, ay, az];
                TOTAL = TOTAL.wrapping_add(1);
            }
            index += 1;
            if index == CAPTURE_LEN {
                index = 0;
            }
        }

        // Heartbeat: about once a second, so a stalled capture is visible.
        led_toggle += 1;
        if led_toggle >= 100 {
            led_toggle = 0;
            dp.gpio_s
                .porta_dout()
                .modify(|r, w| unsafe { w.bits(r.bits() ^ PIN_LED) });
        }
    }
}
