//! Eight windows back to back: does the capture keep up, or does it lose samples between windows?
//!
//! A paced window is 16 ms of audio. The dosimeter needs to compute features on each one, and that computation
//! happens while the converter keeps converting — nothing buffers audio in hardware between windows. So the
//! question this example answers is the one that decides the architecture: **can the CPU take eight windows in a
//! row without the buffer coming up short?**
//!
//! The record carries, per window, the samples captured and the core cycles it took:
//!
//! ```
//! 256 samples, ~304 000 cycles    paced and continuous: 16 ms of audio at 19 MHz
//! fewer than 256                 the loop fell behind; the count says by how much
//! ~9 000 cycles and 256 samples   not paced at all — the reads are the CPU's rate
//! ```
//!
//! It also carries the sum of the eight window times against the total measured span. If the eight windows are
//! contiguous, the two agree to within a few hundred cycles; if the loop lost time between them, the total is
//! visibly larger than the sum, and that gap is exactly the samples that would have been dropped in a product.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use xiao_mg24_bsp::hal::dwt;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

const TARGET_HZ: u32 = 16_000;
/// The local timer counts CLK_SRC_ADC cycles — measured, not assumed (see module_mic_paced).
const TIMER_CLK_HZ: u32 = 19_000_000;
const WINDOW: usize = 256;
const WINDOWS: usize = 8;

struct Text {
    buf: [u8; 240],
    len: usize,
}

impl Text {
    fn new() -> Self {
        Self {
            buf: [0; 240],
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
            mailbox::fail(b"stream: converter will not start");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    if mic.pace_from_timer(TIMER_CLK_HZ, TARGET_HZ).is_err() {
        mailbox::fail(b"stream: converter will not disable for the trigger write");
        loop {
            cortex_m::asm::wfi();
        }
    }
    mic.arm();
    mic.start_timer();

    let mut window = [0u16; WINDOW];
    let mut shortest = u32::MAX;
    let mut longest = 0u32;
    let mut sum = 0u32;
    let mut short_windows = 0u32;
    let mut first = 0u16;
    let mut last = 0u16;

    // SAFETY: nothing else in this program touches DWT.
    let total_start = unsafe { dwt::start() }
        .expect("dwt: the cycle counter does not run without a debug session");

    for w in 0..WINDOWS {
        let t0 = dwt::cycles();
        let got = mic.capture_window(&mut window);
        let took = dwt::since(t0);
        sum = sum.wrapping_add(took);
        if got != WINDOW {
            short_windows += 1;
        }
        if w == 0 {
            first = window[0];
        }
        last = window[WINDOW - 1];
        if got == WINDOW {
            shortest = shortest.min(took);
            longest = longest.max(took);
        }
    }
    let total = dwt::since(total_start);

    // Read the flag *after* the work: CMD is synchronised with the ADC clock, so STATUS immediately after the
    // command still holds the old value — measured 2026-11-17, a read right after start_timer returned 0 while
    // eight windows of 304 000 cycles each were plainly being paced by that timer.
    let timer_running = mic.timer_active();

    let mut t = Text::new();
    t.s(b"stream: windows=");
    t.u(WINDOWS as u32);
    t.s(b" short=");
    t.u(short_windows);
    t.s(b" min=");
    t.u(if shortest == u32::MAX { 0 } else { shortest });
    t.s(b" max=");
    t.u(longest);
    t.s(b" sum=");
    t.u(sum);
    t.s(b" total=");
    t.u(total);
    t.s(b" timer=");
    t.u(u32::from(timer_running));
    t.s(b" edges=");
    t.u(first as u32);
    t.s(b"/");
    t.u(last as u32);

    // The window is 16 ms = 304 000 cycles at 19 MHz. Eight of them: 2 432 000.
    let per_window = ((WINDOW as u64) * 19_000_000u64 / TARGET_HZ as u64) as u32;
    let slack = per_window / 5;
    let paced =
        shortest != u32::MAX && shortest > per_window - slack && longest < per_window + slack;
    // Contiguity: the eight windows should account for the whole span. Allow 2 % for the loop's own bookkeeping.
    let contiguous = total < sum + (sum / 50);

    if short_windows == 0 && timer_running && paced && contiguous {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
