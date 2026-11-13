//! Does the SYSRTC clock gate live in the secure CMU view, the non-secure one, or both?
//!
//! The previous probe wrote `CLKEN0` through the PAC's secure CMU (`0x40008000`) while using the
//! **non-secure** SYSRTC (`0x500A8000`), and then read the gate back from the **non-secure** CMU
//! (`0x50008000`). If those two CMU views are separate register banks rather than aliases of one, that test
//! proved nothing and the gate for the instance actually in use was never set -- which would explain a
//! precise bus error on the first synchronised write.
//!
//! So: set the bits through both views using their `SET` aliases (what emlib does), read both back, then
//! touch the non-secure SYSRTC.

#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

const CMU_S: usize = 0x4000_8000;
const CMU_NS: usize = 0x5000_8000;
const CLKEN0: usize = 0x64;
const ALIAS_SET: usize = 0x1000;
const RTC_NS: usize = 0x400A_8000; // the SECURE instance, to see whether it behaves differently

static mut STEP: u32 = 0;
static mut LOG: [u8; 200] = [b' '; 200];
static mut LOG_N: usize = 0;

/// Append a fact and republish the whole log, so the last record the mailbox holds is the complete story.
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

/// A second mailbox slot, so a fault does not erase the markers that say how far the run got.
fn write_slot(addr: usize, msg: &[u8]) {
    const MAGIC: u32 = 0x4D42_4F58;
    let base = addr as *mut u32;
    let len = core::cmp::min(msg.len(), 200);
    unsafe {
        core::ptr::write_volatile(base, 0);
        core::ptr::write_volatile(base.add(1), 1);
        core::ptr::write_volatile(base.add(2), 2);
        core::ptr::write_volatile(base.add(3), len as u32);
        let dst = (addr + 16) as *mut u8;
        for (i, b) in msg.iter().take(len).enumerate() {
            core::ptr::write_volatile(dst.add(i), *b);
        }
        core::ptr::write_volatile(base, MAGIC);
    }
}

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

/// A fault that reports, in a slot of its own so the step markers survive.
#[no_mangle]
pub extern "C" fn HardFault() -> ! {
    let msp: u32;
    unsafe { core::arch::asm!("mrs {}, msp", out(reg) msp) };
    let frame = msp as *const u32;
    let pc = unsafe { core::ptr::read_volatile(frame.add(6)) };
    let cfsr = rd(0xE000_ED28);
    let bfar = rd(0xE000_ED38);

    let mut buf = Buf {
        b: [0u8; 200],
        n: 0,
    };
    let _ = write!(
        &mut buf,
        "FAULT step={} pc={:#010x} cfsr={:#010x} bfar={:#010x}",
        unsafe { STEP },
        pc,
        cfsr,
        bfar
    );
    write_slot(0x2003_F200, &buf.b[..buf.n]);
    loop {
        core::hint::spin_loop();
    }
}

