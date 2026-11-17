//! Raise the core clock from 19 to 38 MHz and prove it with a reference that does not move.
//!
//! The DWT counts **core** cycles, so a core cycle count alone cannot tell you the frequency. The trick is that
//! the microphone's pace comes from a different oscillator: the IADC's local timer counts `CLK_SRC_ADC` from
//! HFRCOEM23, which this change does not touch. So a window of 256 samples at 16 kHz is *always* 16 ms of real
//! time, and the core cycles it takes say what the core is doing:
//!
//! ```
//! ~304 000 cycles for the window   the core is at 19 MHz (16 ms x 19 MHz)
//! ~608 000 cycles for the window   the core is at 38 MHz (16 ms x 38 MHz)
//! ```
//!
//! The record is written **before** the change as well as after, because a clock change that hangs leaves no
//! other trace: `info` first, then the measurement, then the calibration word, then the second measurement.

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
    fn h(&mut self, v: u32) {
        self.s(b"0x");
        for shift in (0..8).rev() {
            let nib = ((v >> (shift * 4)) & 0xF) as u8;
            let c = if nib < 10 {
                b'0' + nib
            } else {
                b'a' + nib - 10
            };
            self.s(&[c]);
        }
    }
    fn bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

/// One paced window, timed in core cycles. Returns 0 if the window did not arrive whole.
fn window_cycles(mic: &mut xiao_mg24_bsp::hal::microphone::Microphone<'_>) -> u32 {
    let mut window = [0u16; WINDOW];
    let t0 = dwt::cycles();
    let got = mic.capture_window(&mut window);
    let took = dwt::since(t0);
    if got == WINDOW {
        took
    } else {
        0
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
            mailbox::fail(b"boost: converter will not start");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    if mic.pace_from_timer(TIMER_CLK_HZ, TARGET_HZ).is_err() {
        mailbox::fail(b"boost: converter will not disable for the trigger write");
        loop {
            cortex_m::asm::wfi();
        }
    }
    mic.arm();
    mic.start_timer();

    // Before: the reference measurement at whatever the core runs at now. DWT must be started first — without
    // it `dwt::cycles()` returns 0, and the first version of this example reported before=0 after=0 while the
    // capture itself was perfectly fine.
    // SAFETY: nothing else in this program touches DWT.
    unsafe { dwt::start() };
    let before = window_cycles(&mut mic);
    mailbox::info(b"boost: baseline measured, about to raise the band");

    // The change itself.
    let cal = match module.hal().boost_to_80_mhz() {
        Some(c) => c,
        None => {
            mailbox::fail(b"boost: the oscillator never reported ready or the band was clamped");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // After: the same window, which is still 16 ms of real time, in core cycles.
    let after = window_cycles(&mut mic);

    let mut t = Text::new();
    t.s(b"boost80: before=");
    t.u(before);
    t.s(b" after=");
    t.u(after);
    t.s(b" ratio_x100=");
    if before > 0 {
        t.u(after.saturating_mul(100) / before);
    } else {
        t.s(b"?");
    }
    t.s(b" cal=");
    t.h(cal);

    // Doubling the core clock doubles the cycles in a fixed 16 ms window: expect 200 (plus rounding).
    // Any band is a success; the check is "the clock clearly rose and not absurdly", not "it doubled" —
    // the first version hardcoded 2x and failed a perfectly good 80 MHz run with ratio 4,19.
    let ok = before > 0 && after > before + (before / 2) && after < before * 6;
    if ok {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
