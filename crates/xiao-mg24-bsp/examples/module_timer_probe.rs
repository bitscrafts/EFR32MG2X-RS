//! One thing only: does starting TIMER0 at 16 kHz work, and does the core survive it?
//!
//! The paced-capture example invalidated its mailbox record and then wrote nothing, which means the core died
//! before it could report. This example does the smallest possible thing — start the timer, read its counter —
//! so a fault here is the timer and a pass here points at the converter's trigger or the sampling loop.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

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

    mailbox::info(b"timer: took the board");

    let period = match module.hal().pace_microphone(16_000) {
        Some(p) => p,
        None => {
            mailbox::fail(b"timer: HFRCOEM23 never reported ready");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    mailbox::info(b"timer: started, waiting for it to run");

    // If the timer counts, its register block is clocked and accessible. Two reads with work in between: a
    // counter that never moves means the clock is not reaching the module, not that the code is wrong.
    let mut first = 0u32;
    let mut last = 0u32;
    for attempt in 0..8 {
        let now = module.hal().timer0_counter();
        if attempt == 0 {
            first = now;
        }
        last = now;
        cortex_m::asm::delay(20_000);
    }

    let mut t = [0u8; 64];
    let mut n = 0usize;
    for b in b"timer: period=" {
        t[n] = *b;
        n += 1;
    }
    let mut v = period;
    let mut d = [0u8; 10];
    let mut dn = 0;
    while v > 0 {
        d[dn] = b'0' + (v % 10) as u8;
        v /= 10;
        dn += 1;
    }
    while dn > 0 {
        dn -= 1;
        let b = d[dn];
        t[n] = b;
        n += 1;
    }
    for b in b" moved=" {
        t[n] = *b;
        n += 1;
    }
    let moved = last != first;
    t[n] = if moved { b'1' } else { b'0' };
    n += 1;

    if moved {
        mailbox::pass(&t[..n]);
    } else {
        mailbox::fail(&t[..n]);
    }

    loop {
        cortex_m::asm::wfi();
    }
}
