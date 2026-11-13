//! SYSRTC through raw addresses, with a fault handler that reports instead of hanging.
//!
//! The HAL version faults with BFAR=1 and says nothing: the core lands in the default handler, which is an
//! infinite loop, and the only evidence is a stale mailbox. Reading the exception frame from outside needs
//! the debugger to guess which frame is current, so the handler does it from inside where the frame is
//! unambiguous: it reads the stacked PC, CFSR and BFAR and publishes them.
//!
//! Offsets from `efr32mg24_sysrtc.h`: IPVERSION 0x00, EN 0x04, SWRST 0x08, CFG 0x0C, CMD 0x10,
//! STATUS 0x14, CNT 0x18, SYNCBUSY 0x1C, GRP0_IF 0x40, GRP0_IEN 0x44, GRP0_CTRL 0x48,
//! GRP0_CMP0VALUE 0x4C. The SET alias page is +0x1000.

#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

static mut STEP: u32 = 0;

/// Write a record to an arbitrary mailbox slot, same layout as `common/mailbox.rs`.
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

const RTC: usize = 0x500A_8000; // non-secure alias: the secure one faults at CMD
const ALIAS_SET: usize = 0x1000;

fn rd(off: usize) -> u32 {
    unsafe { core::ptr::read_volatile((RTC + off) as *const u32) }
}
fn wr(off: usize, v: u32) {
    unsafe { core::ptr::write_volatile((RTC + off) as *mut u32, v) }
}

/// A minimal writer for the mailbox buffer.
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

/// The fault handler this project should have had from the start: a fault that reports is a diagnosis, a
/// fault that loops is a lost evening.
#[no_mangle]
pub extern "C" fn HardFault() -> ! {
    // Read the exception frame: the hardware stacked the interrupted context on MSP.
    let msp: u32;
    unsafe { core::arch::asm!("mrs {}, msp", out(reg) msp) };
    let frame = msp as *const u32;
    let (pc, lr) = unsafe {
        (
            core::ptr::read_volatile(frame.add(6)),
            core::ptr::read_volatile(frame.add(5)),
        )
    };
    let cfsr = unsafe { core::ptr::read_volatile(0xE000_ED28 as *const u32) };
    let bfar = unsafe { core::ptr::read_volatile(0xE000_ED38 as *const u32) };

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
    // A second slot, so the fault does not overwrite the step markers: the last marker and the fault
    // together are the diagnosis, and one without the other is a guess.
    write_slot(0x2003_F200, &buf.b[..buf.n]);

    loop {
        core::hint::spin_loop();
    }
}

#[entry]
fn main() -> ! {
    let dp = match pac::Peripherals::take() {
        Some(d) => d,
        None => loop {
            core::hint::spin_loop();
        },
    };
    unsafe { STEP = 1 };
    mailbox::info(b"t0: pac taken");

    // Both bus clocks: the LFRCO's own gate (bit 22) and SYSRTC's (bit 30). Without the LFRCO gate the
    // counter's clock domain never runs, and the first write to a synchronised register (CMD) comes back as
    // a precise bus error at that register's address -- which is exactly what BFAR said.
    dp.cmu_s
        .clken0()
        .modify(|_, w| w.lfrco().set_bit().sysrtc0().set_bit());
    dp.cmu_s
        .sysrtc0clkctrl()
        .modify(|r, w| unsafe { w.bits((r.bits() & !0x3) | 1) });
    // Read the gates back. A clock-enable bit that did not take is the difference between "the peripheral
    // is deaf" and "the peripheral faults", and only a read-back distinguishes them.
    let clken0 = unsafe { core::ptr::read_volatile(0x5000_8064 as *const u32) };
    let clkctrl = unsafe { core::ptr::read_volatile(0x5000_81DC as *const u32) };
    let mut dbg = Buf {
        b: [0u8; 200],
        n: 0,
    };
    let _ = write!(
        &mut dbg,
        "clken0={:#010x} sysrtc0bit={} lfrcobit={} clkctrl={:#010x} clksel={}",
        clken0,
        (clken0 >> 30) & 1,
        (clken0 >> 22) & 1,
        clkctrl,
        clkctrl & 3
    );
    unsafe { STEP = 3 };
    mailbox::info(&dbg.b[..dbg.n]);

    let mut spins = 0u32;
    while rd(0x1C) != 0 && spins < 100_000 {
        spins += 1;
    }
    wr(0x0C, 0); // CFG
    wr(0x48, 0x1); // GRP0_CTRL
    wr(0x44, 0); // GRP0_IEN
    wr(0x40, 0x7); // GRP0_IF clear
    unsafe { STEP = 4 };
    mailbox::info(b"t2: configured");

    wr(ALIAS_SET, 1); // EN_SET
    unsafe { STEP = 5 };
    mailbox::info(b"t3: enabled");

    // The suspect: any command write at all, or only START?
    wr(0x10, 0x0);
    unsafe { STEP = 6 };
    mailbox::info(b"t4: cmd=0 ok");
    wr(0x10, 0x2);
    unsafe { STEP = 7 };
    mailbox::info(b"t5: cmd=stop ok");

    wr(0x10, 0x1);
    unsafe { STEP = 8 };
    mailbox::info(b"t6: cmd=start ok");

    unsafe { NVIC::unmask(pac::Interrupt::SYSRTC_APP) };

    let a = rd(0x18);
    for _ in 0..300_000 {
        core::hint::spin_loop();
    }
    let b = rd(0x18);

    let mut buf = Buf {
        b: [0u8; 200],
        n: 0,
    };
    let _ = write!(&mut buf, "count a={} b={} running={}", a, b, rd(0x14) & 1);
    if b != a {
        mailbox::pass(&buf.b[..buf.n]);
    } else {
        // A second slot, so the fault does not overwrite the step markers: the last marker and the fault
        // together are the diagnosis, and one without the other is a guess.
        write_slot(0x2003_F200, &buf.b[..buf.n]);
    }

    loop {
        core::hint::spin_loop();
    }
}
