//! The 32-bit counter's wrap, exercised in milliseconds instead of 36.4 hours.
//!
//! ## What this proves
//!
//! SYSRTC's counter is 32 bits at 32768 Hz: it returns to zero every **36.4 hours**. The 64-bit tick needs to know how
//! many wraps there have been, and there were two ways to know:
//!
//! - **By polling** (`WideTick`): compare each reading with the previous one and count a wrap when the value falls.
//!   It works, but it requires someone to call `now_wide` at least once every 36.4 h -- whoever does not loses exactly
//!   36.4 h of the clock, silently.
//! - **By interrupt** (`GRP0_IF.OVF`): the peripheral counts the wraps and `now_wide` needs nobody.
//!
//! This example arms the interrupt, **puts the counter 4096 ticks before the wrap** and checks that the 64-bit tick
//! crosses the wrap going up, not going down. If the high bit came from polling, or did not exist, the second value
//! would be *smaller* than the first.
//!
//! ## Why polling has to stay off
//!
//! Both would count the same wrap: polling would see the counter fall and add one, and the interrupt would add another.
//! The tick would jump 36.4 h forward. That is why `enable_overflow_interrupt` takes polling out of the path -- it is a
//! choice, not a sum.
//!
//! Read it with `scripts/read-mailbox.sh`.

#![no_std]
#![no_main]

#[path = "common/mailbox.rs"]
mod mailbox;

use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m_rt::entry;
use defmt_rtt as _;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::pin::Port;
use efr32mg2x_hal::sysrtc::Sysrtc;
use panic_probe as _;

/// How many wraps the handler counted.
static WRAPS_IN_HANDLER: AtomicU32 = AtomicU32::new(0);

/// How many times the handler ran, to tell "it did not wrap" from "it wrapped and did not serve the event".
static ENTRIES_IN_HANDLER: AtomicU32 = AtomicU32::new(0);

/// The value put into the counter: 4096 ticks before the wrap, that is 125 ms.
const BEFORE_THE_WRAP: u32 = 0xFFFF_F000;

/// Ticks of 4096 at 32768 Hz.
const TICKS_UNTIL_WRAP: u32 = 0x0000_1000;

/// An iteration bound in the loop, so an error does not spin for ever.
const SPIN_LIMIT: u32 = 40_000_000;

/// The PAC declares `PROVIDE(SYSRTC_APP_IRQHandler = DefaultHandler)` in `device.x`, so a definition with this exact
/// name takes the place of the handler that does not return.
#[no_mangle]
pub extern "C" fn SYSRTC_APP_IRQHandler() {
    ENTRIES_IN_HANDLER.fetch_add(1, Ordering::Relaxed);
    // The two events share one vector: serving only the comparator would leave the wrap's flag set and the vector
    // would re-enter for ever.
    let (_, wrapped) = Sysrtc::service_interrupts();
    if wrapped {
        WRAPS_IN_HANDLER.fetch_add(1, Ordering::Relaxed);
    }
}

fn report(ok: bool, wraps: u32, entries: u32, before: u32, after: u32, delta: u32, ticks: u32) {
    let mut m = [0u8; 140];
    let mut n = 0usize;

    fn text(m: &mut [u8], n: &mut usize, s: &[u8]) {
        for b in s {
            if *n < m.len() {
                m[*n] = *b;
                *n += 1;
            }
        }
    }
    fn number(m: &mut [u8], n: &mut usize, v: u32) {
        let mut d = [0u8; 10];
        let mut k = 0usize;
        let mut v = v;
        loop {
            d[k] = b'0' + (v % 10) as u8;
            v /= 10;
            k += 1;
            if v == 0 || k == d.len() {
                break;
            }
        }
        while k > 0 {
            k -= 1;
            if *n < m.len() {
                m[*n] = d[k];
                *n += 1;
            }
        }
    }

    text(&mut m, &mut n, b"wrap: wraps=");
    number(&mut m, &mut n, wraps);
    text(&mut m, &mut n, b" irq=");
    number(&mut m, &mut n, entries);
    text(&mut m, &mut n, b" before=");
    number(&mut m, &mut n, before);
    text(&mut m, &mut n, b" after=");
    number(&mut m, &mut n, after);
    text(&mut m, &mut n, b" delta=");
    number(&mut m, &mut n, delta);
    text(&mut m, &mut n, b" ticks=");
    number(&mut m, &mut n, ticks);

    if ok {
        mailbox::write(mailbox::Status::Pass, &m[..n]);
    } else {
        mailbox::write(mailbox::Status::Fail, &m[..n]);
    }
}

#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();
    let led = board.output_on(Port::A, 7, false);
    let rtc = board.sysrtc().expect("SYSRTC is handed out once");

    // Arm the interrupt first, or the wrap happens before anyone is counting it.
    rtc.enable_overflow_interrupt();
    rtc.unmask_interrupt();

    // 4096 ticks before the wrap. That is what turns a 36.4 h event into a 125 ms one.
    rtc.set_counter(BEFORE_THE_WRAP);

    let before = rtc.now_wide();
    let low_before = before as u32;

    // `now_wide` is deliberately not called during the wrap: the value has to come from the handler, not from a read.
    let mut spins = 0u32;
    while WRAPS_IN_HANDLER.load(Ordering::Relaxed) == 0 {
        cortex_m::asm::wfi();
        spins += 1;
        if spins > SPIN_LIMIT {
            break;
        }
    }

    let after = rtc.now_wide();
    let wraps = WRAPS_IN_HANDLER.load(Ordering::Relaxed);
    let entries = ENTRIES_IN_HANDLER.load(Ordering::Relaxed);
    let delta = after.wrapping_sub(before) as u32;
    let ticks = rtc.now();

    // The tick has to go up. With the high bit coming from polling switched off, or absent, `after` would be smaller
    // than `before` and this test would say exactly that.
    let went_up = after > before;
    // The distance between the two instants is what was left until the wrap plus what has passed after it. Had the
    // high bit not come in, `delta` would be a meaningless 32-bit difference -- hence the window.
    let coherent = delta >= TICKS_UNTIL_WRAP && delta <= TICKS_UNTIL_WRAP + 3_276;
    let ok = wraps == 1 && went_up && coherent;

    report(ok, wraps, entries, low_before, after as u32, delta, ticks);
    led.on();

    loop {
        cortex_m::asm::wfi();
    }
}
