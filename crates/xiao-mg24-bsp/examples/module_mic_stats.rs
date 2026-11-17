//! What is the microphone really producing: a signal about its own mean, or an offset about 2048?
//!
//! `Microphone::level()` centres every sample on the fixed mid-scale value 2048 and takes the absolute deviation
//! (`hal/microphone.rs:377-403`). That is only a measure of the signal if the converter's actual bias is 2048. If it is
//! not, every sample carries the same constant `|bias - 2048|`, and the reported `rms` and `pk` are dominated by the
//! offset rather than by the sound.
//!
//! This program answers that with numbers instead of opinion, over one window of real samples:
//!
//! ```text
//!   mean    the converter's actual bias for this window
//!   min,max the span, which bounds any offset
//!   r2048   RMS of the deviation from 2048   -- what `level()` reports
//!   rmean   RMS of the deviation from `mean` -- the signal, with the bias removed
//! ```
//!
//! If `mean` is far from 2048, `r2048` is mostly the bias and the dosimeter's published `rms` is not a sound level.
//!
//! Run: `cargo build --release --target thumbv8m.main-none-eabihf -p efr32mg2x-xiao-bsp --features sense
//! --example module_mic_stats`, flash, then read the mailbox (`scripts/read-mailbox.sh`).

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use xiao_mg24_bsp::prelude::*;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Samples per window. The dosimeter averages 63 samples per 1 s window (16 000 Hz / 252), so this is one window.
const WINDOW: usize = 63;

struct Text {
    buf: [u8; 200],
    len: usize,
}

impl Text {
    fn new() -> Self {
        Self {
            buf: [0; 200],
            len: 0,
        }
    }
    fn s(&mut self, text: &[u8]) {
        for b in text {
            if self.len < self.buf.len() {
                self.buf[self.len] = *b;
                self.len += 1;
            }
        }
    }
    fn u(&mut self, mut v: u32) {
        if v == 0 {
            self.s(b"0");
            return;
        }
        let mut d = [0u8; 10];
        let mut n = 0;
        while v > 0 {
            d[n] = b'0' + (v % 10) as u8;
            v /= 10;
            n += 1;
        }
        while n > 0 {
            n -= 1;
            let b = d[n];
            self.s(&[b]);
        }
    }
    fn bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

fn isqrt(v: u64) -> u32 {
    if v == 0 {
        return 0;
    }
    let mut x = v;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + v / x) / 2;
    }
    x as u32
}

#[entry]
fn main() -> ! {
    unsafe { core::ptr::write_volatile(0x2003_F000 as *mut u32, 0) };

    let mut module = match Module::take() {
        Some(m) => m,
        None => loop {
            core::hint::spin_loop();
        },
    };

    let mic = match module.microphone() {
        Ok(mic) => mic,
        Err(_) => {
            mailbox::fail(b"micstats: converter will not start (clock?)");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let mut samples = [0u16; WINDOW];
    let mut taken = 0usize;
    while taken < WINDOW {
        match mic.sample() {
            Some(v) => {
                samples[taken] = v;
                taken += 1;
            }
            None => break,
        }
    }

    if taken == 0 {
        mailbox::fail(b"micstats: no samples");
        loop {
            cortex_m::asm::wfi();
        }
    }

    let mut sum: u64 = 0;
    let mut min = u16::MAX;
    let mut max = 0u16;
    for v in samples.iter().take(taken) {
        sum += *v as u64;
        min = if *v < min { *v } else { min };
        max = if *v > max { *v } else { max };
    }
    let mean = (sum / taken as u64) as u32;

    let mut sq_2048: u64 = 0;
    let mut sq_mean: u64 = 0;
    for v in samples.iter().take(taken) {
        let d2048 = (*v as i32 - 2048).unsigned_abs() as u64;
        let dmean = (*v as i32 - mean as i32).unsigned_abs() as u64;
        sq_2048 += d2048 * d2048;
        sq_mean += dmean * dmean;
    }
    let r2048 = isqrt(sq_2048 / taken as u64);
    let rmean = isqrt(sq_mean / taken as u64);

    // How far the converter's own bias sits from the mid-scale value the HAL subtracts. This is the number that says
    // whether `level()` is measuring sound or measuring its own reference.
    let bias = mean.abs_diff(2048);

    let mut t = Text::new();
    t.s(b"micstats: n=");
    t.u(taken as u32);
    t.s(b" min=");
    t.u(min as u32);
    t.s(b" max=");
    t.u(max as u32);
    t.s(b" mean=");
    t.u(mean);
    t.s(b" bias=");
    t.u(bias);
    t.s(b" r2048=");
    t.u(r2048);
    t.s(b" rmean=");
    t.u(rmean);
    t.s(b" ovf=");
    t.u(mic.overflows());
    t.s(b" first=");
    for v in samples.iter().take(4.min(taken)) {
        t.u(*v as u32);
        t.s(b",");
    }

    // A window whose signal is smaller than its own bias is the defect: the level the product publishes would be the
    // reference error, not the sound.
    if rmean < bias {
        mailbox::fail(t.bytes());
    } else {
        mailbox::pass(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
