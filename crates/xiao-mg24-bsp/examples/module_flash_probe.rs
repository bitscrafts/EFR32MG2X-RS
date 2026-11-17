//! Does this firmware write flash at all? A question NVM3 could not answer, so it is asked one level lower.
//!
//! `nvm3_writeData` returns success, `nvm3_getObjectInfo` confirms an object exists with the right length, and
//! `nvm3_readData` returns 0 — even in the same session. That is not a cache problem and not a page-scan problem; it is
//! the data not being readable after it was "written". This example talks to the flash controller directly, with no
//! NVM3 in the way: erase one page, write a pattern, compare.
//!
//! The page used is the last one of the NVM3 region (0x0817F000). It is inside the region the stack reserves and
//! NVM3 already does not persist anything, so nothing that works today is put at risk.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use xiao_mg24_bsp::prelude::*;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

/// The last flash page, inside the NVM3 region: 2 KB on this part.
const PAGE: u32 = 0x0817_F000;
const PATTERN: [u32; 4] = [0x1234_ABCD, 0xFEED_FACE, 0x00C0_FFEE, 0xDEAD_BEEF];

extern "C" {
    // No `MSC_Init`: the symbol is not in the linked emlib objects in this example (the stack pulls it through NVM3),
    // and the test is worth running without it: a write that fails because the controller was never enabled shows up
    // as a non-zero status, which is itself the answer.
    fn MSC_ErasePage(address: u32) -> u32;
    fn MSC_WriteWord(address: u32, data: *const u8, num_bytes: u32) -> u32;
}

struct Text {
    buf: [u8; 120],
    len: usize,
}

impl Text {
    fn new() -> Self {
        Self {
            buf: [0; 120],
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
    fn u(&mut self, mut v: u32) {
        if v == 0 {
            self.s(b"0");
            return;
        }
        let mut d = [0u8; 10];
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
    fn hex8(&mut self, v: u32) {
        for k in 0..8 {
            let nib = (v >> ((7 - k) * 4)) & 0xF;
            let c = if nib < 10 {
                b'0' + nib as u8
            } else {
                b'a' + (nib - 10) as u8
            };
            self.s(&[c]);
        }
    }
    fn bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

#[entry]
fn main() -> ! {
    // SAFETY: this program's own mailbox slot in RAM.
    unsafe { core::ptr::write_volatile(0x2003_F000 as *mut u32, 0) };

    let _module = match Module::take() {
        Some(m) => m,
        None => loop {
            core::hint::spin_loop();
        },
    };

    // Read before, so "erase did nothing" is distinguishable from "write did nothing".
    let before = unsafe { core::ptr::read_volatile(PAGE as *const u32) };
    let erase = unsafe { MSC_ErasePage(PAGE) };
    let after_erase = unsafe { core::ptr::read_volatile(PAGE as *const u32) };
    let write = unsafe { MSC_WriteWord(PAGE, PATTERN.as_ptr() as *const u8, 16) };

    let mut got = [0u32; 4];
    for (i, slot) in got.iter_mut().enumerate() {
        *slot = unsafe { core::ptr::read_volatile((PAGE as usize + i * 4) as *const u32) };
    }

    let mut t = Text::new();
    t.s(b"flash: before=0x");
    t.hex8(before);
    t.s(b" erase=");
    t.u(erase);
    t.s(b" after=0x");
    t.hex8(after_erase);
    t.s(b" write=");
    t.u(write);
    t.s(b" got=0x");
    t.hex8(got[0]);
    t.s(b"/0x");
    t.hex8(got[3]);
    t.s(b" ok=");
    let ok = got == PATTERN && after_erase == 0xFFFF_FFFF;
    t.u(ok as u32);

    if ok {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
