//! Which group raises which interrupt line?
//!
//! Established so far: the block at `0x60` works, its compare fires exactly one count after the deadline, its flag
//! for channel 0 is bit 1, and its events arrive on **IRQ 68** — the line the RAIL blob owns
//! (`duplicate symbol: SYSRTC_SEQ_IRQHandler`).
//!
//! That leaves the question this probe answers: is there a *second* group whose events arrive on IRQ 67, the line
//! an application may own? The vendor header puts `GRP0_IF` at `0x40` and the blob's block is at `0x60`, so the
//! working theory is `0x40` = group 0 (application) and `0x60` = group 1 (sequencer). Earlier attempts to write
//! `0x40` were called "silently lost", but those attempts also read **bit 0** as the compare flag, and bit 0 is
//! `OVF` — so the failure may have been the same wrong-bit mistake twice.
//!
//! The probe programs group 0 at `0x40`, polls bit 1 of `0x40`, and then reports which of IRQ 67/68 is pending.
//! Neither `IEN` nor the NVIC is touched: a pending bit in `ISPR` appears without any of that, so its presence is
//! purely a fact about the silicon.

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

/// Group 0 as the header lays it out.
const G0_IF: u32 = 0x40;
const G0_IEN: u32 = 0x44;
const G0_CTRL: u32 = 0x48;
const G0_CMP0: u32 = 0x4C;

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
    for shift in [28u32, 24, 20, 16, 12, 8, 4, 0] {
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
    let (pc, cfsr, bfar, s) = unsafe {
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
        core::ptr::write_volatile(slot.add(1), s);
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
        step(1);
        wr(CMU + 0x64, 0x4040_0000);
        wr(CMU + 0x240, 1);

        step(2); // warm reset: stop and disable before touching CFG
        if rd(RTC + 0x04) & 1 != 0 {
            wr(RTC + 0x010, 2);
            let mut g = 0u32;
            while rd(RTC + 0x01C) != 0 {
                if g > 1_000_000 {
                    break;
                }
                g += 1;
            }
            wr(RTC + 0x2004, 1);
            g = 0;
            while rd(RTC + 0x04) & 1 != 0 {
                if g > 1_000_000 {
                    break;
                }
                g += 1;
            }
        }
        wr(RTC + 0x00C, 0);

        step(3); // group 0 at the header's offsets
        wr(RTC + G0_CTRL, 1); // CMP0EN
        wr(RTC + G0_IEN, 0);
        wr(RTC + G0_IF + 0x2000, 0xFFFF_FFFF); // clear through the alias
        let ctrl_rb = rd(RTC + G0_CTRL);

        step(4); // start the counter
        wr(RTC + 0x1004, 1);
        let mut g = 0u32;
        while rd(RTC + 0x04) & 1 == 0 {
            if g > 2_000_000 {
                break;
            }
            g += 1;
        }
        wr(RTC + 0x010, 1);
        cortex_m::asm::dsb();

        step(5); // 300 counts ahead, then watch the flag and the NVIC's pending bits
        let t0 = rd(RTC + 0x018);
        let target = t0.wrapping_add(300);
        wr(RTC + G0_CMP0, target);
        let cmp_rb = rd(RTC + G0_CMP0);
        let mut flag = 0u32;
        let mut t_hit = t0;
        let mut spins = 0u32;
        loop {
            t_hit = rd(RTC + 0x018);
            if rd(RTC + G0_IF) & 0b1111 != 0 {
                flag = rd(RTC + G0_IF);
                break;
            }
            if t_hit.wrapping_sub(t0) > 70_000 {
                flag = rd(RTC + G0_IF);
                break;
            }
            spins = spins.wrapping_add(1);
            if spins > 20_000_000 {
                flag = rd(RTC + G0_IF);
                break;
            }
        }

        step(6); // publish
        let mut m = [0u8; 120];
        let mut n = 0;
        n = label(&mut m, n, b"ctl=");
        n = push_hex(&mut m, n, ctrl_rb);
        n = label(&mut m, n, b" cmp=");
        n = push_hex(&mut m, n, cmp_rb);
        n = label(&mut m, n, b" if=");
        n = push_hex(&mut m, n, flag);
        n = label(&mut m, n, b" d=");
        n = push_dec(&mut m, n, t_hit.wrapping_sub(target));
        n = label(&mut m, n, b" ispr2=");
        n = push_hex(&mut m, n, rd(0xE000_E208));
        n = label(&mut m, n, b" g1if=");
        n = push_hex(&mut m, n, rd(RTC + 0x60));
        mailbox::write(
            if flag != 0 {
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
