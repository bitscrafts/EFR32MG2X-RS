//! Paced sampling at 16 kHz: the order the vendor driver implies, measured.
//!
//! The IADC's local timer is a down-counter in ADC_CLK cycles (`em_iadc.h`: `timerCycles`, "Number of ADC_CLK
//! cycles per TIMER event"). This example configures it, arms the queue, starts the timer **last**
//! (`iadcCmdEnableTimer` is an application step, not part of `IADC_Init`), and then reads one conversion per
//! period without starting any of its own.
//!
//! The record carries what tells the three possible failures apart:
//!
//! ```
//! timer=0                TIMERACTIVE low: the timer never ran, so nothing else in the record means anything
//! n=0 timer=1            the timer ran and the queue never produced: the trigger is not reaching it
//! n=256 spent≈304000     paced: 256 samples at 16 kHz are 16 ms, which is 304 000 cycles at 19 MHz
//! n=256 spent≈9000       free-running: the reads are the CPU's rate, not the timer's
//! ```

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use xiao_mg24_bsp::hal::dwt;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

const TARGET_HZ: u32 = 16_000;
/// The timer's own clock, and it was measured rather than assumed: a period of 593 produced 593 core cycles
/// per sample, i.e. 32 kHz, which is `19 MHz / 593`. So the local timer counts CLK_SRC_ADC (19 MHz from
/// HFRCOEM23) and not ADC_CLK after `SCHED0.PRESCALE` — the opposite of what `timerCycles`' doc comment says.
const TIMER_CLK_HZ: u32 = 19_000_000;
const WINDOW: usize = 256;

struct Text {
    buf: [u8; 220],
    len: usize,
}

impl Text {
    fn new() -> Self {
        Self {
            buf: [0; 220],
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
            mailbox::fail(b"paced: converter will not start");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let cycles = match mic.pace_from_timer(TIMER_CLK_HZ, TARGET_HZ) {
        Ok(c) => c,
        Err(_) => {
            mailbox::fail(b"paced: converter will not disable for the trigger write");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // The vendor's order: arm the queue, then enable the timer.
    mic.arm();
    mic.start_timer();
    let timer_running = mic.timer_active();

    let mut window = [0i16; WINDOW];
    // SAFETY: nothing else in this program touches DWT.
    let t0 = unsafe { dwt::start() };
    let mut n = 0usize;
    let mut misses = 0u32;
    while n < WINDOW {
        match mic.read_paced() {
            Some(v) => {
                window[n] = v as i16;
                n += 1;
            }
            None => {
                misses += 1;
                if misses > 20 {
                    break;
                }
            }
        }
    }
    let cycles_spent = dwt::since(t0);

    let mut distinct = 0usize;
    let mut seen = [0i16; 64];
    for v in window {
        if distinct < seen.len() && !seen[..distinct].contains(&v) {
            seen[distinct] = v;
            distinct += 1;
        }
    }

    let mut t = Text::new();
    t.s(b"paced: hz=");
    t.u(TARGET_HZ);
    t.s(b" cycles_set=");
    t.u(cycles);
    t.s(b" n=");
    t.u(n as u32);
    t.s(b" spent=");
    t.u(cycles_spent);
    t.s(b" distinct=");
    t.u(distinct as u32);
    t.s(b" timer=");
    t.u(u32::from(timer_running));

    // Paced means the window took roughly 256/16000 s of the 19 MHz core: 4 864 000 cycles. A fifth either way
    // covers the timer's rounding; a window that took a few thousand is the free-running rate, not the timer's.
    // Two-sided on purpose: a lower bound alone accepts a window that took ten times too long, which is not
    // pacing either. 256 samples at 16 kHz are 16 ms, so 304 000 cycles at 19 MHz; ±20 % covers the timer's
    // rounding and the loop's own overhead.
    let expected = ((WINDOW as u64) * 19_000_000u64 / TARGET_HZ as u64) as u32;
    let slack = expected / 5;
    let paced = cycles_spent > expected - slack && cycles_spent < expected + slack;

    if n == WINDOW && distinct > 1 && timer_running && paced {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
