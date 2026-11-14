//! A BLE beacon: the smallest thing that proves the whole stack is alive.
//!
//! Plug the board into USB and this advertises a name for anyone with a phone. It is the radio
//! equivalent of the blink: no protocol on top, no service, no connection — just "the stack booted,
//! the crystal is running, the antenna is connected, and a scan finds it".
//!
//! ## What has to be true for the name to appear
//!
//! 1. **The stack boots.** `Bluetooth::new` resets the controller and waits for `system_boot`. If
//!    the HFXO never settles, the wait times out and the mailbox says so — that is a board fault,
//!    not a radio one.
//! 2. **The advertising data fits in 31 bytes.** A legacy PDU carries one name and one UUID; this
//!    example sets a name and lets the builder refuse one that is too long rather than truncating
//!    it silently.
//! 3. **The event loop keeps running.** Every command's answer arrives as an event, so the loop
//!    polls; a beacon that stops polling keeps advertising but stops accepting new commands.
//!
//! ## Reading the result without a phone
//!
//! The mailbox reports `adv=<set> name=<len>b` once advertising starts, and `evt=` counters as
//! events arrive. A phone is still the better instrument: it proves the antenna, not just the API.

#![no_std]
#![no_main]

use efr32mg24_hal::board::Board;
use efr32mg24_hal::entry;
use efr32mg24_hal::pin::Port;
use efr32mg24_hal::prelude::*;
use efr32mg24_radio::Bluetooth;
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

use efr32mg24_bluetooth_sys::blocking::BleError;

#[entry]
fn main() -> ! {
    let board = Board::take().expect("peripherals taken twice");
    let mut delay = board.delay();
    let led = board.output_on(Port::A, 7, true);

    let mut ble = match Bluetooth::beacon(&mut delay, "xiao-mg24") {
        Ok(ble) => ble,
        Err(e) => {
            // Which step failed, as a number. The message used to say only "did not boot", and a status code is
            // the difference between a guess and a diagnosis: 0xB007 means the stack started but never emitted
            // system_boot, 0xFFFF is an internal complaint, and anything else is the stack's own status from
            // `stack_init::start` or the first BGAPI call.
            let code: u16 = match e {
                BleError::BgapiError(c) => c,
                _ => 0xFFFF,
            };
            let (used, fails, viol) = efr32mg24_bluetooth_sys::heap::stats();
            let largest = efr32mg24_bluetooth_sys::heap::largest_free();
            let mut m = [0u8; 96];
            let mut n = 0;
            for b in b"ble: boot failed st=0x" {
                m[n] = *b;
                n += 1;
            }
            let hex = b"0123456789ABCDEF";
            for shift in [12u32, 8, 4, 0] {
                m[n] = hex[((code >> shift) & 0xF) as usize];
                n += 1;
            }
            // Heap state on the same line. "Out of memory" and "the arena was overwritten" are different
            // faults, and the status code alone cannot tell them apart.
            for b in b" heap=" {
                m[n] = *b;
                n += 1;
            }
            n = push_dec(&mut m, n, used as u32);
            for b in b"/f" {
                m[n] = *b;
                n += 1;
            }
            n = push_dec(&mut m, n, fails);
            for b in b"/v" {
                m[n] = *b;
                n += 1;
            }
            n = push_dec(&mut m, n, viol);
            for b in b"/free" {
                m[n] = *b;
                n += 1;
            }
            n = push_dec(&mut m, n, largest as u32);
            mailbox::write(mailbox::Status::Fail, &m[..n]);
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // Give the radio a moment to show whether it is really working, then report what the link layer did.
    //
    // **Why wait before reporting.** The API path answering 0 proves the host accepted the commands; it says nothing
    // about the radio. An advertising set that is created, configured and started, with a link layer that is never
    // serviced, produces exactly that success and no transmission -- so the report carries the service count, and a
    // count of zero is the difference between "the stack is advertising" and "the stack accepted an instruction".
    let mut waited = 0u32;
    while waited < 2_000 {
        let _ = ble.poll_event();
        delay.delay_ms(10);
        waited += 10;
    }

    let mut msg = [0u8; 96];
    let mut at = 0;
    msg[at..at + 4].copy_from_slice(b"adv=");
    at += 4;
    at = push_dec(&mut msg, at, u32::from(ble.advertising_set()));
    msg[at..at + 6].copy_from_slice(b" name=");
    at += 6;
    at = push_dec(&mut msg, at, 9);
    msg[at..at + 2].copy_from_slice(b"b ");
    at += 2;
    msg[at..at + 5].copy_from_slice(b"gatt=");
    at += 5;
    at = push_dec(&mut msg, at, u32::from(ble.is_connected()));
    msg[at..at + 4].copy_from_slice(b" ll=");
    at += 4;
    at = push_dec(
        &mut msg,
        at,
        efr32mg24_bluetooth_sys::stack_init::link_layer_services(),
    );
    mailbox::write(mailbox::Status::Pass, &msg[..at]);

    // One blink per event: the LED is the cheapest event counter there is, and it keeps the board
    // saying "I am still in the loop" while a phone does the actual measuring.
    // The count is what the LED blinks: one blink per event, which is the cheapest proof the loop is alive.
    let mut _events = 0u32;
    loop {
        if ble.poll_event().is_some() {
            _events += 1;
            led.on();
            delay.delay_ms(20);
            led.off();
        } else {
            // Sleep, and note what that costs: the stack's timers are polled by this project because the SYSRTC compare
            // interrupt does not fire yet, so a loop parked in `wfi` only advances them when an interrupt arrives.
            // Replacing this with a 1 ms poll was tried and made no measurable difference to the on-air rate -- the
            // receiver saturates long before the transmitter does (see BLE-STATUS.md), so the experiment could not
            // answer the question and the power cost stayed.
            cortex_m::asm::wfi();
        }
    }
}
