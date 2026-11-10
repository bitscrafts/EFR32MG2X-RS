//! Minimal SSD1306 128x64 driver over the Expansion Base I2C bus.
//!
//! ## What produced garbage on the panel (and why)
//!
//! The first version mixed addressing modes: the init sequence selected
//! **horizontal** addressing (`0x20, 0x00`), and the drawing code then used the
//! **page** commands (`0xB0+page`) to place lines. In horizontal mode the
//! controller ignores those page commands and auto-wraps inside the window, so
//! every write landed somewhere else and the panel showed noise.
//!
//! This version is consistent: page addressing (`0x20, 0x02`), an explicit
//! page + column before each 128-byte row, and a full-frame flush of all eight
//! pages so nothing from a previous frame survives.
//!
//! ## Transfer rules that cost a display each
//!
//! - **The control byte and its payload must be in one transaction.** A lone
//!   `0x00` (command stream) followed by a second transfer puts the controller
//!   in data mode: every byte is ACKed and nothing appears.
//! - **Column addressing needs both nibbles**: `0x00` (low) and `0x10` (high),
//!   or the next write starts at whatever column the last one ended on.
//!
//! The panel cannot be read back over I2C, so "the pixels are right" is the one
//! claim this driver cannot make; it counts the bytes the controller ACKed.

use efr32mg24_pac as pac;

use crate::i2c_bitbang::Bus;

pub const ADDR: u8 = 0x3C;
const CMD: u8 = 0x00;
const DATA: u8 = 0x40;

/// 5x7 glyphs, column-major, LSB = top row. Only the characters used by the
/// examples are present; anything else renders blank.
const GLYPHS: [(u8, [u8; 5]); 27] = [
    (b' ', [0x00, 0x00, 0x00, 0x00, 0x00]),
    (b'.', [0x00, 0x60, 0x60, 0x00, 0x00]),
    (b'-', [0x08, 0x08, 0x08, 0x08, 0x08]),
    (b'%', [0x23, 0x13, 0x08, 0x64, 0x62]),
    (b'0', [0x3E, 0x51, 0x49, 0x45, 0x3E]),
    (b'1', [0x00, 0x42, 0x7F, 0x40, 0x00]),
    (b'2', [0x42, 0x61, 0x51, 0x49, 0x46]),
    (b'3', [0x21, 0x41, 0x45, 0x4B, 0x31]),
    (b'4', [0x18, 0x14, 0x12, 0x7F, 0x10]),
    (b'5', [0x27, 0x45, 0x45, 0x45, 0x39]),
    (b'6', [0x3C, 0x4A, 0x49, 0x49, 0x30]),
    (b'7', [0x01, 0x71, 0x09, 0x05, 0x03]),
    (b'8', [0x36, 0x49, 0x49, 0x49, 0x36]),
    (b'9', [0x06, 0x49, 0x49, 0x29, 0x1E]),
    (b'C', [0x3E, 0x41, 0x41, 0x41, 0x22]),
    (b'E', [0x7F, 0x49, 0x49, 0x49, 0x41]),
    (b'H', [0x7F, 0x08, 0x08, 0x08, 0x7F]),
    (b'M', [0x7F, 0x02, 0x0C, 0x02, 0x7F]),
    (b'P', [0x7F, 0x09, 0x09, 0x09, 0x06]),
    (b'R', [0x7F, 0x09, 0x19, 0x29, 0x46]),
    (b'S', [0x46, 0x49, 0x49, 0x49, 0x31]),
    (b'T', [0x01, 0x01, 0x7F, 0x01, 0x01]),
    (b'U', [0x3F, 0x40, 0x40, 0x40, 0x3F]),
    (b'X', [0x63, 0x14, 0x08, 0x14, 0x63]),
    (b'D', [0x7F, 0x41, 0x41, 0x22, 0x1C]),
    (b'A', [0x7E, 0x11, 0x11, 0x11, 0x7E]),
    (b'K', [0x7F, 0x08, 0x14, 0x22, 0x41]),
];

fn glyph(c: u8) -> [u8; 5] {
    for (ch, g) in GLYPHS.iter() {
        if *ch == c {
            return *g;
        }
    }
    [0u8; 5]
}

pub struct Oled<'a> {
    bus: &'a Bus<'a>,
    /// Bytes the panel acknowledged; the driver's own audit trail.
    pub acked: u32,
}

