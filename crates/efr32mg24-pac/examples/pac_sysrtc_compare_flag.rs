//! One question, one short answer: with the counter provably running, does the compare flag set?
//!
//! Everything this probe publishes fits in a few dozen bytes on purpose. The previous version's message was cut
//! at 64 bytes and the field that mattered -- the counter's value at the end -- was lost, which turned a finished
//! experiment into another round trip.
//!
//! The sequence is the one the HAL uses, including the warm-reset case: SYSRTC sits in the always-on domain and
//! keeps counting across a reset, so it is stopped and disabled before `CFG` is touched. Writing `CFG` while it
//! runs faults at exactly `0x500a800c` -- measured, with BFAR naming the register.

#![no_std]
#![no_main]

#[path = "common/mailbox.rs"]
mod mailbox;

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

const RTC: u32 = 0x500A_8000;
const CMU: u32 = 0x5000_8000;

const STEP_WORD: u32 = 0x2003_F300;
const FAULT_SLOT: u32 = 0x2003_F200;
const FAULT_MAGIC: u32 = 0x464C_5421;

#[inline(always)]
fn step(n: u32) {
    unsafe { core::ptr::write_volatile(STEP_WORD as *mut u32, n) };
}

#[inline(always)]
unsafe fn wr(addr: u32, value: u32) {
    core::ptr::write_volatile(addr as *mut u32, value);
}

#[inline(always)]
unsafe fn rd(addr: u32) -> u32 {
    core::ptr::read_volatile(addr as *const u32)
}

