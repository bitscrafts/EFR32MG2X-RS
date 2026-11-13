//! Custom advertising data: a manufacturer-specific payload instead of a name.
//!
//! This is what a real beacon is. A name is for humans browsing a scanner list; a product advertises
//! a *framed* payload that only its own app understands — an iBeacon has a 128-bit proximity UUID, a
//! major, a minor and a reference RSSI, and all of that has to fit in the same 31 bytes.
//!
//! ## The budget, counted
//!
//! ```text
//! flags                                  3 bytes
//! manufacturer structure header          2 bytes   (length + 0xFF)
//! company identifier                     2 bytes
//! payload (this example)                23 bytes
//! ------------------------------------------------
//! total                                 30 bytes   of 31
//! ```
//!
//! One byte spare. That is the whole design constraint of legacy advertising, and it is why a product
//! with a name *and* manufacturer data usually moves the name to the scan response — or moves to
//! extended advertising, which is a different command set at the cost of compatibility.
//!
//! ## The frame this example sends
//!
//! ```text
//! byte 0      frame version (0x01)
//! bytes 1..3  device id, little endian
//! byte 3      firmware major
//! byte 4      firmware minor
//! bytes 5..9  uptime in seconds, little endian
//! ```
//!
//! A receiving app can parse it in ten lines, which is the bar for a beacon payload: if the frame needs
//! a specification document, it does not fit in 31 bytes.
//!
//! ## Testing it
//!
//! nRF Connect for Mobile shows manufacturer data as a hex blob under the device — the name in the
//! list comes from a scan response, so a passive scanner may show an address with no name. The frame
//! above is what appears in the hex: `ff <company_lo> <company_hi> 01 <id> <fw> <uptime>`.

#![no_std]
#![no_main]

use efr32mg24_hal::board::Board;
use efr32mg24_hal::dwt;
use efr32mg24_hal::entry;
use efr32mg24_hal::pin::Port;
use efr32mg24_hal::prelude::*;
use efr32mg24_radio::Bluetooth;
use panic_halt as _;

#[path = "../../xiao-mg24-bsp/examples/common/mailbox.rs"]
mod mailbox;

/// A company identifier from the Bluetooth SIG's assigned-numbers list. `0xFFFF` is reserved and the
/// builder refuses it; pick your own from the registry before shipping.
const COMPANY_ID: u16 = 0x02E5;

/// Frames go out with this version so a receiver can reject what it does not know.
const FRAME_VERSION: u8 = 0x01;

/// Device id and firmware version, as a product would build them in.
const DEVICE_ID: u16 = 0x0001;
const FW_MAJOR: u8 = 0;
const FW_MINOR: u8 = 1;

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
    let board = Board::take().expect("peripherals taken twice");
    let mut delay = board.delay();
    let led = board.output_on(Port::A, 7, true);

    // The cycle counter gives the uptime the frame carries. Started before the stack boots so the
    // number means "since power on", not "since advertising started".
    // SAFETY: nothing else in this example uses DWT.
    let boot = unsafe { dwt::start() };

    let mut ble = match Bluetooth::new(&mut delay) {
        Ok(ble) => ble,
        Err(_) => {
            mailbox::write(mailbox::Status::Fail, b"adv: stack did not boot");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // 23 bytes of payload: version, id, firmware, uptime. Rebuilt on every advertisement so the
    // uptime advances — a beacon that reports a frozen uptime is a beacon whose app cannot tell a
    // reboot from a stall.
    let mut frame = [0u8; 9];
    let mut last_second = u32::MAX;
    let mut updates: u32 = 0;

    loop {
        let uptime_s = dwt::cycles_to_us(dwt::since(boot), 19_000_000) / 1_000_000;
        if uptime_s != last_second {
            last_second = uptime_s;
            frame[0] = FRAME_VERSION;
            frame[1..3].copy_from_slice(&DEVICE_ID.to_le_bytes());
            frame[3] = FW_MAJOR;
            frame[4] = FW_MINOR;
            frame[5..9].copy_from_slice(&uptime_s.to_le_bytes());

            if ble.set_manufacturer_data(COMPANY_ID, &frame).is_ok() {
                // Restart advertising so the new payload goes out; the stack does not re-read the
                // advertising data while a set is running.
                if ble.start_advertising(200, 0).is_ok() {
                    updates += 1;
                    let mut msg = [0u8; 64];
                    let mut at = 0;
                    msg[at..at + 10].copy_from_slice(b"adv frame=");
                    at += 10;
                    at = push_dec(&mut msg, at, frame.len() as u32);
                    msg[at..at + 4].copy_from_slice(b"B up=");
                    at += 4;
                    at = push_dec(&mut msg, at, uptime_s);
                    msg[at..at + 4].copy_from_slice(b" n=");
                    at += 3;
                    at = push_dec(&mut msg, at, updates);
                    mailbox::write(mailbox::Status::Pass, &msg[..at]);
                } else {
                    mailbox::write(mailbox::Status::Fail, b"adv: advertising refused");
                }
            } else {
                // The only way this fails is the budget: the builder refuses rather than truncating,
                // because a truncated frame is a frame a receiver parses wrongly.
                mailbox::write(
                    mailbox::Status::Fail,
                    b"adv: manufacturer data does not fit in 31B",
                );
            }
        }

        // One blink per second, so the board says "still advertising" without a host attached.
        led.on();
        delay.delay_ms(30);
        led.off();
        delay.delay_ms(970);
    }
}
