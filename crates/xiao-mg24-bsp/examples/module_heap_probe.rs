//! What the Bluetooth heap looks like when the security manager fails to get 64 bytes.
//!
//! A review of `smp.c.obj` found that a Just Works pairing aborts in `smp_state_slave_public_key_wait` when
//! `bg_zalloc(64)` returns NULL, and that abort goes on the air as SMP reason 0x0B. The arena is 64 KiB, so a
//! 64-byte allocation failing is not a capacity problem — unless the stack has consumed the arena, in which case
//! it is. This example measures which:
//!
//! ```
//! in_use ≈ 65 536        the stack took the whole arena: capacity, and the fix is sizing
//! in_use modest          fragmentation, and the fix is the allocator
//! failures > 0           a failed allocation has already happened; last_failed_size says what was refused
//! ```

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_radio::Bluetooth;
use panic_halt as _;
use xiao_mg24_bsp::prelude::*;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

/// How long to let the stack settle and advertise before the heap is read.
const SETTLE_MS: u32 = 2_000;

struct Text {
    buf: [u8; 160],
    len: usize,
}

impl Text {
    fn new() -> Self {
        Self {
            buf: [0; 160],
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

    let before = efr32mg24_radio::heap_stats();
    mailbox::info(b"heap: before the stack");

    let mut ble = match Bluetooth::new(&mut delay) {
        Ok(b) => b,
        Err(_) => {
            mailbox::fail(b"heap: the stack did not start");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    let _ = ble.enable_pairing();
    let _ = ble.set_device_name("xiao-heap-probe");
    let _ = ble.start_connectable(200, 0);

    // Let the host and controller allocate whatever they allocate at steady state.
    for _ in 0..SETTLE_MS {
        while let Some(_event) = ble.poll_event() {}
        delay.delay_ms(1);
    }

    let after = efr32mg24_radio::heap_stats();
    let audit = efr32mg24_radio::heap_audit();

    let mut t = Text::new();
    t.s(b"heap: before_inuse=");
    t.u(before.0 as u32);
    t.s(b" inuse=");
    t.u(after.0 as u32);
    t.s(b" fails=");
    t.u(after.1);
    t.s(b" viol=");
    t.u(after.2);
    t.s(b" bad=");
    t.u(audit.0);
    t.s(b" total=");
    t.u(audit.1 as u32);
    t.s(b" badsize=");
    t.u(audit.2 as u32);

    // A pass means the arena was measured and nothing is corrupt. The interesting number is `inuse`: close to
    // 65 536 means the stack consumed the arena and the security manager has nothing left to ask for 64 bytes.
    if after.0 > 0 && audit.0 == 0 {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
