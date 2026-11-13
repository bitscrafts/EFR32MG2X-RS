//! What rate does SYSRTC actually count at? Measured in wall-clock time, not against another chip clock.
//!
//! Every previous rate measurement divided by an *assumed* core frequency, and this project already has a
//! known clock discrepancy (`delay_ms(10)` taking 4.35 ms). So this probe removes the assumption entirely: it
//! keeps publishing the counter into a fixed RAM word, and the host reads that word twice with a stopwatch.
//!
//!     rate = (word2 - word1) / (t2 - t1)
//!
//! with no chip clock anywhere in the arithmetic. That is the only kind of measurement that can settle whether
//! the counter runs at 32.768 kHz or at something else.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

const RTC: usize = 0x500A_8000;
const CMU: usize = 0x5000_8000;
/// Published counter: the host reads this word over SWD.
const OUT: usize = 0x2003_F300;

fn rd(addr: usize) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}
fn wr(addr: usize, v: u32) {
    unsafe { core::ptr::write_volatile(addr as *mut u32, v) };
}

fn sync_wait(bound: u32) -> u32 {
    let mut spins = 0u32;
    while rd(RTC + 0x1C) != 0 && spins < bound {
        spins += 1;
    }
    spins
}

#[entry]
fn main() -> ! {
    let _dp = match pac::Peripherals::take() {
        Some(d) => d,
        None => loop {
            core::hint::spin_loop();
        },
    };

    // The order that works: gates, clock select, wait, CFG, group, EN, START, dsb.
    wr(CMU + 0x64, rd(CMU + 0x64) | (1 << 30) | (1 << 22));
    wr(CMU + 0x240, (rd(CMU + 0x240) & !0x3) | 1); // LFRCO
    let _ = sync_wait(50_000);

    wr(RTC + 0x0C, 0); // CFG
    wr(RTC + 0x68, 0x1); // GRP0_CTRL (blob's group block)
    wr(RTC + 0x64, 0); // GRP0_IEN
    wr(RTC + 0x60, 0x7); // GRP0_IF clear

    wr(RTC + 0x1004, 1); // EN_SET
    let _ = sync_wait(50_000);
    wr(RTC + 0x10, 0x1); // CMD = START
    unsafe { core::arch::asm!("dsb 0xf", "isb 0xf") };

    // Publish forever. The host reads OUT twice with a wall clock between reads.
    wr(OUT, rd(RTC + 0x18));
    loop {
        wr(OUT, rd(RTC + 0x18));
    }
}
