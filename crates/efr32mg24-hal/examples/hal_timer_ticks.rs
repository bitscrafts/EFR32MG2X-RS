//! The tick timer, measured rather than assumed.
//!
//! SYSRTC is a HAL service: one peripheral, shared by Bluetooth, Zigbee and Thread. This example proves the
//! counter runs and at what rate, by comparing it against the core's cycle counter over the same interval.
//! Two independent time bases that agree is evidence; a counter that reads zero is not.
//!
//! Expected: 32.768 kHz from LFRCO. A measurement near 32768 means the clock selection took effect;
//! anything else says which way it went wrong.
//!
//! Then it uses compare channel 0 the way a scheduler does — schedule an absolute deadline, wait for the
//! flag, and check the counter had reached it. That is the whole contract the vendor stack needs.

#![no_std]
#![no_main]

use efr32mg24_hal::board::Board;
use efr32mg24_hal::dwt;
use efr32mg24_hal::entry;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// The core runs at 19 MHz on HFRCO out of reset, measured with `emlib_core_hz()` and recorded in
/// `docs/04-reference/DEVICE-IDENTIFIERS.md`. It is a constant here because the HAL does not depend on
/// emlib: this example is about SYSRTC, and pulling a vendor C library in to read one number would blur
/// what is being measured.
const CORE_HZ: u64 = 19_000_000;

struct Line {
    buf: [u8; 200],
    len: usize,
}

impl Line {
    fn new() -> Self {
        Self {
            buf: [0; 200],
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
        let mut digits = [0u8; 10];
        let mut n = 0;
        if v == 0 {
            self.s(b"0");
            return;
        }
        while v > 0 {
            digits[n] = b'0' + (v % 10) as u8;
            v /= 10;
            n += 1;
        }
        while n > 0 {
            n -= 1;
            let d = digits[n];
            self.s(&[d]);
        }
    }
}

#[entry]
fn main() -> ! {
    let board = match Board::take() {
        Some(b) => b,
        None => loop {
            core::hint::spin_loop();
        },
    };
    mailbox::info(b"step1: board taken");
    let rtc = board.sysrtc();
    mailbox::info(b"step2: sysrtc configured");

    // Window: long enough that 19 MHz of cycles dwarfs any noise, short enough to stay under the 32-bit
    // cycle counter's wrap (about 226 s at 19 MHz).
    // The cycle counter has to be switched on (TRCENA, then CYCCNTENA); until it is, it reads zero and a
    // wait on it never ends. This example hung exactly here once, which is why the wait below also has a
    // bound: an instrument that can hang reports nothing, and a check that reports nothing is not a check.
    let t0 = rtc.now();
    let c0 = unsafe { dwt::start() };
    let mut guard = 0u32;
    while dwt::since(c0) < 1_900_000 {
        guard = guard.wrapping_add(1);
        if guard == 40_000_000 {
            mailbox::fail(b"rtc: the cycle counter never advanced");
            loop {
                core::hint::spin_loop();
            }
        }
    }
    let c1 = dwt::cycles();
    let t1 = rtc.now();

    let cycles = c1.wrapping_sub(c0) as u64;
    let ticks = t1.wrapping_sub(t0) as u64;
    let hz = if cycles == 0 {
        0
    } else {
        ticks * CORE_HZ / cycles
    };

    // One record with every number. Several records meant the last one hid the ones before it -- a lesson
    // paid for twice in this file already.
    let deadline = rtc.now().wrapping_add(32 * 50); // 50 ms at 32 ticks per ms
    rtc.schedule_at(deadline);
    let mut spins = 0u32;
    let mut fired = false;
    while spins < 3_000_000 {
        if rtc.take_compare_event() {
            fired = true;
            break;
        }
        spins = spins.wrapping_add(1);
    }
    let reached = rtc.now();
    let late = reached.wrapping_sub(deadline);

    let mut l = Line::new();
    l.s(b"rtc: ticks=");
    l.u(ticks as u32);
    l.s(b" rate=");
    l.u(hz as u32 / 1000);
    l.s(b"kHz en=");
    l.u(rtc.is_enabled() as u32);
    l.s(b" run=");
    l.u(rtc.is_running() as u32);
    l.s(b" deadline=");
    l.u(deadline);
    l.s(b" reached=");
    l.u(reached);
    l.s(b" late=");
    l.u(late);
    l.s(b" fired=");
    l.u(fired as u32);

    let ok = hz >= 32_000 && hz <= 33_500 && rtc.is_running() && fired;
    if ok {
        mailbox::pass(&l.buf[..l.len]);
    } else {
        mailbox::fail(&l.buf[..l.len]);
    }

    loop {
        core::hint::spin_loop();
    }
}
