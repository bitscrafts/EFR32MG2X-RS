//! PAC-only I2C bus scan on I2C0, pins PC04 (SDA) / PC05 (SCL).
//!
//! The XIAO pin map puts I2C on D4 = PC04 (SDA) and D5 = PC05 (SCL), which is
//! the pair the Expansion Base and the Grove I2C connectors share. Scanning
//! 0x08..0x77 answers three questions at once:
//!
//! 1. does our I2C master work at all;
//! 2. which devices are physically answering (OLED SSD1306 0x3C, RTC PCF8563
//!    0x51, BME680 0x76/0x77 are the expected ones on the Expansion Base);
//! 3. is the board actually seated in the base with the sensors connected.
//!
//! Everything is reported through the mailbox, one line per device found plus a
//! summary, so the result is read back over SWD (`scripts/read-mailbox.sh`).
//!
//! ## Safety
//!
//! Every wait is bounded. A bus with no pull-ups, a held SDA line or a missing
//! device must not hang the image: on timeout the transfer is ABORTed, the
//! peripheral is reset with CLEARPC and the scan continues. Nothing here writes
//! to flash, clocks or the regulator; the worst case is a scan that reports
//! nothing.
//!
//! ## Register facts used (verified against the SVD in this crate)
//!
//! - `CMU_CLKEN0.I2C0` is bit 14, `GPIO` is bit 26;
//! - `I2C0_S` base is 0x4B00_0000;
//! - pins are routed with `GPIO_S.I2C0_SCLROUTE/SDAROUTE` (port/pin) and
//!   enabled with `I2C0_ROUTEEN.SCLPEN/SDAPEN`;
//! - the pins are set to WIREDOR (open-drain) because I2C is a wired-AND bus.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Assumed HFCLK for the divider; any sane value scans successfully, the clock
/// only sets how fast SCL toggles.
const HFCLK_HZ: u32 = 19_000_000;
/// Target SCL frequency for a scan: slow is fine and more tolerant.
const SCL_HZ: u32 = 100_000;
/// Upper bound for any single peripheral wait. A byte at 100 kHz takes ~90 us,
/// i.e. under 2000 cycles at 19 MHz; 50 000 spins is two orders of magnitude
/// of headroom. The first version used 500 000 and a bus with no pull-ups made
/// the scan take minutes (three waits per address x 112 addresses).
const SPIN_LIMIT: u32 = 50_000;

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

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let i2c = &dp.i2c0_s;

    // Bus clocks: GPIO and I2C0 (bit 26 and bit 14 of CMU_CLKEN0).
    dp.cmu_s
        .clken0()
        .modify(|_r, w| w.gpio().set_bit().i2c0().set_bit());

    // PC04 = SDA, PC05 = SCL. PORTC_MODEL holds pins 0-7; mode4/mode5 are the
    // fields for pins 4 and 5.
    //
    // WIREDOR (mode 6) is WRONG for I2C and silently hangs the bus: the SVD
    // calls mode 8 WIREDAND = "Open-drain output", and open-drain is what a
    // wired-AND bus needs. With mode 6 the peripheral sat in STATE=START with
    // PSTART pending forever (measured: STATE=0x42 -> field 2 = START,
    // STATUS=0x801 -> PSTART=1, bus idle high 0x30), so every scan reported
    // "0 devices" while the bus was in fact healthy.
    dp.gpio_s
        .portc_model()
        .modify(|_r, w| w.mode4().wiredand().mode5().wiredand());

    // DOUT high = release both lines (open-drain only pulls low).
    dp.gpio_s
        .portc_dout()
        .modify(|r, w| unsafe { w.bits(r.bits() | (1 << 4) | (1 << 5)) });

    // Route the peripheral to those pins and enable the two routes.
    // PORT field: 0 = PORTA, 1 = PORTB, 2 = PORTC, 3 = PORTD (plain field,
    // no enum in the SVD). PIN is a 4-bit field.
    dp.gpio_s
        .i2c0_sclroute()
        .write(|w| unsafe { w.port().bits(2).pin().bits(5) });
    dp.gpio_s
        .i2c0_sdaroute()
        .write(|w| unsafe { w.port().bits(2).pin().bits(4) });
    dp.gpio_s
        .i2c0_routeen()
        .write(|w| w.sclpen().set_bit().sdapen().set_bit());

    // Master mode, standard clock ratio, divider for ~100 kHz.
    i2c.en().write(|w| w.en().set_bit());
    i2c.ctrl().write(|w| w.slave().disable().clhr().standard());
    let clkdiv = (HFCLK_HZ / (8 * SCL_HZ)).saturating_sub(1).min(0x1FF);
    i2c.clkdiv().write(|w| unsafe { w.bits(clkdiv) });

    mailbox::info(b"boot: i2c0 scan on PC04/PC05");

    let mut found: u32 = 0;
    let mut addr_lo: u32 = 0;
    let mut addr_hi: u32 = 0;

    for addr in 0x08u8..=0x77 {
        // START
        i2c.cmd().write(|w| w.start().set_bit());
        let mut spin = 0u32;
        while i2c.status().read().pstart().bit_is_set() {
            spin += 1;
            if spin > SPIN_LIMIT {
                break;
            }
        }

        // Address byte, write direction.
        i2c.saddr().write(|w| unsafe { w.addr().bits(addr << 1) });
        i2c.txdata().write(|w| unsafe { w.txdata().bits(0) });

        // Wait for the byte to leave; a NACK means "nobody at this address".
        let mut acked = false;
        let mut spin = 0u32;
        loop {
            if i2c.if_().read().nack().bit_is_set() {
                i2c.if_().write(|w| w.nack().set_bit());
                break;
            }
            if i2c.status().read().txc().bit_is_set() {
                acked = true;
                break;
            }
            spin += 1;
            if spin > SPIN_LIMIT {
                break;
            }
        }

        // STOP (always, even on timeout: never leave the bus held).
        if !acked {
            i2c.cmd().write(|w| w.abort().set_bit());
        }
        i2c.cmd().write(|w| w.stop().set_bit());
        let mut spin = 0u32;
        while i2c.status().read().pstop().bit_is_set() {
            spin += 1;
            if spin > SPIN_LIMIT {
                break;
            }
        }
        i2c.cmd().write(|w| w.clearpc().set_bit());

        if addr % 16 == 0 {
            let mut buf = [0u8; 24];
            let mut pos = push_bytes(&mut buf, 0, b"scan at 0x");
            pos = push_hex2(&mut buf, pos, addr);
            mailbox::info(&buf[..pos]);
        }

        if acked {
            found += 1;
            if addr < 0x40 {
                addr_lo |= 1 << (addr - 0x08);
            } else {
                addr_hi |= 1 << (addr - 0x40);
            }

            let mut buf = [0u8; 32];
            let mut pos = push_bytes(&mut buf, 0, b"i2c ack 0x");
            pos = push_hex2(&mut buf, pos, addr);
            pos = push_bytes(&mut buf, pos, b" ok");
            mailbox::pass(&buf[..pos]);
        }
    }

    let mut buf = [0u8; 48];
    let mut pos = push_bytes(&mut buf, 0, b"scan done found=");
    pos = push_hex8(&mut buf, pos, found);
    pos = push_bytes(&mut buf, pos, b" lo=");
    pos = push_hex8(&mut buf, pos, addr_lo);
    pos = push_bytes(&mut buf, pos, b" hi=");
    pos = push_hex8(&mut buf, pos, addr_hi);
    if found == 0 {
        mailbox::fail(&buf[..pos]);
    } else {
        mailbox::pass(&buf[..pos]);
    }

    // Idle: keep the result in the mailbox and blink slowly so the board is
    // visibly alive without a timer dependency.
    let mut led_on = false;
    loop {
        for _ in 0..2_000_000 {
            cortex_m::asm::nop();
        }
        dp.gpio_s.porta_model().modify(|_r, w| w.mode7().pushpull());
        dp.gpio_s
            .porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 7)) });
        led_on = !led_on;
    }
}
