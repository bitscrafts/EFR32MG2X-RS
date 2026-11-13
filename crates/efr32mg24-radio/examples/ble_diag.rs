//! `ble_beacon` plus the RAIL call counters of `efr32mg24_radio::diag`: a beacon that says what the link layer asked of the radio.
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
    efr32mg24_radio::diag::init();
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

    // Report once a second: the stack's service count and the RAIL call counters. The full table is a `.bss` static
    // (`DIAG`), read from the host without halting; this line is only the headline.
    loop {
        let mut waited = 0u32;
        while waited < 1_000 {
            let _ = ble.poll_event();
            delay.delay_ms(10);
            waited += 10;
        }
        let mut msg = [0u8; 96];
        let mut at = efr32mg24_radio::diag::summary(&mut msg);
        msg[at..at + 4].copy_from_slice(b" ll=");
        at += 4;
        at = push_dec(
            &mut msg,
            at,
            efr32mg24_bluetooth_sys::stack_init::link_layer_services(),
        );
        mailbox::write(mailbox::Status::Pass, &msg[..at]);
        led.on();
        delay.delay_ms(5);
        led.off();
    }
}
