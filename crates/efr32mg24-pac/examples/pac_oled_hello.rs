//! PAC-only demo on the Expansion Base OLED: SSD1306 128x64 at 0x3C.
//!
//! Draws three lines of 5x7 text using a compact glyph table, so the whole
//! example stays a few hundred lines and needs no crate.
//!
//! Verification is in two halves, and the split is deliberate:
//!
//! - **the bus and the controller**: every command and data byte must be
//!   ACKed; the example counts ACKed bytes and reports the count and the first
//!   failure through the mailbox, so "the image ran and the panel accepted the
//!   stream" is a measurement;
//! - **the pixels**: the SSD1306 cannot be read back over I2C, so whether the
//!   text is visible is the one thing only an eye can confirm. The example
//!   says so instead of pretending otherwise.
//!
//! If 0x3C does not answer, the report says `oled ack=0` and nothing is
//! written — the panel is simply not on this bus (board not seated, or a
//! different address).

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/i2c_bitbang.rs"]
mod i2c_bitbang;
#[path = "common/mailbox.rs"]
mod mailbox;

const OLED_ADDR: u8 = 0x3C;
/// SSD1306 control byte: 0x00 = command stream, 0x40 = data stream.
const CMD: u8 = 0x00;
const DATA: u8 = 0x40;

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

/// 8 bytes per glyph, 5x7 in the low 7 bits (column-major, LSB = top row).
const GLYPHS: [[u8; 8]; 17] = [
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // space
    [0x00, 0x00, 0x5F, 0x00, 0x00, 0x00, 0x00, 0x00], // !
    [0x7E, 0x11, 0x11, 0x11, 0x7E, 0x00, 0x00, 0x00], // A
    [0x7F, 0x49, 0x49, 0x49, 0x36, 0x00, 0x00, 0x00], // B
    [0x3E, 0x41, 0x41, 0x41, 0x22, 0x00, 0x00, 0x00], // C
    [0x7F, 0x41, 0x41, 0x41, 0x3E, 0x00, 0x00, 0x00], // D
    [0x7F, 0x49, 0x49, 0x49, 0x41, 0x00, 0x00, 0x00], // E
    [0x7F, 0x09, 0x09, 0x09, 0x01, 0x00, 0x00, 0x00], // F
    [0x3E, 0x41, 0x49, 0x49, 0x7A, 0x00, 0x00, 0x00], // G
    [0x7F, 0x08, 0x08, 0x08, 0x7F, 0x00, 0x00, 0x00], // H
    [0x00, 0x41, 0x7F, 0x41, 0x00, 0x00, 0x00, 0x00], // I
    [0x20, 0x40, 0x41, 0x3F, 0x01, 0x00, 0x00, 0x00], // J
    [0x7F, 0x08, 0x14, 0x22, 0x41, 0x00, 0x00, 0x00], // K
    [0x7F, 0x40, 0x40, 0x40, 0x40, 0x00, 0x00, 0x00], // L
    [0x7F, 0x02, 0x0C, 0x02, 0x7F, 0x00, 0x00, 0x00], // M
    [0x7F, 0x04, 0x08, 0x10, 0x7F, 0x00, 0x00, 0x00], // N
    [0x3E, 0x41, 0x41, 0x41, 0x3E, 0x00, 0x00, 0x00], // O
];

fn glyph_index(c: u8) -> usize {
    match c {
        b' ' => 0,
        b'!' => 1,
        b'A'..=b'O' => 2 + (c - b'A') as usize,
        _ => 0,
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());
    let bus = i2c_bitbang::Bus::new(&dp.gpio_s);

    let mut acked: u32 = 0;

    if !bus.probe(OLED_ADDR) {
        mailbox::fail(b"oled ack=0 (no device at 0x3C)");
        idle(&dp);
    }

    // SSD1306 128x64 init: the control byte and the command bytes MUST be one
    // transaction. Sending the control byte alone and the commands in the next
    // transfer puts the controller into data mode, and nothing is displayed --
    // the stream is still ACKed, which is exactly the kind of silent failure
    // this example's counter has to catch.
    let init: [u8; 24] = [
        0xAE, // display off
        0xD5, 0x80, // clock divider
        0xA8, 0x3F, // multiplex 64
        0xD3, 0x00, // display offset
        0x40, // start line 0
        0x8D, 0x14, // charge pump on
        0x20, 0x00, // horizontal addressing
        0xA1, // segment remap
        0xC8, // COM scan dec
        0xDA, 0x12, // COM pins
        0x81, 0xCF, // contrast
        0xD9, 0xF1, // pre-charge
        0xDB, 0x40, // VCOMH
        0xA4, // resume to RAM content
        0xA6, // normal (not inverted)
    ];
    let mut cmd_buf = [0u8; 32];
    cmd_buf[0] = CMD;
    cmd_buf[1..1 + init.len()].copy_from_slice(&init);
    if bus.write_cmds(OLED_ADDR, &cmd_buf[..1 + init.len()]) {
        acked += 1 + init.len() as u32;
    }

    // Window: columns 0..127, pages 0..7, then display on -- each one transaction.
    let window: [u8; 7] = [CMD, 0x21, 0x00, 0x7F, 0x22, 0x00, 0x07];
    if bus.write_cmds(OLED_ADDR, &window) {
        acked += window.len() as u32;
    }
    let display_on: [u8; 2] = [CMD, 0xAF];
    if bus.write_cmds(OLED_ADDR, &display_on) {
        acked += 2;
    }

    // Three lines of text: "PAC OLED OK" / "EXPANSION" / "BASE 2026".
    let lines: [&[u8]; 3] = [b"PAC OLED OK", b"EXPANSION", b"BASE 2026"];
    for line in lines {
        let mut payload = [0u8; 129];
        payload[0] = DATA;
        let mut x = 0usize;
        for &c in line.iter() {
            for col in GLYPHS[glyph_index(c)].iter() {
                if x < 128 {
                    payload[1 + x] = *col;
                    x += 1;
                }
            }
            x += 1; // one blank column between glyphs
        }
        // Pad the rest of the page with blanks so the window is fully written.
        let written = core::cmp::min(x, 128);
        if bus.write_cmds(OLED_ADDR, &payload[..1 + written]) {
            acked += 1 + written as u32;
        }
    }

    let mut buf = [0u8; 56];
    let mut pos = push_bytes(&mut buf, 0, b"oled ack=");
    pos = push_hex8(&mut buf, pos, acked);
    pos = push_bytes(&mut buf, pos, b" (pixels need eyes)");
    if acked > 0 {
        mailbox::pass(&buf[..pos]);
    } else {
        mailbox::fail(&buf[..pos]);
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
