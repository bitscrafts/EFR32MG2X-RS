//! Is the microphone's `sample()` really repeating itself? Count, don't argue.
//!
//! A review on 2026-11-17 claimed `Microphone::sample()` is broken: it waits on `SINGLEFIFODV` and then reads
//! `SINGLEDATA`, the *latest* value, without popping the FIFO — so after the first conversion the flag would
//! stay set, the spin would exit immediately, and every call would return the same number at CPU speed.
//!
//! Swapping the read to `SINGLEFIFODATA` (the popping register) was tried on this board and made the level
//! **worse**: `rms=2048 pk=2048`, a dead mid-scale value, where the original path had produced
//! `rms=593 pk=615`. So the argument is not settled by reading register descriptions — it is settled by
//! looking at the numbers the converter produces.
//!
//! This probe answers the question directly: take a window of samples and report **how many are distinct**.
//!
//! - 1 distinct value over 128 samples → the review is right, the data repeats.
//! - Dozens of distinct values around a noise floor → the review's mechanism does not reproduce, whatever the
//!   register descriptions say.
//!
//! It also reports the first four raw samples, so a reader can see the data rather than a summary of it.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

const WINDOW: usize = 128;

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
            mailbox::fail(b"micprobe: converter will not start (clock?)");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let mut samples = [0u16; WINDOW];
    let mut taken = 0usize;
    let mut misses = 0u32;
    while taken < WINDOW {
        match mic.sample() {
            Some(v) => {
                samples[taken] = v;
                taken += 1;
            }
            None => {
                misses += 1;
                if misses > 4 {
                    break;
                }
            }
        }
    }

    // Distinct values, and the extremes: a constant stream shows as 1 and min == max.
    let mut distinct = 0u32;
    let mut min = u16::MAX;
    let mut max = 0u16;
    for i in 0..taken {
        let v = samples[i];
        min = if v < min { v } else { min };
        max = if v > max { v } else { max };
        let mut seen = false;
        for j in 0..i {
            if samples[j] == v {
                seen = true;
                break;
            }
        }
        if !seen {
            distinct += 1;
        }
    }

    let mut t = Text::new();
    t.s(b"micprobe: n=");
    t.u(taken as u32);
    t.s(b" distinct=");
    t.u(distinct);
    t.s(b" min=");
    t.u(min as u32);
    t.s(b" max=");
    t.u(max as u32);
    t.s(b" first=");
    for i in 0..4.min(taken) {
        t.u(samples[i] as u32);
        t.s(b",");
    }
    t.s(b" misses=");
    t.u(misses);

    if taken == 0 {
        mailbox::fail(t.bytes());
    } else if distinct == 1 {
        // The review's claim, reproduced: the converter is handing back one value.
        mailbox::fail(t.bytes());
    } else {
        mailbox::pass(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
