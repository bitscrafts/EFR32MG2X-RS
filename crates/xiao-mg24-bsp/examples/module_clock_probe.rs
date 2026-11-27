//! What is this board's core actually running at? Measured, not assumed.
//!
//! The HAL carries `DEFAULT_SYSCLK_HZ = 19_000_000` with an honest comment — it is a statement about the
//! configuration, not a measurement — and this example measures the core clock so the number stops being a
//! statement. (The old note here said `DELAY-CLOCK-DEFECT.md` derived ~44 MHz from a delay 2.3x short. That is
//! withdrawn: the delay was measured against the crystal on 2026-11-27 and it is right, so the ~44 MHz was another
//! division by an assumed clock -- see `docs/04-reference/DELAY-MEASURED-2026-11-27.md`.)
//!
//! So this measures it across two independent clock domains:
//!
//! - **DWT cycle counter** counts core cycles.
//! - **SYSRTC** counts on LFRCO, 32768 Hz, and is already proven on this board (its counter climbs).
//!
//! Run a fixed number of core cycles, read the SYSRTC counter on both sides, and
//! `core_hz = cycles * 32768 / ticks`. No assumption about either clock survives — only the LFRCO's
//! 32768 Hz, which is the one frequency this part generates from a crystal-accurate source.
//!
//! The record carries the raw numbers as well as the result, so a wrong answer can be re-derived rather
//! than argued about.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use xiao_mg24_bsp::hal::dwt;
use xiao_mg24_bsp::hal::sysrtc::Sysrtc;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Long enough that one LFRCO tick of jitter is a fraction of a percent, short enough to keep the run
/// inside a blink. 19 MHz for 0.5 s is 9.5 M cycles; 44 MHz is 22 M — both fit in the DWT's 32 bits.
const CYCLES_TARGET: u32 = 9_000_000;
const LFRCO_HZ: u64 = 32_768;

struct Text {
    buf: [u8; 128],
    len: usize,
}

impl Text {
    fn new() -> Self {
        Self {
            buf: [0; 128],
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
    fn u(&mut self, mut v: u64) {
        if v == 0 {
            self.s(b"0");
            return;
        }
        let mut d = [0u8; 24];
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
    // Clear the slot on the first instruction, so a crash before writing reads as "no record"
    // rather than as the previous example's record.
    // SAFETY: MAILBOX_ADDR is this program's own record slot, mapped in RAM, and nothing else writes it.
    unsafe { core::ptr::write_volatile(0x2003_F000 as *mut u32, 0) };

    let module = match Module::take() {
        Some(m) => m,
        None => loop {
            core::hint::spin_loop();
        },
    };

    // `Board::sysrtc()` is the seam: it takes the CMU and SYSRTC singletons and configures the
    // low-frequency clock, which is the domain this measurement borrows.
    let sysrtc: Sysrtc = module.hal().sysrtc().expect("SYSRTC is handed out once");

    // SAFETY: nothing else in this program touches DWT; it is taken here for good.
    let start = unsafe { dwt::start() }
        .expect("dwt: the cycle counter does not run without a debug session");
    let ticks_before = sysrtc.now();
    while dwt::since(start) < CYCLES_TARGET {
        core::hint::spin_loop();
    }
    let cycles = dwt::since(start);
    let ticks_after = sysrtc.now();
    let ticks = ticks_after.wrapping_sub(ticks_before);

    let mut t = Text::new();
    t.s(b"clock: cycles=");
    t.u(cycles as u64);
    t.s(b" lfrco_ticks=");
    t.u(ticks as u64);
    if ticks == 0 {
        mailbox::fail(b"clock: no LFRCO tick in the window, cannot divide");
    } else {
        let hz = (cycles as u64 * LFRCO_HZ) / ticks as u64;
        t.s(b" core_hz=");
        t.u(hz);
        t.s(b" nearest_mhz=");
        t.u((hz + 500_000) / 1_000_000);
        mailbox::pass(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
