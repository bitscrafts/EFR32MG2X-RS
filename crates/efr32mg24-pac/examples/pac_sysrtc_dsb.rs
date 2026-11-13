//! Does the bus stall belong to SYSRTC, or to a pending SYSRTC write?
//!
//! The review's decisive test, and the one thing not yet measured: after `CMD = START`, insert a `dsb` and
//! then read an **unrelated** peripheral. Two different faults hide behind the same symptom:
//!
//!   - if the unrelated read ALSO hangs, an outstanding SYSRTC write is holding the bus, and the fix is about
//!     how the START is issued (ordering, a wait the vendor does not need because its clock is running);
//!   - if the unrelated read returns and only the SYSRTC read hangs, the peripheral itself is waiting on its
//!     LF clock domain, and the fix is in the clock path.
//!
//! Configuration is the one that got furthest before: LFRCO selected, group block at the offsets the blob
//! itself uses (GRP0_IF 0x60, GRP0_IEN 0x64, GRP0_CTRL 0x68).

#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

const RTC: usize = 0x500A_8000;
const CMU: usize = 0x5000_8000;
const CMU_CLKEN0: usize = 0x64;
const CMU_SYSRTC0CLKCTRL: usize = 0x240;
const CMU_IPVERSION: usize = 0x00;

static mut STEP: u32 = 0;
static mut LOG: [u8; 190] = [b' '; 190];
static mut LOG_N: usize = 0;

fn rd(addr: usize) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}
fn wr(addr: usize, v: u32) {
    unsafe { core::ptr::write_volatile(addr as *mut u32, v) };
}

/// Append a fact and republish the whole log: a later hang must not erase the earlier markers.
fn note(text: &str) {
    unsafe {
        for byte in text.as_bytes() {
            if LOG_N < LOG.len() {
                LOG[LOG_N] = *byte;
                LOG_N += 1;
            }
        }
        mailbox::info(&LOG[..LOG_N]);
    }
}

struct Buf {
    b: [u8; 64],
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

/// A fault that reports, in its own slot so the log survives.
#[no_mangle]
pub extern "C" fn HardFault() -> ! {
    let msp: u32;
    unsafe { core::arch::asm!("mrs {}, msp", out(reg) msp) };
    let pc = unsafe { core::ptr::read_volatile((msp as *const u32).add(6)) };
    let cfsr = rd(0xE000_ED28);
    let bfar = rd(0xE000_ED38);

    let mut buf = Buf { b: [0u8; 64], n: 0 };
    let _ = write!(
        &mut buf,
        "FAULT step={} pc={:#010x} cfsr={:#010x} bfar={:#010x}",
        unsafe { STEP },
        pc,
        cfsr,
        bfar
    );
    const MAGIC: u32 = 0x4D42_4F58;
    let base = 0x2003_F200 as *mut u32;
    unsafe {
        core::ptr::write_volatile(base, 0);
        core::ptr::write_volatile(base.add(1), 1);
        core::ptr::write_volatile(base.add(2), 2);
        core::ptr::write_volatile(base.add(3), buf.n as u32);
        let dst = 0x2003_F210 as *mut u8;
        for (i, b) in buf.b[..buf.n].iter().enumerate() {
            core::ptr::write_volatile(dst.add(i), *b);
        }
        core::ptr::write_volatile(base, MAGIC);
    }
    loop {
        core::hint::spin_loop();
    }
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

    // Clocks: both gates, then LFRCO as the counter clock.
    wr(
        CMU + CMU_CLKEN0,
        rd(CMU + CMU_CLKEN0) | (1 << 30) | (1 << 22),
    );
    wr(
        CMU + CMU_SYSRTC0CLKCTRL,
        (rd(CMU + CMU_SYSRTC0CLKCTRL) & !0x3) | 1,
    );
    unsafe { STEP = 1 };
    note("clocks | ");

    // Stop if running, then configure. Disable through EN_CLR (base + 0x2004), which is what the vendor does
    // and what writing 0 to EN_SET does not.
    if rd(RTC + 0x04) & 1 != 0 {
        wr(RTC + 0x10, 0x2); // CMD = STOP
        let _ = sync_wait(50_000);
        wr(RTC + 0x2004, 1); // EN_CLR
        let _ = sync_wait(50_000);
    }
    wr(RTC + 0x0C, 0); // CFG: debug-run off
    wr(RTC + 0x68, 0x1); // GRP0_CTRL: compare 0 enable (blob's group block)
    wr(RTC + 0x64, 0); // GRP0_IEN: none
    wr(RTC + 0x60, 0x7); // GRP0_IF: clear all
    unsafe { STEP = 2 };
    note("configured | ");

    wr(RTC + 0x1004, 1); // EN_SET
    let spins_en = sync_wait(50_000);
    let en = rd(RTC + 0x04);
    unsafe { STEP = 3 };
    {
        let mut b = Buf { b: [0u8; 64], n: 0 };
        let _ = write!(&mut b, "en={:#x} sync={} | ", en, spins_en);
        note(core::str::from_utf8(&b.b[..b.n]).unwrap_or("en ?"));
    }

    // The test: START, then dsb, then an unrelated peripheral.
    wr(RTC + 0x10, 0x1); // CMD = START
    unsafe { core::arch::asm!("dsb 0xf", "isb 0xf") };
    unsafe { STEP = 4 };
    note("start+dsb | ");

    let ipv = rd(CMU + CMU_IPVERSION);
    unsafe { STEP = 5 };
    {
        let mut b = Buf { b: [0u8; 64], n: 0 };
        let _ = write!(&mut b, "cmu={:#x} | ", ipv);
        note(core::str::from_utf8(&b.b[..b.n]).unwrap_or("cmu ?"));
    }

    // Only now the SYSRTC read.
    let sync_spins = sync_wait(200_000);
    unsafe { STEP = 6 };
    {
        let mut b = Buf { b: [0u8; 64], n: 0 };
        let _ = write!(&mut b, "syncbusy spins={} | ", sync_spins);
        note(core::str::from_utf8(&b.b[..b.n]).unwrap_or("sync ?"));
    }

    let a = rd(RTC + 0x18);
    for _ in 0..200_000 {
        core::hint::spin_loop();
    }
    let b = rd(RTC + 0x18);
    unsafe { STEP = 7 };
    {
        let mut out = Buf { b: [0u8; 64], n: 0 };
        let _ = write!(&mut out, "cnt {}->{} delta={}", a, b, b.wrapping_sub(a));
        if b != a {
            mailbox::pass(&out.b[..out.n]);
            note(" COUNTING");
        } else {
            mailbox::fail(&out.b[..out.n]);
        }
    }

    loop {
        core::hint::spin_loop();
    }
}
