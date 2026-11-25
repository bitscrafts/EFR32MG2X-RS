//! The tick timer, measured rather than assumed.
//!
//! SYSRTC is a HAL service: one peripheral, shared by Bluetooth, Zigbee and Thread. This example proves the counter
//! runs and at what rate by comparing it against a **core-clock** time base over the same interval: two independent
//! clocks that agree is evidence, and one counter reading zero is not.
//!
//! ## The reference is TIMER0, not DWT, and that took two failures to learn
//!
//! The first version waited on the DWT cycle counter itself (`while dwt::since(c0) < 1_900_000`) with a
//! 40-million-iteration guard, and reported `the cycle counter never advanced`: the guard expired, so the counter had
//! moved less than 1.9 M cycles in 40 M iterations. A probe reading it across 10 M iterations saw 15 240 "cycles".
//!
//! The second version took the window from the SYSRTC -- a clock measured against a host stopwatch -- and still read
//! `core=283kHz`: 28 400 cycles for 100 ms of ticks. Two probes settled it, both anchored outside both loops:
//!
//! - **Detached** (flashed, read later over SWD): two reads 10.8 s apart on the host clock showed `c=29605` twice,
//!   while the SYSRTC advanced 346 091 ticks -- 32 060 Hz, correct. **The DWT counter was frozen.**
//! - **Attached** (`probe-rs run` reading RTT): DWT/SYSRTC came out at **578.4** over any interval, and 19 MHz
//!   divided by 32.768 kHz is **579.8**. Both counters correct to 0.2 %.
//!
//! So on this part **the DWT cycle counter only runs while a debug session is attached**: it lives in the debug domain
//! and stops when the probe lets go. Anything timed with `dwt` has to be measured under `probe-rs run`, never from a
//! mailbox record read afterwards. Full record: `docs/04-reference/DWT-NEEDS-A-DEBUG-SESSION.md`.
//!
//! The reference here is **TIMER0**, clocked from HFRCOEM23 (nominal 19 MHz, a separate oscillator from the core's own
//! HFRCO). It is not the core clock and the record does not pretend it is: what is measured is the ratio between two
//! oscillators, and the band below is wide enough for both to be at their nominal value together.
//!
//! ## What is checked
//!
//!   1. The timer clock, from TIMER0 counts and SYSRTC ticks over the same 100 ms: `counts / (ticks / 32768)`. It must
//!      land near 19 MHz, the nominal HFRCOEM23 rate.
//!   2. Compare channel 0, the way a scheduler uses it: schedule an absolute deadline, wait for the flag, then check
//!      the counter had reached it. That is the whole contract the vendor stack needs.
//!
//! VERIFIED ON HARDWARE (2026-11-24): the record is in this file's header and in the crate README's dated table.

#![no_std]
#![no_main]

use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::entry;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

/// SYSRTC ticks in 100 ms at 32.768 kHz. The window, and a rate this board measured against a host stopwatch.
const WINDOW_TICKS: u32 = 3277;
/// Nominal rate of the tick clock, used only to turn ticks into seconds.
const TICK_HZ: u64 = 32_768;
/// TIMER0's counter is 16 bits; the rate asked for is the one that puts TOP at its maximum, so the counter is a
/// free-running 16-bit window into the timer clock and the poll below only has to handle its wrap.
const TIMER_FULL_TOP_HZ: u32 = 290;

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
        if v == 0 {
            self.s(b"0");
            return;
        }
        let mut digits = [0u8; 10];
        let mut n = 0;
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
    mailbox::invalidate();
    let board = match Board::take() {
        Some(b) => b,
        None => loop {
            core::hint::spin_loop();
        },
    };
    let rtc = board.sysrtc().expect("SYSRTC is handed out once");

    // TIMER0 as a free-running counter into the fast clock. `pace_microphone` is the board's door to it: it unlocks
    // the timer, sets the prescaler to /1 and starts it, which is what a counter read needs. It returns the period it
    // programmed, and that number is the counter's wrap point.
    // The programmed period is not needed by the arithmetic: `u16::wrapping_sub` is correct for any wrap point, and
    // the poll is far more frequent than one wrap. It is read so a refused start reports instead of counting zeros.
    let _top = match board.pace_microphone(TIMER_FULL_TOP_HZ) {
        Some(p) => p,
        None => {
            mailbox::fail(b"rtc: TIMER0 could not be started");
            loop {
                core::hint::spin_loop();
            }
        }
    };

    // The window is the SYSRTC's own 100 ms. The iteration guard exists so a stopped SYSRTC reports a verdict
    // instead of hanging.
    // The window is the SYSRTC's own 100 ms; the timer is sampled inside it and its 16-bit wraps are added up. The
    // poll interval is a few counts, so a wrap can never be missed between two samples.
    let t0 = rtc.now();
    let mut prev = board.timer0_counter() as u16;
    let mut elapsed: u64 = 0;
    let mut guard = 0u32;
    while rtc.now().wrapping_sub(t0) < WINDOW_TICKS {
        let now = board.timer0_counter() as u16;
        elapsed += u64::from(now.wrapping_sub(prev));
        prev = now;
        guard = guard.wrapping_add(1);
        if guard == 20_000_000 {
            mailbox::fail(b"rtc: sysrtc stopped during the window");
            loop {
                core::hint::spin_loop();
            }
        }
    }
    let t1 = rtc.now();

    let ticks = u64::from(t1.wrapping_sub(t0));
    let cycles = elapsed;
    let core_hz = if ticks == 0 {
        0
    } else {
        cycles * TICK_HZ / ticks
    };

    // The compare channel, as a scheduler uses it.
    let deadline = rtc.now().wrapping_add(WINDOW_TICKS / 2); // ~50 ms
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
    l.s(b" cyc=");
    l.u(cycles as u32);
    l.s(b" fast=");
    l.u((core_hz / 1000) as u32);
    l.s(b"kHz late=");
    l.u(late);
    l.s(b" fired=");
    l.u(fired as u32);
    l.s(b" run=");
    l.u(rtc.is_running() as u32);

    // 19 MHz nominal for HFRCOEM23; 17.5-21 MHz leaves room for two oscillators at once without admitting a time base
    // that ticks at a fraction of it (that reads thousands, not millions).
    let ok = (17_500_000..=21_000_000).contains(&core_hz)
        && rtc.is_running()
        && fired
        && late < WINDOW_TICKS;
    if ok {
        mailbox::pass(&l.buf[..l.len]);
    } else {
        mailbox::fail(&l.buf[..l.len]);
    }

    loop {
        core::hint::spin_loop();
    }
}
