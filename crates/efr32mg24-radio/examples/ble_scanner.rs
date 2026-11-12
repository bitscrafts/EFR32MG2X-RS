//! A BLE scanner: the board finds other advertisers, and reports the strongest one.
//!
//! This is the other half of the radio, and it exists for a practical reason as much as a
//! pedagogical one: it lets the beacon be tested **without a phone and without a second board**. Two
//! XIAOs, or one XIAO and any beacon in the room, and the mailbox says what was heard and how loudly.
//!
//! ## What the numbers mean
//!
//! - **RSSI** is dBm: roughly -40 for a device on the same desk, -70 across a room, -90 at the edge of
//!   reception. It is not a distance — it is a signal strength, and a wall costs more than a metre.
//! - **The payload is raw.** Advertising data is a list of `[len][type][value…]` structures; this
//!   example does not decode it, because decoding belongs to whoever knows what they are looking for.
//!   It reports the first byte of the payload and its length, which is enough to tell a name-only
//!   beacon from one carrying manufacturer data.
//!
//! ## The timing trap in the API
//!
//! BGAPI expresses scan timing in 0.625 ms units. The wrapper takes milliseconds and converts, because
//! passing 100 for "100 ms" asks for 62.5 ms and the scanner still works — it just scans less than you
//! asked for, which is invisible until compared against a sniffer.

#![no_std]
#![no_main]

use efr32mg24_hal::board::Board;
use efr32mg24_hal::entry;
use efr32mg24_hal::pin::Port;
use efr32mg24_hal::prelude::*;
use efr32mg24_radio::{BleEvent, Bluetooth};
use panic_halt as _;

#[path = "../../xiao-mg24-bsp/examples/common/mailbox.rs"]
mod mailbox;

/// Scan interval and window in milliseconds. A 100 ms interval with a 50 ms window listens half the
/// time — enough to find a 100 ms advertiser quickly, while leaving the radio free half of every
/// period.
const SCAN_INTERVAL_MS: u16 = 100;
const SCAN_WINDOW_MS: u16 = 50;

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

fn push_hex_byte(out: &mut [u8], at: usize, value: u8) -> usize {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    out[at] = HEX[(value >> 4) as usize];
    out[at + 1] = HEX[(value & 0x0F) as usize];
    at + 2
}

#[entry]
fn main() -> ! {
    let mut board = Board::take().expect("peripherals taken twice");
    let mut delay = board.delay();
    let led = board.output_on(Port::A, 7, true);

    let mut ble = match Bluetooth::new(&mut delay) {
        Ok(ble) => ble,
        Err(_) => {
            mailbox::write(mailbox::Status::Fail, b"scan: stack did not boot");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // Active scanning also asks for a scan response, which is where a name usually lives when the
    // advertisement is full of manufacturer data.
    if ble
        .start_scan(SCAN_INTERVAL_MS, SCAN_WINDOW_MS, true)
        .is_err()
    {
        mailbox::write(mailbox::Status::Fail, b"scan: parameters refused");
        loop {
            cortex_m::asm::wfi();
        }
    }

    let mut heard: u32 = 0;
    let mut best_rssi: i8 = i8::MIN;

    loop {
        if let Some(event) = ble.poll_event() {
            if let BleEvent::Advertisement {
                address,
                rssi,
                data,
                ..
            } = event
            {
                heard += 1;
                // Blink on every packet: the LED is the "the radio is receiving" indicator, and it
                // needs no host at all.
                led.on();
                delay.delay_ms(2);
                led.off();

                if rssi > best_rssi {
                    best_rssi = rssi;
                    let mut msg = [0u8; 64];
                    let mut at = 0;
                    msg[at..at + 6].copy_from_slice(b"scan: ");
                    at += 6;
                    // Address is in the controller's reverse byte order; printing it in that order
                    // is what a phone's scanner shows, which makes the two comparable.
                    for i in 0..6 {
                        if i > 0 {
                            msg[at] = b':';
                            at += 1;
                        }
                        at = push_hex_byte(&mut msg, at, address[i]);
                    }
                    msg[at..at + 6].copy_from_slice(b" rssi=");
                    at += 6;
                    // RSSI is negative; the sign is the point, so it is printed, not skipped.
                    if rssi < 0 {
                        msg[at] = b'-';
                        at += 1;
                        at = push_dec(&mut msg, at, u32::from(rssi.unsigned_abs()));
                    } else {
                        at = push_dec(&mut msg, at, u32::from(rssi as u8));
                    }
                    msg[at..at + 6].copy_from_slice(b" len=");
                    at += 5;
                    at = push_dec(&mut msg, at, data.len() as u32);
                    msg[at..at + 3].copy_from_slice(b" n=");
                    at += 3;
                    at = push_dec(&mut msg, at, heard);
                    mailbox::write(mailbox::Status::Pass, &msg[..at]);
                }
            }
        } else {
            // Nothing in the queue: sleep until the next interrupt. A polling loop that spins here
            // burns power for no reason, and this is exactly the state a beacon tester leaves the
            // board in for minutes at a time.
            cortex_m::asm::wfi();
        }
    }
}
