//! Bit-banged I2C on PC04 (SDA) / PC05 (SCL), for the Expansion Base devices.
//!
//! ## Why bit-bang instead of the I2C0 peripheral
//!
//! The on-chip master would not complete a read: after the read address it
//! reports `STATE=0xd7` — BUSY|MASTER|**TRANSMITTER**|NACKED, state field 6
//! (DATAACK) — and never raises RXDATAV, with or without `CMD.ACK`, with and
//! without AUTOACK, with a repeated START and with two separate transactions.
//! That is with the pins correctly configured (open-drain, both lines idle high
//! at 0x30) and writes working. Rather than spend more flashes guessing at the
//! peripheral's state machine, the bus is driven from software where every edge
//! is ours: the ACK bit is sampled by us, so the answer is unambiguous.
//!
//! Speed is not the point here — correctness is. Half-period is ~5 us (about
//! 100 kHz), produced with `cortex_m::asm::delay`, which is cycle-based and
//! therefore independent of nothing but SYSCLK; the bus tolerates it.
//!
//! ## Electrical setup
//!
//! - pins in MODE 8 (SVD `WIREDAND` = open drain),
//! - `DOUT` = 0 pulls the line low, `DOUT` = 1 releases it,
//! - the pull-ups come from the Expansion Base (both lines read 1 when idle),
//! - `DIN` is read back to sample SDA.
//!
//! Reports through the mailbox: ack mask for 0x3C/0x51/0x76/0x77, the BME680
//! chip id and the PCF8563 seconds/minutes, plus a full scan mask.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Half-bit delay in cycles. ~5 us at 19 MHz; the exact figure only changes the
/// bus speed, and any value from ~1 us up is within the I2C spec for 100 kHz.
const HALF_BIT: u32 = 60;

const PIN_SDA: u32 = 1 << 4; // PC04
const PIN_SCL: u32 = 1 << 5; // PC05

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

fn push_hex2(buf: &mut [u8], mut pos: usize, value: u8) -> usize {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    if pos < buf.len() {
        buf[pos] = HEX[(value >> 4) as usize];
        pos += 1;
    }
    if pos < buf.len() {
        buf[pos] = HEX[(value & 0xF) as usize];
        pos += 1;
    }
    pos
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

struct Bus<'a> {
    gpio: &'a pac::gpio_s::RegisterBlock,
}

