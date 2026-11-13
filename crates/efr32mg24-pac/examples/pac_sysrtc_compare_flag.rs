//! Does the compare channel exist where the blob says it does, and does its flag ever set?
//!
//! Four questions, one flash, and each answer is a read-back rather than an inference:
//!
//! 1. Does a write to `GRP0_CTRL` at `+0x68` stick? If it reads back zero, that offset is not the control
//!    register and every previous attempt to enable the compare channel went into reserved space.
//! 2. Does a write to `GRP0_CMP0VALUE` at `+0x6C` stick?
//! 3. Does `GRP0_IF` at `+0x60` ever set, with the counter running past the compare value?
//! 4. What does `CNT` say when the flag appears — or never appears?
//!
//! The polling is bounded in wall time by watching `CNT` itself: the counter is measured at 32 968 Hz, so 32 768
//! ticks is one second and the loop stops after three.

#![no_std]
#![no_main]

#[path = "common/mailbox.rs"]
mod mailbox;

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

/// Non-secure SYSRTC, the instance the working probe uses.
const RTC: u32 = 0x500A_8000;
/// CMU, non-secure.
const CMU: u32 = 0x5000_8000;

/// SYSRTC register offsets, as measured from the blob: the group block is at `0x60`, not the header's `0x40`.
const R_CNT: u32 = 0x18;
const R_GRP0_IF: u32 = 0x60;
const R_GRP0_IEN: u32 = 0x64;
const R_GRP0_CTRL: u32 = 0x68;
const R_GRP0_CMP0: u32 = 0x6C;
const R_EN_SET: u32 = 0x1004;
const R_CMD: u32 = 0x010;
const R_STATUS: u32 = 0x014;
const R_SYNCBUSY: u32 = 0x01C;

#[inline(always)]
unsafe fn wr(addr: u32, value: u32) {
    core::ptr::write_volatile(addr as *mut u32, value);
}

#[inline(always)]
unsafe fn rd(addr: u32) -> u32 {
    core::ptr::read_volatile(addr as *const u32)
}

fn push_dec(out: &mut [u8], mut at: usize, value: u32) -> usize {
    let mut digits = [0u8; 10];
    let mut n = 0;
    let mut v = value;
    loop {
        digits[n] = b'0' + (v % 10) as u8;
        v /= 10;
        n += 1;
        if v == 0 || n == digits.len() {
            break;
        }
    }
    for i in (0..n).rev() {
        out[at] = digits[i];
        at += 1;
    }
    at
}

fn push_hex(out: &mut [u8], mut at: usize, value: u32) -> usize {
    let hex = b"0123456789ABCDEF";
    for shift in [28u32, 24, 20, 16, 12, 8, 4, 0] {
        out[at] = hex[((value >> shift) & 0xF) as usize];
        at += 1;
    }
    at
}

#[entry]
fn main() -> ! {
    let _dp = pac::Peripherals::take().expect("peripherals taken twice");

    unsafe {
        // Bus clock, then the clock select, then the config: the order the working probe established.
        wr(CMU + 0x64, 0x4040_0000); // CLKEN0: SYSRTC0 | LFRCO
        wr(CMU + 0x240, 1); // SYSRTC0CLKCTRL: LFRCO
        while rd(RTC + R_SYNCBUSY) != 0 {}
        wr(RTC + 0x0C, 0); // CFG
        wr(RTC + R_EN_SET, 1);
        while rd(RTC + R_STATUS) & 1 == 0 {}
        wr(RTC + R_CMD, 1); // START
        cortex_m::asm::dsb();

        // Clear any flag left over from a previous run, so "IF never set" means this run.
        wr(RTC + 0x2060, 0xFFFF_FFFF); // IF clear alias
        wr(RTC + R_GRP0_IEN, 0xFFFF_FFFF); // enable every interrupt source, for the poll

        // Question 1: does CTRL stick?
        wr(RTC + R_GRP0_CTRL, 1);
        let ctrl_rb = rd(RTC + R_GRP0_CTRL);

        // Question 2: does the compare value stick?
        let start = rd(RTC + R_CNT);
        let deadline = start.wrapping_add(3_000);
        wr(RTC + R_GRP0_CMP0, deadline);
        let cmp_rb = rd(RTC + R_GRP0_CMP0);

        // Question 3: wait for the flag, bounded by three nominal seconds of counting.
        let mut flag_at = 0u32;
        let mut spins = 0u32;
        while spins < 400_000_000 {
            let now = rd(RTC + R_CNT);
            if rd(RTC + R_GRP0_IF) & 1 != 0 {
                flag_at = now;
                break;
            }
            if now.wrapping_sub(deadline) < 0x8000_0000 && now.wrapping_sub(start) > 98_304 {
                break; // three seconds of counting and still no flag
            }
            spins += 1;
        }

        let mut m = [0u8; 120];
        let mut n = 0;
        for b in b"ctrl=" {
            m[n] = *b;
            n += 1;
        }
        n = push_hex(&mut m, n, ctrl_rb);
        for b in b" cmp=" {
            m[n] = *b;
            n += 1;
        }
        n = push_hex(&mut m, n, cmp_rb);
        for b in b" start=" {
            m[n] = *b;
            n += 1;
        }
        n = push_dec(&mut m, n, start);
        for b in b" flag_at=" {
            m[n] = *b;
            n += 1;
        }
        n = push_dec(&mut m, n, flag_at);
        for b in b" if=" {
            m[n] = *b;
            n += 1;
        }
        n = push_hex(&mut m, n, rd(RTC + R_GRP0_IF));
        for b in b" cnt=" {
            m[n] = *b;
            n += 1;
        }
        n = push_dec(&mut m, n, rd(RTC + R_CNT));
        let fired = flag_at != 0;
        mailbox::write(
            if fired {
                mailbox::Status::Pass
            } else {
                mailbox::Status::Fail
            },
            &m[..n],
        );
    }

    loop {
        cortex_m::asm::wfi();
    }
}
