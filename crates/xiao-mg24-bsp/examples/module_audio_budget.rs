//! What does one audio window cost the CPU? Decimation and eight bands, measured.
//!
//! The microphone hands out 303 148 samples per second, measured on this board (`module_mic_rate`). The
//! dosimeter does not want that: it wants 16 kHz windows of 256 samples, which is 62 windows a second, and it
//! wants eight band levels from each. Both steps cost the CPU, and neither cost was known.
//!
//! This example measures them separately, with the cycle counter, so the budget can be built from numbers:
//!
//! - **acquire + decimate**: `D` raw samples are summed into one, `D` times, to fill a 256-sample window.
//!   `D = 19`, because 303 148 / 16 000 = 18.95 — the rate is not exactly 19, and pretending it is costs 0.3 %
//!   of pitch, which matters for octave bands.
//! - **eight bands**: a Goertzel pass per band over the 256-sample window. Goertzel rather than an FFT because
//!   eight bins out of 128 do not justify a full transform, and it needs no twiddle table in RAM.
//!
//! The record carries both cycle counts, the raw sample count and the sum of the band powers, which must be
//! positive — a zero there would mean the window was empty, and the timing would be measuring nothing.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use xiao_mg24_bsp::hal::dwt;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

/// One window of the dosimeter's features: 256 samples at 16 kHz = 16 ms.
const WINDOW: usize = 256;
/// Decimation factor, rounded from 303 148 / 16 000.
const DECIM: usize = 19;

// 2*cos(2*pi*f/16000) for octave centres, computed offline so the target needs no libm.
const G63: f32 = 1.999_387_962_f32;
const G125: f32 = 1.997_590_912_f32;
const G250: f32 = 1.990_369_453_f32;
const G500: f32 = 1.961_570_561_f32;
const G1000: f32 = 1.847_759_065_f32;
const G2000: f32 = 1.414_213_562_f32;
const G4000: f32 = 0.0_f32;
const G6000: f32 = -1.414_213_562_f32;

const COEFFS: [f32; 8] = [G63, G125, G250, G500, G1000, G2000, G4000, G6000];

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

#[entry]
fn main() -> ! {
    // SAFETY: this program's own mailbox slot in RAM.
    unsafe { core::ptr::write_volatile(0x2003_F000 as *mut u32, 0) };

    let mut module = match Module::take() {
        Some(m) => m,
        None => loop {
            core::hint::spin_loop();
        },
    };
    let mic = match module.microphone() {
        Ok(m) => m,
        Err(_) => {
            mailbox::fail(b"budget: converter will not start");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let mut window = [0i32; WINDOW];
    // SAFETY: nothing else in this program touches DWT.
    let t0 = unsafe { dwt::start() }
        .expect("dwt: the cycle counter does not run without a debug session");

    // Acquire and decimate: box-average DECIM raw samples per output sample.
    let mut raw_count: u32 = 0;
    for slot in window.iter_mut() {
        let mut acc = 0i32;
        for _ in 0..DECIM {
            if let Some(v) = mic.sample() {
                acc += v as i32;
            }
            raw_count += 1;
        }
        *slot = acc / DECIM as i32;
    }
    let acquire_cycles = dwt::since(t0);

    // Eight Goertzel passes over the window.
    let t1 = dwt::cycles();
    let mut band_sum = 0u32;
    for coeff in COEFFS {
        let mut s0 = 0f32;
        let mut s1 = 0f32;
        let mut s2 = 0f32;
        for v in window {
            // Centre on mid-scale: the converter's bias is not audio.
            s0 = coeff * s1 - s2 + (v as f32 - 2048.0);
            s2 = s1;
            s1 = s0;
        }
        let power = s1 * s1 + s2 * s2 - coeff * s1 * s2;
        band_sum = band_sum.wrapping_add(power.max(0.0) as u32);
    }
    let band_cycles = dwt::since(t1);

    let mut t = Text::new();
    t.s(b"budget: raw=");
    t.u(raw_count);
    t.s(b" acq_cycles=");
    t.u(acquire_cycles);
    t.s(b" band_cycles=");
    t.u(band_cycles);
    t.s(b" bands_sum=");
    t.u(band_sum);
    t.s(b" w0=");
    t.u(window[0].unsigned_abs());

    // A pass means a window was acquired (raw_count == WINDOW*DECIM) and the bands saw something.
    if raw_count as usize == WINDOW * DECIM && band_sum > 0 {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