fn push_dec(out: &mut [u8], mut at: usize, mut value: u32) -> usize {
    let mut digits = [0u8; 10];
    let mut n = 0;
    loop {
        digits[n] = b'0' + (value % 10) as u8;
        value /= 10;
        n += 1;
        if value == 0 || n == digits.len() {
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
    for shift in [12u32, 8, 4, 0] {
        out[at] = hex[((value >> shift) & 0xF) as usize];
        at += 1;
    }
    at
}

fn label(out: &mut [u8], mut at: usize, s: &[u8]) -> usize {
    for b in s {
        out[at] = *b;
        at += 1;
    }
    at
}

#[no_mangle]
pub extern "C" fn HardFault() -> ! {
    let msp: u32;
    unsafe { core::arch::asm!("mrs {}, msp", out(reg) msp) };
    let frame = msp as *const u32;
    let (pc, cfsr, bfar, step_now) = unsafe {
        (
            core::ptr::read_volatile(frame.add(6)),
            core::ptr::read_volatile(0xE000_ED28 as *const u32),
            core::ptr::read_volatile(0xE000_ED38 as *const u32),
            core::ptr::read_volatile(STEP_WORD as *const u32),
        )
    };
    unsafe {
        let slot = FAULT_SLOT as *mut u32;
        core::ptr::write_volatile(slot, FAULT_MAGIC);
        core::ptr::write_volatile(slot.add(1), step_now);
        core::ptr::write_volatile(slot.add(2), pc);
        core::ptr::write_volatile(slot.add(3), cfsr);
        core::ptr::write_volatile(slot.add(4), bfar);
    }
    loop {
        core::hint::spin_loop();
    }
}

#[entry]
fn main() -> ! {
    let _dp = pac::Peripherals::take().expect("peripherals taken twice");
    unsafe {
        core::ptr::write_volatile(FAULT_SLOT as *mut u32, 0);
        core::ptr::write_volatile(STEP_WORD as *mut u32, 0);
    }

    unsafe {
        step(1); // bus clocks: SYSRTC0 and LFRCO
        wr(CMU + 0x64, 0x4040_0000);

        step(2); // clock select: LFRCO
        wr(CMU + 0x240, 1);

        step(3); // stop and disable if a warm reset left it running
        let en_at_entry = rd(RTC + 0x04);
        if en_at_entry & 1 != 0 {
            wr(RTC + 0x010, 2); // CMD = STOP
            let mut g = 0u32;
            while rd(RTC + 0x01C) != 0 {
                if g > 1_000_000 {
                    break;
                }
                g += 1;
            }
            wr(RTC + 0x2004, 1); // EN_CLR
            g = 0;
            while rd(RTC + 0x04) & 1 != 0 {
                if g > 1_000_000 {
                    break;
                }
                g += 1;
            }
        }

        step(4); // CFG is only writable while disabled
        wr(RTC + 0x00C, 0);

        step(5); // compare channel 0 on, at the offsets the blob uses
        wr(RTC + 0x068, 1); // GRP0_CTRL.CMP0EN
        wr(RTC + 0x064, 0); // GRP0_IEN, off: the flag is polled, not serviced
        wr(RTC + 0x060, 0xFFFF_FFFF); // GRP0_IF, clear

        step(6); // enable, start
        wr(RTC + 0x1004, 1); // EN_SET
        let mut g = 0u32;
        while rd(RTC + 0x04) & 1 == 0 {
            if g > 2_000_000 {
                break;
            }
            g += 1;
        }
        let en_after = rd(RTC + 0x04);
        wr(RTC + 0x010, 1); // CMD = START
        cortex_m::asm::dsb();

        step(7); // clear the flags, so what appears later is this experiment's
                 // The previous run read GRP0_IF as 0x22 with only CMP0 enabled, so bit 0 is demonstrably not the compare
                 // flag. Clearing and then watching *which* bit appears names it, instead of assuming the header's layout.
        wr(RTC + 0x2060, 0xFFFF_FFFF); // IF, through the CLR alias
        let if_after_clear = rd(RTC + 0x060);

        step(8); // a compare 300 counts ahead, then poll for any bit at all
        let t0 = rd(RTC + 0x018);
        let target = t0.wrapping_add(300);
        wr(RTC + 0x06C, target);
        let mut if_seen = if_after_clear;
        let mut t_hit = t0;
        let mut spins = 0u32;
        loop {
            t_hit = rd(RTC + 0x018);
            let iflag = rd(RTC + 0x060);
            if iflag != if_after_clear {
                if_seen = iflag;
                break;
            }
            if t_hit.wrapping_sub(t0) > 70_000 {
                break;
            }
            spins = spins.wrapping_add(1);
            if spins > 20_000_000 {
                break;
            }
        }

        step(9); // now the same for compare channel 1, to tell bit 1 from bit 5
        let if_cmp0 = if_seen;
        wr(RTC + 0x2060, 0xFFFF_FFFF); // clear
        wr(RTC + 0x068, 3); // GRP0_CTRL: CMP0EN | CMP1EN
        let t1a = rd(RTC + 0x018);
        let target1 = t1a.wrapping_add(300);
        wr(RTC + 0x070, target1); // GRP0_CMP1VALUE, the next word after CMP0
        let mut if_cmp1 = rd(RTC + 0x060);
        let mut t1_hit = t1a;
        let mut spins1 = 0u32;
        loop {
            t1_hit = rd(RTC + 0x018);
            let iflag = rd(RTC + 0x060);
            if iflag != 0 {
                if_cmp1 = iflag;
                break;
            }
            if t1_hit.wrapping_sub(t1a) > 70_000 {
                break;
            }
            spins1 = spins1.wrapping_add(1);
            if spins1 > 20_000_000 {
                break;
            }
        }

        step(10); // publish: short enough that nothing is truncated
                  // "a=<if from CMP0> b=<if from CMP1> d=<cnt at CMP0 hit - target> e=<cnt at CMP1 hit - target1>"
        let mut m = [0u8; 80];
        let mut n = 0;
        n = label(&mut m, n, b"a=");
        n = push_hex(&mut m, n, if_cmp0);
        n = label(&mut m, n, b" b=");
        n = push_hex(&mut m, n, if_cmp1);
        n = label(&mut m, n, b" d=");
        n = push_dec(&mut m, n, t_hit.wrapping_sub(target));
        n = label(&mut m, n, b" e=");
        n = push_dec(&mut m, n, t1_hit.wrapping_sub(target1));
        mailbox::write(
            if if_cmp1 != 0 {
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
