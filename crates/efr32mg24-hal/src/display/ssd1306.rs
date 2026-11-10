//! SSD1306 128x64 OLED driver on any [`I2cBus`].
//!
//! The drawing model is a 1 KiB framebuffer (8 pages x 128 columns) flushed in
//! one go, because that is what the controller wants and what makes text
//! updates atomic on the panel — no flicker from clearing then redrawing.
//!
//! ## Three rules this driver encodes, all learned the hard way
//!
//! 1. **One addressing mode only.** Page commands (`0xB0+page`) are ignored
//!    while the controller is in horizontal mode; mixing them paints noise
//!    while every byte is still ACKed.
//! 2. **Control byte and payload in the same transaction.** A lone `0x00`
//!    followed by a second transfer leaves the controller expecting data, and
//!    nothing appears.
//! 3. **Both column nibbles are needed**: `0x00` (low) and `0x10` (high), or the
//!    write starts wherever the previous one ended.
//!
//! The panel cannot be read back over I2C, so this driver cannot verify pixels.
//! What it can do is count the bytes the controller acknowledged, which is what
//! the examples report.

use crate::bus::{BusError, I2cBus};

/// Address the Expansion Base OLED answers on.
pub const DEFAULT_ADDR: u8 = 0x3C;

const CMD: u8 = 0x00;
const DATA: u8 = 0x40;

/// 5x7 glyphs, column-major, LSB = top row.
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
    (b'A', [0x7E, 0x11, 0x11, 0x11, 0x7E]),
    (b'C', [0x3E, 0x41, 0x41, 0x41, 0x22]),
    (b'D', [0x7F, 0x41, 0x41, 0x22, 0x1C]),
    (b'E', [0x7F, 0x49, 0x49, 0x49, 0x41]),
    (b'H', [0x7F, 0x08, 0x08, 0x08, 0x7F]),
    (b'K', [0x7F, 0x08, 0x14, 0x22, 0x41]),
    (b'M', [0x7F, 0x02, 0x0C, 0x02, 0x7F]),
    (b'P', [0x7F, 0x09, 0x09, 0x09, 0x06]),
    (b'R', [0x7F, 0x09, 0x19, 0x29, 0x46]),
    (b'S', [0x46, 0x49, 0x49, 0x49, 0x31]),
    (b'T', [0x01, 0x01, 0x7F, 0x01, 0x01]),
    (b'U', [0x3F, 0x40, 0x40, 0x40, 0x3F]),
    (b'X', [0x63, 0x14, 0x08, 0x14, 0x63]),
];

fn glyph(c: u8) -> [u8; 5] {
    for (ch, g) in GLYPHS.iter() {
        if *ch == c {
            return *g;
        }
    }
    [0u8; 5]
}

/// A 128x64 SSD1306 panel with a 1 KiB framebuffer.
pub struct Ssd1306<B: I2cBus> {
    bus: B,
    addr: u8,
    buffer: [u8; 1024],
    /// Bytes the controller acknowledged, so a caller can report something
    /// factual about a device it cannot read back.
    pub acked: u32,
}

impl<B: I2cBus> Ssd1306<B> {
    /// Probe, then initialise. Returns `None` when nothing answers.
    pub fn new(bus: B) -> Option<Self> {
        Self::with_address(bus, DEFAULT_ADDR)
    }

    /// Same as [`Self::new`] but for a panel strapped to another address.
    pub fn with_address(mut bus: B, addr: u8) -> Option<Self> {
        if !bus.probe(addr) {
            return None;
        }
        let mut oled = Self {
            bus,
            addr,
            buffer: [0u8; 1024],
            acked: 0,
        };
        // PAGE addressing: every later write sets the page explicitly.
        let init: [u8; 24] = [
            0xAE, 0xD5, 0x80, 0xA8, 0x3F, 0xD3, 0x00, 0x40, 0x8D, 0x14, 0x20, 0x02, 0xA1, 0xC8,
            0xDA, 0x12, 0x81, 0xCF, 0xD9, 0xF1, 0xDB, 0x40, 0xA4, 0xA6,
        ];
        let mut frame = [0u8; 25];
        frame[0] = CMD;
        frame[1..].copy_from_slice(&init);
        if oled.send(&frame).is_ok() {
            oled.acked += 25;
        }
        if oled.send(&[CMD, 0xAF]).is_ok() {
            oled.acked += 2;
        }
        Some(oled)
    }

    fn send(&mut self, bytes: &[u8]) -> Result<(), BusError> {
        self.bus.write(self.addr, bytes)
    }

    /// Clear the framebuffer (does not touch the panel until [`flush`]).
    pub fn clear(&mut self) {
        self.buffer = [0u8; 1024];
    }

    /// Draw text at `column`, page `0..8`. Only printable ASCII in the table.
    pub fn text(&mut self, column: usize, page: usize, text: &[u8]) {
        if page >= 8 {
            return;
        }
        let mut x = column;
        for &c in text {
            for col in glyph(c).iter() {
                if x < 128 {
                    self.buffer[page * 128 + x] = *col;
                    x += 1;
                }
            }
            x += 1; // one blank column between glyphs
        }
    }

    /// Push the whole framebuffer to the panel.
    pub fn flush(&mut self) {
        for page in 0..8usize {
            // Page address + column low nibble + column high nibble.
            let set_pos: [u8; 4] = [CMD, 0xB0 + page as u8, 0x00, 0x10];
            if self.send(&set_pos).is_ok() {
                self.acked += 4;
            }
            let mut payload = [0u8; 129];
            payload[0] = DATA;
            payload[1..].copy_from_slice(&self.buffer[page * 128..page * 128 + 128]);
            if self.send(&payload).is_ok() {
                self.acked += 129;
            }
        }
    }

    /// Blank the panel and push the empty frame.
    pub fn clear_display(&mut self) {
        self.clear();
        self.flush();
    }

    /// Give the bus back.
    pub fn free(self) -> B {
        self.bus
    }
}