#[entry]
fn main() -> ! {
    let _dp = match pac::Peripherals::take() {
        Some(d) => d,
        None => loop {
            core::hint::spin_loop();
        },
    };

    // 1. The gates, through both views, using the SET alias emlib uses.
    let bits = (1u32 << 30) | (1u32 << 22); // SYSRTC0 | LFRCO
    wr(CMU_S + ALIAS_SET + CLKEN0, bits);
    wr(CMU_NS + ALIAS_SET + CLKEN0, bits);
    unsafe { STEP = 1 };

    // The LFRCO: does it run on demand, or must it be forced? Its own STATUS says which. Offset 0x04 is
    // CTRL (FORCEEN bit 0), 0x08 is STATUS (RDY bit 0, ENS bit 16), measured with the vendor headers.
    let lfrco = 0x5002_4000;
    let before = rd(lfrco + 0x08);
    wr(lfrco + 0x04, rd(lfrco + 0x04) | 1); // FORCEEN
    let mut rdy_spins = 0u32;
    while rd(lfrco + 0x08) & 1 == 0 && rdy_spins < 200_000 {
        rdy_spins += 1;
    }
    let after = rd(lfrco + 0x08);
    let mut lb = Buf {
        b: [0u8; 200],
        n: 0,
    };
    let _ = write!(
        &mut lb,
        "lfrco before={:#x} after={:#x} spins={} | ",
        before, after, rdy_spins
    );
    note(core::str::from_utf8(&lb.b[..lb.n]).unwrap_or("lfrco ?"));

    let s = rd(CMU_S + CLKEN0);
    let ns = rd(CMU_NS + CLKEN0);
    let mut d = Buf {
        b: [0u8; 200],
        n: 0,
    };
    let _ = write!(
        &mut d,
        "gates S={:#x} NS={:#x} same={} s30={} ns30={} | ",
        s,
        ns,
        s == ns,
        (s >> 30) & 1,
        (ns >> 30) & 1
    );
    note(core::str::from_utf8(&d.b[..d.n]).unwrap_or("gates ?"));

    // 2. Select LFRCO on the non-secure SYSRTC clock control, and report what it holds.
    unsafe { STEP = 2 };
    let clkctrl = CMU_NS + 0x240; // SYSRTC0CLKCTRL, offset measured with the vendor headers (the PAC says 0x1DC and is wrong)
    wr(clkctrl, (rd(clkctrl) & !0x3) | 3); // CLKSEL=ULFRCO: always on
    let back = rd(clkctrl);
    let mut d = Buf {
        b: [0u8; 200],
        n: 0,
    };
    let _ = write!(&mut d, "clkctrl={:#x} clksel={} | ", back, back & 3);
    note(core::str::from_utf8(&d.b[..d.n]).unwrap_or("clkctrl ?"));

    // 3. Configured, then the write that has been faulting.
    unsafe { STEP = 3 };
    wr(RTC_NS + 0x0C, 0);
    // The group block at the offsets the RAIL blob itself uses (SYSRTC_SEQ_IRQHandler reads GRP0_IF at +0x60
    // and GRP0_IEN at +0x64), not the 0x40..0x4C the headers describe.
    wr(RTC_NS + 0x68, 0x1); // GRP0_CTRL: compare 0 enable
    wr(RTC_NS + 0x64, 0); // GRP0_IEN: none yet
    wr(RTC_NS + 0x60, 0x7); // GRP0_IF: clear all
    note("configured | ");

    unsafe { STEP = 4 };
    wr(RTC_NS + 0x1004, 1); // EN_SET: the alias page mirrors the layout, so EN is at 0x1004, not 0x1000
    let en = rd(RTC_NS + 0x04);
    let mut d = Buf {
        b: [0u8; 200],
        n: 0,
    };
    let _ = write!(&mut d, "en={:#x} enabled={}", en, en & 1);
    note(core::str::from_utf8(&d.b[..d.n]).unwrap_or("en ?"));

    unsafe { STEP = 5 };
    wr(RTC_NS + 0x10, 0x1); // CMD = START
    note("cmd=start accepted");

    unsafe { STEP = 6 };

    // Does SYNCBUSY ever clear? A bounded count, and the number is the answer: 0 means synchronised, the
    // bound means the clock is not running and no write will ever complete.
    let mut spins = 0u32;
    while rd(RTC_NS + 0x1C) != 0 && spins < 200_000 {
        spins += 1;
    }
    let mut d = Buf {
        b: [0u8; 200],
        n: 0,
    };
    let _ = write!(&mut d, "syncspins={} | ", spins);
    note(core::str::from_utf8(&d.b[..d.n]).unwrap_or("?"));

    unsafe { STEP = 7 };
    let a = rd(RTC_NS + 0x18);
    let mut d = Buf {
        b: [0u8; 200],
        n: 0,
    };
    let _ = write!(&mut d, "a={} | ", a);
    note(core::str::from_utf8(&d.b[..d.n]).unwrap_or("?"));

    unsafe { STEP = 8 };
    for _ in 0..200_000 {
        core::hint::spin_loop();
    }
    let b = rd(RTC_NS + 0x18);
    let status = rd(RTC_NS + 0x14);
    let mut d = Buf {
        b: [0u8; 200],
        n: 0,
    };
    let _ = write!(
        &mut d,
        "b={} delta={} status={:#x}",
        b,
        b.wrapping_sub(a),
        status
    );
    note(core::str::from_utf8(&d.b[..d.n]).unwrap_or("?"));

    loop {
        core::hint::spin_loop();
    }
}
