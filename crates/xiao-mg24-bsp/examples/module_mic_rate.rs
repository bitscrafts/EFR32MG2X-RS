//! How fast can this board actually sample the microphone? The number the whole dosimeter plan hangs on.
//!
//! The spec's first task, `E1-MIC.1`, exists because every later decision — window length, Goertzel vs FFT,
//! how many bands, whether the model even has usable features — depends on the sustained sample rate. The
//! driver takes one conversion per call (`CMD.SINGLESTART`, wait, read), so the rate is whatever that loop
//! costs, not a datasheet number.
//!
//! This measures it against the core's own cycle counter, and reports the raw counts rather than only a
//! derived frequency, so the result can be recomputed at whatever the core clock turns out to be:
//!
//! - **phase 1**: back-to-back single conversions for a fixed cycle budget, counting successes and misses.
//! - **phase 2**: `level(128)` calls for the same budget — the *useful* rate, since every feature needs whole
//!   windows and the window carries the RMS computation on top of the sampling.
//!
//! Both numbers matter: phase 1 is the converter's ceiling through this driver, phase 2 is what the dosimeter
//! can build features from.
//!
//! The cycle budget is fixed in cycles, so a faster core does the same amount of work in the same number of
//! cycles — the ratio of samples to cycles is what travels, not the time.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use xiao_mg24_bsp::hal::dwt;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

/// One second of the 19 MHz core, in cycles (the core clock measured on this board, 2026-11-17).
const BUDGET_CYCLES: u32 = 19_000_000;

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
    // SAFETY: the mailbox slot is this program's own record, in RAM, written by nothing else.
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
            mailbox::fail(b"rate: converter will not start (clock?)");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // SAFETY: nothing else touches DWT; started once, only read afterwards.
    let start = unsafe { dwt::start() };

    // Phase 1: raw conversions.
    let mut samples: u32 = 0;
    let mut misses: u32 = 0;
    let mut last = 0u32;
    loop {
        match mic.sample() {
            Some(v) => {
                samples = samples.wrapping_add(1);
                last = v as u32;
            }
            None => {
                misses = misses.wrapping_add(1);
                if misses > 32 {
                    break;
                }
            }
        }
        if dwt::since(start) >= BUDGET_CYCLES {
            break;
        }
    }
    let phase1_cycles = dwt::since(start);

    // Phase 2: whole windows, which is what features are built from.
    let t2 = dwt::cycles();
    let mut windows: u32 = 0;
    let mut last_rms = 0u32;
    loop {
        let level = mic.level(128);
        windows = windows.wrapping_add(1);
        last_rms = level.rms;
        if dwt::since(t2) >= BUDGET_CYCLES {
            break;
        }
    }
    let phase2_cycles = dwt::since(t2);

    let mut t = Text::new();
    t.s(b"rate: samples=");
    t.u(samples);
    t.s(b" cycles=");
    t.u(phase1_cycles);
    t.s(b" misses=");
    t.u(misses);
    t.s(b" windows=");
    t.u(windows);
    t.s(b" cycles2=");
    t.u(phase2_cycles);
    t.s(b" last=");
    t.u(last);
    t.s(b" rms=");
    t.u(last_rms);

    // A pass means the converter delivered *something*; the numbers themselves are the deliverable, and a
    // zero would mean the analog path is dead again, which is a failure and not a slow rate.
    if samples > 0 && windows > 0 {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
