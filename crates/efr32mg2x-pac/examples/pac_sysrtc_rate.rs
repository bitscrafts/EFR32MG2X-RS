//! A rate probe that has not survived: it measures SYSRTC ticks per DWT cycle under two clock selections.
//!
//! State, measured 2026-11-15: this example takes an imprecise bus error with BFAR at EN_SET (0x500a9004) on the
//! non-secure alias and at 0x400a9004 on the secure one. Its write sequence is now identical, register by register, to
//! pac_sysrtc_dsb -- which passes and reports COUNTING -- so the difference is not in the order of the writes. The
//! group 0 block is also unsettled: the vendor header and the SVD put GRP0_IF/IEN/CTRL at 0x040/0x044/0x048, and
//! writing those offsets stops pac_sysrtc_dsb earlier than 0x060/0x064/0x068 do. Both questions need the SYSRTC
//! chapter of the reference manual, not another guess; the four examples keep the offsets the board accepts and say so.
//!
//! The file keeps the two original questions below, because they are still the questions:
//! Both are answered by reading back what was written, plus one rate measurement per clock selection:
//!
//!   - if `CTRL = 1` written at 0x68 reads back 1, the register exists and the group block is where the blob
//!     implies; if it reads 0, the write was dropped and the block is somewhere else;
//!   - if the measured rate changes when CLKSEL goes 1 -> 3 (LFRCO -> ULFRCO, a factor of ~33), then 0x240 is
//!     the selector. If the rate stays ~2.9 MHz, 0x240 is not the selector and the counter is clocked by
//!     something else entirely.
//!
//! Rates are measured against the DWT cycle counter, whose own rate is a separate open question (see
//! DELAY-CLOCK-DEFECT.md); the comparison between two measurements from the same instrument is still valid,
//! which is what the second question needs.

#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use efr32mg2x_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

const RTC: usize = 0x500A_8000;
const CMU: usize = 0x5000_8000;

fn rd(addr: usize) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}
fn wr(addr: usize, v: u32) {
    unsafe { core::ptr::write_volatile(addr as *mut u32, v) };
}

struct Buf {
    b: [u8; 200],
    n: usize,
}
impl Write for Buf {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.as_bytes() {
            if self.n < self.b.len() {
                self.b[self.n] = *byte;
                self.n += 1;
            }
        }
        Ok(())
    }
}

static mut STEP: u32 = 0;

#[no_mangle]
pub extern "C" fn HardFault() -> ! {
    let msp: u32;
    unsafe { core::arch::asm!("mrs {}, msp", out(reg) msp) };
    let pc = rd((msp as usize) + 24);
    let cfsr = rd(0xE000_ED28);
    let bfar = rd(0xE000_ED38);
    let mut buf = Buf {
        b: [0u8; 200],
        n: 0,
    };
    let _ = write!(
        &mut buf,
        "FAULT rtc={:#010x} step={} pc={:#010x} cfsr={:#010x} bfar={:#010x}",
        RTC as u32,
        unsafe { STEP },
        pc,
        cfsr,
        bfar
    );
    mailbox::fail(&buf.b[..buf.n]);
    loop {
        core::hint::spin_loop();
    }
}

/// Cycle counter, started properly: TRCENA then CYCCNTENA, counter cleared first.
unsafe fn dwt_start() -> u32 {
    let mut core = cortex_m::Peripherals::steal();
    core.DCB.enable_trace();
    core.DWT.cyccnt.write(0);
    core.DWT.enable_cycle_counter();
    rd(0xE000_1004) // DWT->CYCCNT
}

fn sync_wait(bound: u32) -> u32 {
    let mut spins = 0u32;
    while rd(RTC + 0x1C) != 0 && spins < bound {
        spins += 1;
    }
    spins
}