impl Bus<'_> {
    fn delay(&self) {
        cortex_m::asm::delay(HALF_BIT);
    }

    /// Drive (level 0) or release (level 1) both lines. Open drain: only low is
    /// driven, so a 1 means "someone else may pull it low".
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
    }

    fn sda_high(&self) -> bool {
        self.gpio.portc_din().read().bits() & PIN_SDA != 0
    }

    fn start(&self) {
        self.set(true, true); // SDA low while SCL low
        self.delay();
        self.set(true, false); // release SCL
        self.delay();
        self.set(false, false); // SDA high -> low while SCL high = START
        self.delay();
        self.set(true, false);
        self.delay();
    }

    fn stop(&self) {
        self.set(true, true);
        self.delay();
        self.set(true, false); // SCL high
        self.delay();
        self.set(false, false); // SDA low -> high while SCL high = STOP
        self.delay();
    }

    /// Send one byte MSB first; returns true when the device ACKed.
    fn write_byte(&self, byte: u8) -> bool {
        for i in (0..8).rev() {
            let bit = (byte >> i) & 1;
            self.set(bit == 0, true); // set SDA while SCL low
            self.delay();
            self.set(bit == 0, false); // SCL high
            self.delay();
            self.set(bit == 0, true); // SCL low
        }
        // ninth clock: release SDA and sample it
        self.set(false, true);
        self.delay();
        self.set(false, false);
        self.delay();
        let acked = !self.sda_high();
        self.set(false, true);
        self.delay();
        acked
    }

    /// Receive one byte; `ack` = true keeps the bus for another byte.
    fn read_byte(&self, ack: bool) -> u8 {
        let mut value = 0u8;
        self.set(false, true); // release SDA
        for _ in 0..8 {
            self.delay();
            self.set(false, false); // SCL high
            self.delay();
            value = (value << 1) | if self.sda_high() { 1 } else { 0 };
            self.set(false, true); // SCL low
        }
        // ninth clock: ACK (low) or NACK (released)
        self.set(ack, true);
        self.delay();
        self.set(ack, false);
        self.delay();
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
        if !self.write_byte(addr << 1) {
            self.stop();
            return None;
        }
        if !self.write_byte(reg) {
            self.stop();
            return None;
        }
        self.start(); // repeated START, no STOP in between
        if !self.write_byte((addr << 1) | 1) {
            self.stop();
            return None;
        }
        let value = self.read_byte(false);
        self.stop();
        Some(value)
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let gpio = &dp.gpio_s;

    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());

    // Open drain on PC04/PC05, both released (high).
    gpio.portc_model()
        .modify(|_r, w| w.mode4().wiredand().mode5().wiredand());
    gpio.portc_dout()
        .modify(|r, w| unsafe { w.bits(r.bits() | PIN_SDA | PIN_SCL) });

    let bus = Bus { gpio };

    // Full scan, so the report says what is really on the bus.
    let mut present: [u32; 4] = [0; 4];
    let mut count = 0u32;
    for addr in 0x08u8..=0x77 {
        if bus.probe(addr) {
            // 112 addresses fit in four 32-bit words (bucket size 32). Using
            // 24 per word overflowed the array at address 0x68 and the image
            // died in rust_begin_unwind -- a panic, not a bus problem.
            present[((addr - 0x08) / 32) as usize] |= 1 << ((addr - 0x08) % 32);
            count += 1;
        }
    }

    let mut buf = [0u8; 96];
    let mut pos = push_bytes(&mut buf, 0, b"bb scan n=");
    pos = push_hex8(&mut buf, pos, count);
    for (i, word) in present.iter().enumerate() {
        pos = push_bytes(&mut buf, pos, b" ");
        pos = push_hex2(&mut buf, pos, i as u8);
        pos = push_bytes(&mut buf, pos, b":");
        pos = push_hex8(&mut buf, pos, *word);
    }
    if count > 0 {
        mailbox::pass(&buf[..pos]);
    } else {
        mailbox::fail(&buf[..pos]);
    }

    // BME680: chip id lives at 0xD0 and must be 0x61.
    let bme_addr = if bus.probe(0x76) {
        Some(0x76u8)
    } else if bus.probe(0x77) {
        Some(0x77u8)
    } else {
        None
    };
    let (bme_step, bme_id) = match bme_addr {
        Some(a) => match bus.read_reg(a, 0xD0) {
            Some(v) => (0u8, v),
            None => (6u8, 0u8),
        },
        None => (0xFF, 0),
    };

    // PCF8563: seconds (0x02) and minutes (0x03), BCD, top bit = VL flag.
    let (rtc_step, rtc_sec, rtc_min) = if bus.probe(0x51) {
        match (bus.read_reg(0x51, 0x02), bus.read_reg(0x51, 0x03)) {
            (Some(s), Some(m)) => (0u8, s, m),
            _ => (6u8, 0u8, 0u8),
        }
    } else {
        (0xFF, 0u8, 0u8)
    };

    let mut buf = [0u8; 96];
    let mut pos = push_bytes(&mut buf, 0, b"bme=");
    pos = push_hex2(&mut buf, pos, bme_step);
    pos = push_bytes(&mut buf, pos, b"/0x");
    pos = push_hex2(&mut buf, pos, bme_id);
    pos = push_bytes(&mut buf, pos, b" rtc=");
    pos = push_hex2(&mut buf, pos, rtc_step);
    pos = push_bytes(&mut buf, pos, b"/0x");
    pos = push_hex2(&mut buf, pos, rtc_sec & 0x7F);
    pos = push_bytes(&mut buf, pos, b"/0x");
    pos = push_hex2(&mut buf, pos, rtc_min & 0x7F);
    if bme_step == 0 && bme_id == 0x61 && rtc_step == 0 {
        mailbox::pass(&buf[..pos]);
    } else {
        mailbox::fail(&buf[..pos]);
    }

    // Idle blink so the board is visibly alive and the result stays readable.
    gpio.porta_model().modify(|_r, w| w.mode7().pushpull());
    loop {
        for _ in 0..2_000_000 {
            cortex_m::asm::nop();
        }
        gpio.porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 7)) });
    }
}
