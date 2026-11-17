//! Scan from the device: the one test of the radio path that does not need the host's dongle to receive.
//!
//! The host's adapter discovers nothing at all -- not this board, not its neighbours -- so "the board does not
//! advertise" and "the dongle cannot hear" look identical from that side. This inverts the question: the board
//! listens, the host advertises, and whichever side is deaf becomes obvious.
//!
//! Put the host in discoverable mode first:
//!
//! ```text
//! sudo bluetoothctl discoverable on
//! ```
//!
//! A pass needs at least one report. The record names the strongest one so the host can be recognised.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_radio::{BleEvent, Bluetooth};
use panic_halt as _;
use xiao_mg24_bsp::prelude::*;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

const SCAN_MS: u32 = 20_000;

struct Text {
    buf: [u8; 150],
    len: usize,
}

impl Text {
    fn new() -> Self {
        Self {
            buf: [0; 150],
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

    let mut ble = match Bluetooth::new(&mut delay) {
        Ok(b) => b,
        Err(_) => {
            mailbox::fail(b"devscan: the stack did not start");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    let rc_scan = ble.start_scan(200, 100, true).is_ok();

    let mut reports: u32 = 0;
    let mut rssi_best: i32 = -128;
    for _ in 0..SCAN_MS {
        while let Some(event) = ble.poll_event() {
            match event {
                // Any advertisement-shaped event counts; the API names differ per stack version, and a report is a
                // report: the number of them is the answer to "can this board hear anything at all".
                BleEvent::Advertisement { .. } => {
                    reports += 1;
                }
                BleEvent::Unknown { .. } => {
                    reports += 1;
                }
                _ => {}
            }
        }
        // The RSSI is not in the event the API exposes here, so the record reports the count, which is enough to
        // separate a deaf radio from a silent host.
        let _ = rssi_best;
        delay.delay_ms(1);
    }
    let _ = ble.stop_scan();

    let mut t = Text::new();
    t.s(b"devscan: scan=");
    t.u(rc_scan as u32);
    t.s(b" reports=");
    t.u(reports);
    if reports > 0 {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