impl<'a> Oled<'a> {
    /// Probe and initialise. `None` when nothing answers at 0x3C.
    pub fn new(bus: &'a Bus<'a>) -> Option<Self> {
        if !bus.probe(ADDR) {
            return None;
        }
        let mut oled = Self { bus, acked: 0 };
        let init: [u8; 24] = [
            0xAE, // display off
            0xD5, 0x80, // clock divider
            0xA8, 0x3F, // multiplex 64
            0xD3, 0x00, // display offset 0
            0x40, // start line 0
            0x8D, 0x14, // charge pump on
            0x20, 0x02, // PAGE addressing (see the module docs)
            0xA1, // segment remap
            0xC8, // COM scan direction
            0xDA, 0x12, // COM pins
            0x81, 0xCF, // contrast
            0xD9, 0xF1, // pre-charge
            0xDB, 0x40, // VCOMH
            0xA4, // display follows RAM
            0xA6, // not inverted
        ];
        let mut frame = [0u8; 25];
        frame[0] = CMD;
        frame[1..].copy_from_slice(&init);
        if oled.send(&frame[..]) {
            oled.acked += 25;
        }
        let on: [u8; 2] = [CMD, 0xAF];
        if oled.send(&on) {
            oled.acked += 2;
        }
        Some(oled)
    }

    fn send(&mut self, bytes: &[u8]) -> bool {
        self.bus.write_cmds(ADDR, bytes)
    }

    /// Draw up to three lines of text, one 8-pixel page each (0, 3 and 6), and
    /// blank every page first so a shorter line cannot leave a tail behind.
    pub fn show_lines(&mut self, lines: [&[u8]; 3]) {
        let mut pages = [[0u8; 128]; 8];
        for (row, line) in lines.iter().enumerate() {
            let page = row * 3; // pages 0, 3, 6 -- one blank page between lines
            if page >= 8 {
                continue;
            }
            let mut x = 0usize;
            for &c in line.iter() {
                for col in glyph(c).iter() {
                    if x < 128 {
                        pages[page][x] = *col;
                        x += 1;
                    }
                }
                x += 1; // one blank column between glyphs
            }
        }
        for (page, pixels) in pages.iter().enumerate() {
            // Page address + column low nibble + column high nibble, then the
            // 128 pixel bytes of that page -- page addressing mode.
            let set_pos: [u8; 4] = [CMD, 0xB0 + page as u8, 0x00, 0x10];
            if self.send(&set_pos) {
                self.acked += 4;
            }
            let mut payload = [0u8; 129];
            payload[0] = DATA;
            payload[1..].copy_from_slice(pixels);
            if self.send(&payload) {
                self.acked += 129;
            }
        }
    }
}

/// Format a value expressed in hundredths with `decimals` digits into `out`,
/// returning the number of bytes written.
pub fn format_fixed(out: &mut [u8], value_times_100: i32, decimals: u8) -> usize {
    let negative = value_times_100 < 0;
    let magnitude = value_times_100.unsigned_abs();
    let mut buf = [0u8; 24];
    let mut n = 0usize;
    if negative {
        buf[0] = b'-';
        n = 1;
    }
    let int_part = magnitude / 100;
    let frac = magnitude % 100;
    let mut digits = [0u8; 8];
    let mut d = 0;
    let mut v = int_part;
    if v == 0 {
        digits[0] = b'0';
        d = 1;
    }
    while v > 0 {
        digits[d] = b'0' + (v % 10) as u8;
        v /= 10;
        d += 1;
    }
    while d > 0 {
        d -= 1;
        buf[n] = digits[d];
        n += 1;
    }
    if decimals > 0 {
        buf[n] = b'.';
        n += 1;
        buf[n] = b'0' + (frac / 10) as u8;
        n += 1;
        if decimals > 1 {
            buf[n] = b'0' + (frac % 10) as u8;
            n += 1;
        }
    }
    let len = n.min(out.len());
    out[..len].copy_from_slice(&buf[..len]);
    len
}

/// Append `src` to `dst` at `pos`, returning the new position.
pub fn append(dst: &mut [u8], pos: usize, src: &[u8]) -> usize {
    let mut p = pos;
    for b in src {
        if p < dst.len() {
            dst[p] = *b;
            p += 1;
        }
    }
    p
}

/// The GPIO clock gate every consumer of this module needs.
pub fn enable_gpio_clock(cmu: &pac::cmu_s::RegisterBlock) {
    cmu.clken0().modify(|_r, w| w.gpio().set_bit());
}
