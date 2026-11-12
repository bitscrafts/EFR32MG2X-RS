//! Who is this chip? Read from DEVINFO, not from a label.
//!
//! Three identifiers matter for the radio work, and all three come out of the part:
//!
//! - the **64-bit unique id** (`SYSTEM_GetUnique`), which is what Silicon Labs' stacks derive a BLE
//!   address from — without it a packet cannot be attributed to a board on the bench;
//! - the **chip revision**, which decides which errata apply;
//! - the **core clock**, which the delay conversions depend on.
//!
//! Everything is reported through the mailbox as hex so it can be compared with the same numbers read
//! over SWD, from the USB descriptor, or from a document — three sources that must agree before any of
//! them is trusted.

#![no_std]
#![no_main]

use efr32mg24_hal::entry;
use efr32mg24_hal::prelude::*;
use panic_halt as _;

#[path = "../../xiao-mg24-bsp/examples/common/mailbox.rs"]
mod mailbox;

use mailbox::Status;

use efr32mg24_emlib_sys::{emlib_chip_revision, emlib_sysclk_hz, emlib_unique_id};

#[entry]
fn main() -> ! {
    let board = efr32mg24_hal::board::Board::take().expect("board");
    let _delay = board.delay();

    mailbox::write(Status::Info, b"id: entered main");

    let unique = unsafe { emlib_unique_id() };
    let revision = unsafe { emlib_chip_revision() };
    let sysclk = unsafe { emlib_sysclk_hz() };

    let mut msg = [0u8; 96];
    let mut at = 0;
    mailbox::push(&mut msg, &mut at, b"id uid=");
    at = push_hex64(&mut msg, at, unique);
    mailbox::push(&mut msg, &mut at, b" rev=");
    at = push_hex32(&mut msg, at, revision);
    mailbox::push(&mut msg, &mut at, b" clk=");
    at = push_dec(&mut msg, at, sysclk / 1000);
    mailbox::push(&mut msg, &mut at, b"k");

    // A unique id of all ones or all zeros is what an unprogrammed or misread DEVINFO looks like, and
    // reporting it as a success would send the next step chasing a phantom device.
    let ok = unique != 0 && unique != u64::MAX;
    mailbox::write(if ok { Status::Pass } else { Status::Fail }, &msg[..at]);

    loop {
        // Nothing to wait for: the record is written, and sleeping would gain nothing measurable
        // here because the whole point of this example is to leave the identity in RAM for a reader.
        core::hint::spin_loop();
    }
}

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

fn push_hex32(out: &mut [u8], at: usize, value: u32) -> usize {
    let mut at = at;
    for shift in (0..8).rev() {
        let nibble = (value >> (shift * 4)) & 0xF;
        out[at] = if nibble < 10 {
            b'0' + nibble as u8
        } else {
            b'a' + (nibble - 10) as u8
        };
        at += 1;
    }
    at
}

fn push_hex64(out: &mut [u8], at: usize, value: u64) -> usize {
    let at = push_hex32(out, at, (value >> 32) as u32);
    push_hex32(out, at, value as u32)
}
