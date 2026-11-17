//! Does NVM3 actually keep what it is given? Written to answer one question, with no phone involved.
//!
//! With the bonding database enabled, the peer aborts the pairing (0x1208) and with it disabled the same pairing
//! completes. That isolates the defect to persisting keys, and this example asks the narrowest version of it:
//!
//! ```
//! run 1   the object is absent -> the example writes it and says so
//! reset   the board restarts WITHOUT reflashing
//! run 2   the object is read back; the value proves the write survived the reset
//! ```
//!
//! The handle is the stack's own: `Bluetooth::new` calls `nvm3_open` during start-up, so this uses the very instance
//! the bonding code uses rather than opening a second one beside it.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_radio::Bluetooth;
use panic_halt as _;
use xiao_mg24_bsp::prelude::*;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Survives a reset because `.uninit` is not zeroed by the runtime. It is the control: without it, "read=0 after a
/// reset" is indistinguishable from "the reset never happened and this is the same run".
#[link_section = ".uninit"]
static mut BOOTS: u32 = 0;

/// An object id in NVM3's user range. Keys are 20 bits wide: `0xABCDEF` is not a key at all, NVM3 answers
/// `ECODE_NVM3_ERR_KEY_INVALID` (0xF000E00A) and the first version of this test read that as a failed storage path.
const KEY: u32 = 0x0400;
/// Two more keys in different ranges: NVM3 keys are 20 bits and some ranges are reserved by the SDK, so a collision
/// would look exactly like a broken read while `getObjectInfo` still answers.
const KEY_LOW: u32 = 0x0001;
const KEY_HIGH: u32 = 0x10000;
/// `0xB0 0x1D` in ASCII is a bond that came back.
const VALUE: [u8; 4] = [0xB0, 0x1D, 0x00, 0x01];

