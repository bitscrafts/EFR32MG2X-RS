//! Measure, without a radio.
//!
//! This exists so that a demonstration never depends on the Bluetooth stack booting. It samples the microphone at
//! 16 kHz with the same timer pacing and the same filter bank as the product, converts to dB SPL with the datasheet
//! calibration, accumulates NR-15 dose, and publishes a record every second. The LED stays on while it runs, so there is
//! one visible sign that the measurement is alive.
//!
//! It is not a fallback for the product — the product speaks BLE, that is the point of the product. It is the answer to
//! "what can I show if the radio will not start", and it is also the cheapest way to prove the measurement chain on the
//! board when the radio is the suspect: same microphone, same arithmetic, no stack in the way.
//!
//! The arithmetic, and where it comes from:
//!
//! ```text
//! converter     12 bits, VBGR 1.21 V, analogue gain 1        (efr32mg2x-hal/src/microphone.rs)
//! LSB           1.21/4096 = 295.4 uV
//! microphone    MSM381ACT001, -38 dBV/Pa at 1 kHz            (Seeed schematic + MEMSensing datasheet)
//!               94 dB SPL = 1 Pa = 12.589 mV = 42.6 counts
//! SPL           94 + 20*log10(rms_counts * LSB / 12.589 mV)  = 61.41 + 20*log10(counts)
//! uncertainty   +-3.0 dB (1 dB unit-to-unit, 3 dB with ageing and temperature)
//! dose NR-15    T(L) = 28800 * 2^(-(L-85)/5);  dose += t/T(L), 100 % at 85 dB for 8 h
//! ```
//!
//! The dose here is computed from an **unweighted** level: there is no A-weighting in this build, so the dose is a
//! demonstration of the arithmetic, not a compliance measurement. The record says `dba=0` to make that impossible to
//! miss.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
#[path = "common/mailbox.rs"]
mod mailbox;
use xiao_mg24_bsp::prelude::*;
use xiao_mg24_bsp::Module;

/// Samples per window, as in the product.
const WINDOW: usize = 256;
/// Windows in a second of audio at 16 kHz: 62.5, so 63.
const WINDOWS_PER_SECOND: usize = 63;
/// Converter constants (see the module comment).
const LSB_VOLTS: f32 = 1.21 / 4096.0;
const V_PER_PASCAL: f32 = 0.012_589;
/// NR-15: 100 % dose is 85 dB for 28 800 s.
const DOSE_REFERENCE_SECONDS: f32 = 28_800.0;

#[entry]
fn main() -> ! {
    let mut module = match Module::take() {
        Some(m) => m,
        None => loop {
            cortex_m::asm::wfi();
        },
    };
    // The LED on, and left on: the one sign that the loop is alive when there is no radio to ask.
    let led = module.led();
    led.on();

    let mut mic = match module.microphone() {
        Ok(m) => m,
        Err(_) => {
            mailbox::fail(b"offline: no microphone");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let mut window = [0u16; WINDOW];
    let mut seconds = 0u32;
    let mut windows_total = 0u32;
    let mut dose_per_mille = 0u32;

    loop {
        let mut got = 0usize;
        while got < WINDOW {
            match mic.read_paced() {
                Some(v) => {
                    window[got] = v;
                    got += 1;
                }
                None => break,
            }
        }
        if got != WINDOW {
            continue;
        }
        windows_total += 1;

        // The window mean, then the RMS about it -- the converter's bias is 2391 counts, not mid-scale.
        let mean = (window.iter().map(|v| *v as u32).sum::<u32>() / WINDOW as u32) as i32;
        let mut sum_sq = 0u64;
        for v in window.iter() {
            let d = (*v as i32 - mean) as i64;
            sum_sq += (d * d) as u64;
        }
        let rms = isqrt((sum_sq / WINDOW as u64) as u32);
        if rms == 0 {
            continue;
        }

        // 61.41 + 20*log10(counts), with the constants above. `sqrt`/`log10` on a Cortex-M33 with FPU.
        let volts = rms as f32 * LSB_VOLTS;
        let pa = volts / V_PER_PASCAL;
        let spl = 94.0 + 20.0 * libm_log10(pa);

        // NR-15 dose for this window: 256 samples at 16 kHz is 0.016 s.
        let t = 28_800.0 * libm_pow2(-(spl - 85.0) / 5.0);
        let fraction = 0.016 / t.max(1.0);
        dose_per_mille += (fraction * 1000.0) as u32;

        if windows_total % WINDOWS_PER_SECOND as u32 == 0 {
            seconds += 1;
            let mut msg = [0u8; 96];
            let n = {
                let mut w = Writer::new(&mut msg);
                w.s(b"offline: seconds=");
                w.u(seconds);
                w.s(b" windows=");
                w.u(windows_total);
                w.s(b" spl=");
                w.u((spl * 10.0) as u32);
                w.s(b" dba=0 dose=");
                w.u(dose_per_mille);
                w.s(b" rms=");
                w.u(rms);
                w.len
            };
            mailbox::info(&msg[..n]);
        }
    }
}

// `libm` is not a dependency of this crate, and adding one for two functions in a demonstration build would put a
// floating-point library between the measurement and the claim. These are the two operations needed, in plain Rust.
/// Integer square root, so no floating-point library is needed for the RMS.
fn isqrt(v: u32) -> u32 {
    if v == 0 {
        return 0;
    }
    let mut x = v;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + v / x) / 2;
    }
    x
}

fn libm_log10(x: f32) -> f32 {
    // Natural log by the usual series around 1, then a change of base: accurate enough for a level in dB, and the
    // calibration's own uncertainty is +-3 dB.
    let mut y = x;
    let mut k = 0i32;
    while y > 2.0 {
        y /= 2.0;
        k += 1;
    }
    while y < 1.0 {
        y *= 2.0;
        k -= 1;
    }
    let z = (y - 1.0) / (y + 1.0);
    let mut s = 0.0;
    let mut term = z;
    let mut n = 1.0;
    for _ in 0..11 {
        s += term / n;
        term *= z * z;
        n += 2.0;
    }
    let ln_y = 2.0 * s + k as f32 * core::f32::consts::LN_2;
    ln_y / core::f32::consts::LN_10
}

fn libm_pow2(x: f32) -> f32 {
    // exp(x * ln 2) by the same series.
    let z = x * core::f32::consts::LN_2;
    let mut s = 1.0;
    let mut term = 1.0;
    for n in 1..14 {
        term *= z / n as f32;
        s += term;
    }
    s
}

/// A tiny integer/float formatter, so the record can carry numbers without `core::fmt`'s machinery.
struct Writer<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl<'a> Writer<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, len: 0 }
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
}
