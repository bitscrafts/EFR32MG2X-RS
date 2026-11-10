//! Display drivers for the Expansion Base.
//!
//! `ssd1306` is the 128x64 OLED that sits on the base board; the driver is
//! generic over the I2C transport so it runs on the bit-banged bus today and on
//! the on-chip I2C0 once that read path works.

pub mod ssd1306;

pub use ssd1306::Ssd1306;

/// Format a value in hundredths with `decimals` digits into `out`, returning the
/// number of bytes written. Kept here (not in an example) because every display
/// example needs it and each copy is a place for a rounding bug to hide.
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