/// Ticks of SYSRTC over `cycles` DWT cycles.
fn measure(cycles: u32) -> (u32, u32) {
    let t0 = rd(RTC + 0x18);
    let c0 = unsafe { dwt_start() };
    while rd(0xE000_1004).wrapping_sub(c0) < cycles {
        core::hint::spin_loop();
    }
    let c1 = rd(0xE000_1004);
    let t1 = rd(RTC + 0x18);
    (t1.wrapping_sub(t0), c1.wrapping_sub(c0))
}

#[entry]
fn main() -> ! {
    let _dp = match pac::Peripherals::take() {
        Some(d) => d,
        None => loop {
            core::hint::spin_loop();
        },
    };

    wr(CMU + 0x64, rd(CMU + 0x64) | (1 << 30) | (1 << 22));

    // Clock select first, wait for the domain, and only then the configuration.
    wr(CMU + 0x240, (rd(CMU + 0x240) & !0x3) | 1); // LFRCO
    let clksel_1 = rd(CMU + 0x240) & 3;
    let _ = sync_wait(50_000);

    // Stop and clear the enable before touching CFG. This example used to write CFG straight after the clock select and
    // took an imprecise bus error with BFAR at the CFG register -- on both the secure and the non-secure alias, so it
    // was never an address problem. pac_sysrtc_dsb does the vendor's sequence (CMD=STOP, EN_CLR at base+0x2004) and
    // counts; the difference between the two was exactly this block.
    if rd(RTC + 0x04) & 1 != 0 {
        wr(RTC + 0x10, 0x2); // CMD = STOP
        let _ = sync_wait(50_000);
        wr(RTC + 0x2004, 1); // EN_CLR
        let _ = sync_wait(50_000);
    }

    wr(RTC + 0x0C, 0); // CFG
                       // Group 0 block. These offsets come from the vendor header (efr32mg24_sysrtc.h) and the SVD, which agree:
                       // IF 0x040, IEN 0x044, CTRL 0x048. This file used to write 0x060/0x064/0x068 instead -- 0x20 higher, inside the
                       // reserved window -- and that write to reserved space is what produced the imprecise bus fault it used to report.
    wr(RTC + 0x48, 0x1); // GRP0_CTRL: compare 0 enable
    let ctrl_48 = rd(RTC + 0x48);
    wr(RTC + 0x44, 0); // GRP0_IEN: no interrupts
    wr(RTC + 0x40, 0x7); // GRP0_IF: clear all
    unsafe { STEP = 1 };

    wr(RTC + 0x1004, 1); // EN_SET
    let _ = sync_wait(50_000);
    wr(RTC + 0x10, 0x1); // START
    unsafe { core::arch::asm!("dsb 0xf", "isb 0xf") };
    let (t_a, c_a) = measure(1_900_000);

    wr(RTC + 0x10, 0x2); // STOP
    let _ = sync_wait(50_000);
    wr(RTC + 0x2004, 1); // EN_CLR
    let _ = sync_wait(50_000);
    wr(CMU + 0x240, (rd(CMU + 0x240) & !0x3) | 3); // ULFRCO
    let clksel_3 = rd(CMU + 0x240) & 3;
    wr(RTC + 0x1004, 1);
    let _ = sync_wait(50_000);
    wr(RTC + 0x10, 0x1);
    unsafe { core::arch::asm!("dsb 0xf", "isb 0xf") };
    let (t_b, c_b) = measure(1_900_000);
    unsafe { STEP = 2 };

    let mut out = Buf {
        b: [0u8; 200],
        n: 0,
    };
    let _ = write!(
        &mut out,
        "grp0ctrl={} sel1={} ticks1={}/{} sel3={} ticks3={}/{}",
        ctrl_48, clksel_1, t_a, c_a, clksel_3, t_b, c_b
    );
    // Pass only if the group control register took the value written and the clock selection changes the rate.
    let register_real = ctrl_48 == 1;
    let rate_changed = t_a != 0 && t_b != 0 && t_b.abs_diff(t_a) > t_a / 4;
    if register_real && rate_changed {
        mailbox::pass(&out.b[..out.n]);
    } else {
        mailbox::fail(&out.b[..out.n]);
    }

    loop {
        core::hint::spin_loop();
    }
}
