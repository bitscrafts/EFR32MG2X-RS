//! What clock is the radio actually running from? emlib answers, and that is the point.
//!
//! `crates/efr32mg24-emlib-sys` exists because the radio blobs call into emlib. An example that only
//! proved "it links" would pass the moment the archive was on the link line, whether or not the vendor
//! code was correct for this part. So this one asks emlib a question whose answer is measurable
//! independently: `SystemSYSCLKGet()` returns the core clock, and the same figure is available from
//! the cycle counter against a known delay.
//!
//! Two sources that must agree is the check. If emlib were built for the wrong device, or the clock
//! tree were misread, the two numbers would drift apart by exactly the ratio of the mistake.
//!
//! ## Reading it
//!
//! The mailbox and `scripts/read-mailbox.sh`, or by hand:
//!
//! ```text
//! ssh <build-host> 'sudo -n openocd -f interface/cmsis-dap.cfg -f /tmp/openocd-mg24-mini.cfg \
//!   -c "init; halt" -c "mdw 0x2003F000 12" -c "resume; shutdown"'
//! ```

#![no_std]
#![no_main]

use efr32mg24_hal::entry;
use efr32mg24_hal::prelude::*;
use panic_halt as _;

/// The mailbox the examples and `scripts/read-mailbox.sh` share.
#[path = "../../xiao-mg24-bsp/examples/common/mailbox.rs"]
mod mailbox;

use mailbox::Status;

// emlib, as compiled by efr32mg24-emlib-sys.
use efr32mg24_emlib_sys::{emlib_core_hz, emlib_sysclk_hz};

#[entry]
fn main() -> ! {
    let mut board = efr32mg24_hal::board::Board::take().expect("board");
    let mut delay = board.delay();

    mailbox::write(Status::Info, b"emlib: entered main");

    let sysclk = unsafe { emlib_sysclk_hz() };
    mailbox::write(Status::Info, b"emlib: sysclk ok");
    let coreclk = unsafe { emlib_core_hz() };
    mailbox::write(Status::Info, b"emlib: core ok");

    // Independent measurement: count cycles across a delay of known length. The delay is 10 ms, so
    // cycles / 10 000 gives Hz without needing to know the clock in advance.
    // DWT is a core debug block: enabling it is a register write, hence unsafe at the call site.
    unsafe { efr32mg24_hal::dwt::start() };
    let t0 = efr32mg24_hal::dwt::cycles();
    delay.delay_ms(10);
    let cycles = efr32mg24_hal::dwt::since(t0);
    let measured = cycles / 10; // cycles per ms = kHz
    mailbox::write(Status::Info, b"emlib: measured");

    let mut msg = [0u8; 64];
    let mut at = 0;
    mailbox::push(&mut msg, &mut at, b"emlib: sys=");
    at = push_dec(&mut msg, at, sysclk / 1000);
    mailbox::push(&mut msg, &mut at, b"k core=");
    at = push_dec(&mut msg, at, coreclk / 1000);
    mailbox::push(&mut msg, &mut at, b"k meas=");
    at = push_dec(&mut msg, at, measured);
    mailbox::push(&mut msg, &mut at, b"k");

    // Agreement within 5%: the two paths use different hardware, so exactness is not expected, but a
    // factor-of-two error (a wrong divider) must fail loudly.
    let agreed = measured.abs_diff(cycles / 10) < 1;
    let _ = agreed;
    let ok = {
        let a = sysclk / 1000;
        let b = measured;
        let diff = a.abs_diff(b);
        diff * 20 <= a // 5%
    };
    mailbox::write(if ok { Status::Pass } else { Status::Fail }, &msg[..at]);

    loop {
        delay.delay_ms(1000);
    }
}

/// Decimal, no allocation: the format the mailbox reader expects.
fn push_dec(out: &mut [u8], mut at: usize, mut value: u32) -> usize {
    if value == 0 {
        out[at] = b'0';
        return at + 1;
    }
    let mut digits = [0u8; 10];
    let mut n = 0;
    while value > 0 && n < 10 {
        digits[n] = b'0' + (value % 10) as u8;
        value /= 10;
        n += 1;
    }
    while n > 0 {
        n -= 1;
        out[at] = digits[n];
        at += 1;
    }
    at
}
