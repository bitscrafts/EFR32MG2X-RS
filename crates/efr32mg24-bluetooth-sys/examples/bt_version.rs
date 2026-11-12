//! FFI, one call deep: ask the Bluetooth stack for its version.
//!
//! This is the `-sys` crate's own level, and it is the rung below `efr32mg24-radio`'s examples:
//! no safe wrapper, no advertising data, no event loop — one BGAPI command issued straight against
//! the bindings, with the reply decoded from the emulated union.
//!
//! ## Why it is worth having when the radio crate has better examples
//!
//! When a link fails, this example tells you *which* part failed. It exercises exactly three things:
//!
//! 1. the bindings (a function that exists in `sl_bt_api.h` and therefore in `bindings.rs`),
//! 2. the blob (the symbol is resolved by `libbluetooth_host`),
//! 3. the CMSIS helpers in `cmsis-stubs`, because the host blob's first command takes a critical
//!    section.
//!
//! A failure here is a build-provenance failure. A failure in `ble_beacon` and a success here is a
//! stack-configuration failure. Those two are days apart in diagnosis time, and this example is
//! what makes them distinguishable without a phone.
//!
//! ## The reply, decoded by hand
//!
//! `sl_bt_system_get_version` answers with event `sl_bt_evt_system_version`, whose payload is
//! written into the caller's response buffer. Reading it means taking the message header apart and
//! reading the union — which is the whole point: this example shows what the safe layer is doing
//! for you, and what it costs to not have it.

#![no_std]
#![no_main]

use efr32mg24_bluetooth_sys::ffi as bt;
use efr32mg24_hal::board::Board;
use efr32mg24_hal::entry;
use efr32mg24_hal::pin::Port;
use efr32mg24_hal::prelude::*;
use panic_halt as _;

#[path = "../../xiao-mg24-bsp/examples/common/mailbox.rs"]
mod mailbox;

fn push_dec(out: &mut [u8], mut at: usize, mut value: u32) -> usize {
    let mut digits = [0u8; 10];
    let mut n = 0;
    loop {
        digits[n] = b'0' + (value % 10) as u8;
        value /= 10;
        n += 1;
        if value == 0 || n == digits.len() {
            break;
        }
    }
    for i in (0..n).rev() {
        out[at] = digits[i];
        at += 1;
    }
    at
}

#[entry]
fn main() -> ! {
    let mut board = Board::take().expect("peripherals taken twice");
    let mut delay = board.delay();
    let led = board.output_on(Port::A, 7, true);

    // Six out-parameters: major, minor, patch, build, bootloader version, and a build hash. All
    // are written by the stack; each is a local, so nothing can alias.
    let mut major: u16 = 0;
    let mut minor: u16 = 0;
    let mut patch: u16 = 0;
    let mut build: u16 = 0;
    let mut bootloader: u32 = 0;
    let mut hash: u32 = 0;
    // SAFETY: six valid pointers to six distinct locals, and the stack writes each exactly once.
    let status = unsafe {
        bt::sl_bt_system_get_version(
            &mut major,
            &mut minor,
            &mut patch,
            &mut build,
            &mut bootloader,
            &mut hash,
        )
    };

    let mut msg = [0u8; 64];
    let mut at = 0;
    if status != 0 {
        msg[at..at + 12].copy_from_slice(b"bt version: ");
        at += 12;
        at = push_dec(&mut msg, at, status);
        mailbox::write(mailbox::Status::Fail, &msg[..at]);
    } else {
        // A version of 0.0.0 from a blob that links would mean the stack answered nothing, which
        // is not "version zero" -- it is "this call did not reach the stack".
        msg[at..at + 9].copy_from_slice(b"bt ver: ");
        at += 9;
        at = push_dec(&mut msg, at, u32::from(major));
        msg[at] = b'.';
        at += 1;
        at = push_dec(&mut msg, at, u32::from(minor));
        msg[at] = b'.';
        at += 1;
        at = push_dec(&mut msg, at, u32::from(patch));
        msg[at..at + 5].copy_from_slice(b" bld=");
        at += 5;
        at = push_dec(&mut msg, at, u32::from(build));
        mailbox::write(
            if major == 0 && minor == 0 && patch == 0 {
                mailbox::Status::Fail
            } else {
                mailbox::Status::Pass
            },
            &msg[..at],
        );
    }

    loop {
        led.on();
        delay.delay_ms(500);
        led.off();
        delay.delay_ms(500);
    }
}
