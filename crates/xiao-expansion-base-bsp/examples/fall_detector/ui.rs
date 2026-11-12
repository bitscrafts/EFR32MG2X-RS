//! What the panel shows — formatting only, no logic.
//!
//! Kept apart so the application can be read without display code in the middle of
//! it, and so a different panel (or none) changes this file alone.

use xiao_expansion_base_bsp::hal::display::{append, Ssd1306};

use crate::app::Status;

/// Append an unsigned decimal number.
pub fn decimal(out: &mut [u8], mut pos: usize, mut value: u32) -> usize {
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
        if pos < out.len() {
            out[pos] = digits[n];
            pos += 1;
        }
    }
    pos
}

/// Redraw the whole screen from the current status.
pub fn render(
    display: &mut Option<Ssd1306<impl xiao_expansion_base_bsp::hal::bus::I2cBus>>,
    status: &Status,
) {
    let Some(d) = display.as_mut() else {
        return;
    };
    d.clear();

    let mut line = [0u8; 24];
    let mut pos = append(&mut line, 0, b"FALLS ");
    pos = decimal(&mut line, pos, status.falls);
    d.text(0, 0, &line[..pos]);

    let mut line = [0u8; 24];
    let mut pos = append(&mut line, 0, b"IMPACTS ");
    pos = decimal(&mut line, pos, status.impacts);
    d.text(0, 1, &line[..pos]);

    d.text(
        0,
        2,
        if status.in_free_fall {
            b"STATE FREE FALL"
        } else {
            b"STATE WATCHING"
        },
    );

    let mut line = [0u8; 24];
    let mut pos = append(&mut line, 0, b"PEAK ");
    pos = decimal(&mut line, pos, status.peak_fall_mg as u32);
    d.text(0, 4, &line[..pos]);

    let mut line = [0u8; 24];
    let mut pos = append(&mut line, 0, b"A ");
    pos = decimal(&mut line, pos, status.magnitude_mg as u32);
    d.text(0, 5, &line[..pos]);

    d.text(
        0,
        7,
        match status.last_event {
            Some(motion_detection::Event::Fall { .. }) => b"LAST FALL",
            Some(motion_detection::Event::Impact { .. }) => b"LAST IMPACT",
            Some(motion_detection::Event::FreeFall { .. }) => b"AIRBORNE",
            None => b"LAST -",
        },
    );

    d.flush();
}
