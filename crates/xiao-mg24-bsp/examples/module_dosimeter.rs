//! The application loop, with the features computed *while waiting* for the samples.
//!
//! The first version of this loop captured a whole window and only then computed its eight bands. It took
//! **108,3 % of real time** at 19 MHz — over budget, and the reason was not arithmetic: a paced read spends
//! 1187 cycles per sample waiting for the timer, of which about ten are work. Those 1177 idle cycles per sample
//! are 301 312 free cycles per window, against the 46 044 that the bands need.
//!
//! So the bands run one step per sample, incrementally: every time a sample arrives, all eight Goertzel states
//! advance by that sample, and the powers are finished when the window closes. The arithmetic hides inside the
//! wait, and the loop fits at 19 MHz with no clock change at all.
//!
//! The record says which it did: `pct_of_realtime` must be at or just under 100. Anything above means the
//! arithmetic did not fit inside the waiting and the windows have gaps — which `short=0` cannot see.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use xiao_mg24_bsp::hal::dwt;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

const TARGET_HZ: u32 = 16_000;
const TIMER_CLK_HZ: u32 = 19_000_000;
const WINDOW: usize = 256;
const WINDOWS: usize = 63;

/// 2*cos(2*pi*f/16000) for the octave centres, computed offline so the target needs no libm.
const COEFFS: [f32; 8] = [
    1.999_387_962_f32,
    1.997_590_912_f32,
    1.990_369_453_f32,
    1.961_570_561_f32,
    1.847_759_065_f32,
    1.414_213_562_f32,
    0.0_f32,
    -1.414_213_562_f32,
];

/// Eight Goertzel filters advanced one sample at a time.
///
/// This is the whole point of the example: the same arithmetic as a per-band loop over a stored window, but
/// spread across the waits between samples instead of run in one block afterwards.
struct Bands {
    s1: [f32; 8],
    s2: [f32; 8],
}

impl Bands {
    fn new() -> Self {
        Self {
            s1: [0.0; 8],
            s2: [0.0; 8],
        }
    }

    /// Advance every band by one sample. The converter's bias is removed here, once, for all bands.
    fn step(&mut self, sample: u16) {
        let x = sample as f32 - 2048.0;
        for b in 0..8 {
            let s0 = COEFFS[b] * self.s1[b] - self.s2[b] + x;
            self.s2[b] = self.s1[b];
            self.s1[b] = s0;
        }
    }

    /// Close the window: the power in each band, summed so a caller can prove the window was not empty.
    fn finish(&self) -> u32 {
        let mut sum = 0f32;
        for b in 0..8 {
            let p = self.s1[b] * self.s1[b] + self.s2[b] * self.s2[b]
                - COEFFS[b] * self.s1[b] * self.s2[b];
            if p > 0.0 {
                sum += p;
            }
        }
        sum as u32
    }
}

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
    let mut mic = match module.microphone() {
        Ok(m) => m,
        Err(_) => {
            mailbox::fail(b"interleaved: converter will not start");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    if mic.pace_from_timer(TIMER_CLK_HZ, TARGET_HZ).is_err() {
        mailbox::fail(b"interleaved: converter will not disable for the trigger write");
        loop {
            cortex_m::asm::wfi();
        }
    }
    mic.arm();
    mic.start_timer();

    let mut windows = 0usize;
    let mut missing = 0u32;
    let mut band_sum = 0u32;

    // SAFETY: nothing else in this program touches DWT.
    let t0 = unsafe { dwt::start() }
        .expect("dwt: the cycle counter does not run without a debug session");

    for _ in 0..WINDOWS {
        let mut bands = Bands::new();
        let mut got = 0usize;
        while got < WINDOW {
            match mic.read_paced() {
                Some(v) => {
                    // The arithmetic happens here, one step per sample, inside the wait for the next one.
                    bands.step(v);
                    got += 1;
                }
                None => break,
            }
        }
        if got != WINDOW {
            missing += 1;
            continue;
        }
        band_sum = band_sum.wrapping_add(bands.finish());
        windows += 1;
    }
    let elapsed = dwt::since(t0);
    let timer_running = mic.timer_active();

    // Real time for this many windows, in core cycles at the clock the part boots at.
    let expected = ((windows as u64) * (WINDOW as u64) * 19_000_000u64 / TARGET_HZ as u64) as u32;
    let pct = elapsed.saturating_mul(100) / expected.max(1);

    let mut t = Text::new();
    t.s(b"interleaved: windows=");
    t.u(windows as u32);
    t.s(b" missing=");
    t.u(missing);
    t.s(b" cycles=");
    t.u(elapsed);
    t.s(b" pct=");
    t.u(pct);
    t.s(b" timer=");
    t.u(u32::from(timer_running));
    t.s(b" bands=");
    t.u(band_sum);

    // Fits means at or under real time, with a small allowance for the loop's bookkeeping.
    if windows == WINDOWS && missing == 0 && timer_running && band_sum > 0 && pct <= 102 {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