extern "C" {
    /// The default NVM3 instance's handle, set up by the stack's `nvm3_open`.
    static mut nvm3_defaultHandle: *mut u8;
    /// Returns `Ecode_t` — 0 on success — **not** a byte count. Reading it as a length made a working read look like
    /// "object not found" and sent this investigation down three wrong paths (measured 2026-11-17).
    fn nvm3_readData(handle: *mut u8, key: u32, data: *mut u8, len: usize) -> i32;
    fn nvm3_writeData(handle: *mut u8, key: u32, data: *const u8, len: usize) -> i32;
    /// What NVM3 asks for its page size (`nvm3_hal_flash.c:147`). It is computed from `DEVINFO.MEMINFO` as
    /// `1 << ((field + 10) & 0x1F)`; if the field reads wrong, NVM3 lays its pages out for a page smaller than the
    /// 8 KB the controller erases, which would explain a write that "succeeds" and an object that cannot be read.
    fn SYSTEM_GetFlashPageSize() -> u32;
    /// Forces NVM3 to rewrite its pages. If the object becomes readable only after this, the write path buffers and
    /// never flushes, which is a different defect from a broken read.
    fn nvm3_repack(handle: *mut u8) -> i32;
    fn MSC_ErasePage(address: u32) -> u32;
    fn MSC_WriteWord(address: u32, data: *const u8, num_bytes: u32) -> u32;
    /// NVM3's own view of the object: `(type, len)` and a status. Raw bytes say the data reached the flash; this says
    /// whether NVM3 agrees an object exists, which is the difference between "written" and "written where NVM3 looks".
    fn nvm3_getObjectInfo(handle: *mut u8, key: u32, obj_type: *mut u32, len: *mut usize) -> i32;

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

    let mut module = match Module::take() {
        Some(m) => m,
        None => loop {
            core::hint::spin_loop();
        },
    };
    let mut delay = module.delay();

    // The stack start is what opens NVM3, so it happens before any storage call.
    let mut ble = match Bluetooth::new(&mut delay) {
        Ok(b) => b,
        Err(_) => {
            mailbox::fail(b"nvm3: the stack did not start, so NVM3 was never opened");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    let _ = ble.stop_advertising();

    let handle = unsafe { nvm3_defaultHandle };
    if handle.is_null() {
        mailbox::fail(b"nvm3: default handle is null after start-up");
        loop {
            cortex_m::asm::wfi();
        }
    }

    // A counter rather than a constant: the value itself is the evidence. Run once and it reads 1; reset the board
    // and a counter that persisted reads 2. If it reads 1 every time, nothing survived, and no reset trickery is
    // needed to interpret that.
    let mut readback = [0u8; 4];
    let got = unsafe { nvm3_readData(handle, KEY, readback.as_mut_ptr(), readback.len()) };
    let previous = if got == 0 { u32::from_le_bytes(readback) } else { 0 };
    let next = previous.saturating_add(1);
    let status = unsafe { nvm3_writeData(handle, KEY, next.to_le_bytes().as_ptr(), 4) };

    let boots = unsafe {
        let n = core::ptr::addr_of_mut!(BOOTS);
        let v = core::ptr::read_volatile(n).wrapping_add(1);
        core::ptr::write_volatile(n, v);
        v
    };

    let mut obj_type: u32 = 0;
    let mut obj_len: usize = 0;
    let info_status = unsafe { nvm3_getObjectInfo(handle, KEY, &mut obj_type, &mut obj_len) };

    // Read it back in this same session: NVM3 keeps a cache, so a same-session read can succeed from RAM while the
    // flash image still has nothing. That separates "the write never reached the flash" from "it reached it and the
    // next open cannot validate it".
    let mut same_session = [0u8; 4];
    let same = unsafe { nvm3_readData(handle, KEY, same_session.as_mut_ptr(), same_session.len()) };
    let repack = unsafe { nvm3_repack(handle) };
    let mut after_repack = [0u8; 4];
    let repacked = unsafe { nvm3_readData(handle, KEY, after_repack.as_mut_ptr(), after_repack.len()) };

    // The flash path with no NVM3 in the way: one page above the image (the image ends well below 0x0803E000) is
    // erased, written and read back. If this fails, NVM3 cannot be expected to work and the defect is the controller.
    const PAGE: u32 = 0x0803_E000;
    const PATTERN: [u32; 4] = [0x1234_ABCD, 0xFEED_FACE, 0x00C0_FFEE, 0xDEAD_BEEF];
    let flash_erase = unsafe { MSC_ErasePage(PAGE) };
    let flash_write = unsafe { MSC_WriteWord(PAGE, PATTERN.as_ptr() as *const u8, 16) };
    let mut flash_ok = true;
    for (i, want) in PATTERN.iter().enumerate() {
        let got = unsafe { core::ptr::read_volatile((PAGE as usize + i * 4) as *const u32) };
        if got != *want {
            flash_ok = false;
        }
    }

    // The same write/read on two other key ranges.
    let mut alt = [0u8; 4];
    let mut alt_result = [0u32; 4];
    for (i, key) in [KEY_LOW, KEY_HIGH].iter().enumerate() {
        let w = unsafe { nvm3_writeData(handle, *key, alt.as_ptr(), 4) };
        let r = unsafe { nvm3_readData(handle, *key, alt.as_mut_ptr(), 4) };
        alt_result[i * 2] = w as u32;
        alt_result[i * 2 + 1] = r as u32;
    }

    let mut t = Text::new();
    t.s(b"nvm3: low=");
    t.u(alt_result[0]);
    t.s(b"/");
    t.u(alt_result[1]);
    t.s(b" high=");
    t.u(alt_result[2]);
    t.s(b"/");
    t.u(alt_result[3]);
    t.s(b" page=");
    t.u(unsafe { SYSTEM_GetFlashPageSize() });
    t.s(b" flash=");
    t.u(flash_erase);
    t.s(b"/");
    t.u(flash_write);
    t.s(b"/");
    t.u(flash_ok as u32);
    t.s(b" boots=");
    t.u(boots);
    t.s(b" info=");
    t.u(info_status as u32);
    t.s(b" t=");
    t.u(obj_type);
    t.s(b" l=");
    t.u(obj_len as u32);
    t.s(b" same=");
    t.u(same as u32);
    t.s(b" repack=");
    t.u(repack as u32);
    t.s(b" after=");
    t.u(repacked as u32);
    t.s(b" read=");
    t.u(got as u32);
    t.s(b" previous=");
    t.u(previous);
    t.s(b" wrote=");
    t.u(next);
    t.s(b" status=");
    t.u(status as u32);

    // One run proves the call works; two runs with an incrementing value prove the storage survives. A first run
    // reading 0 and writing 1 is a correct measurement, so it is reported as info, not as a failure.
    // `got` and `status` are Ecode_t: 0 is success. A first run reads an unknown key (an error code), so it is
    // reported as info; any run that reads a value and writes the next one is a pass.
    if status == 0 && got == 0 && previous > 0 {
        mailbox::pass(t.bytes());
    } else if status == 0 {
        mailbox::info(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
