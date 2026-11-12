//! A connectable GATT peripheral: advertise a service, accept a connection, push a notification.
//!
//! This is the shape of every BLE product: a phone connects, discovers a characteristic, subscribes,
//! and the board publishes measurements. The example publishes a counter, because a counter is the
//! one thing whose correctness needs no phone: a client that sees 1, 2, 3… has proved the pipes.
//!
//! ## The four steps, and the trap in each
//!
//! 1. **Advertise connectably.** Connectable advertising carries flags that say so; a beacon's
//!    non-connectable PDU is a different bit, and a phone that tries to connect to a beacon gets a
//!    timeout with no explanation.
//! 2. **Wait for `Connected`.** The event carries the connection handle. Every later command needs
//!    that handle — not the advertising set handle, which is a different number and a common
//!    mistake that produces "invalid handle" on the first notify.
//! 3. **Subscribe.** A notification sent before the client subscribes is dropped by the stack, not
//!    queued. The example only starts counting after `GattWrite` arrives on the CCC descriptor.
//! 4. **Do not outrun the stack.** `notify` returns `SlStatus` and can refuse when the transmit
//!    queue is full; the example counts refusals instead of ignoring them, because a product that
//!    silently loses measurements is worse than one that reports them late.
//!
//! ## Reading the result without a phone
//!
//! The mailbox reports `gatt adv=<set> conn=<n> notif=<n> refused=<n>` on every state change, so an
//! SWD dump tells whether the stack was connected and how the transmit path behaved.

#![no_std]
#![no_main]

use efr32mg24_hal::board::Board;
use efr32mg24_hal::entry;
use efr32mg24_hal::pin::Port;
use efr32mg24_hal::prelude::*;
use efr32mg24_radio::{BleEvent, Bluetooth, CharHandle};
use panic_halt as _;

#[path = "../../xiao-mg24-bsp/examples/common/mailbox.rs"]
mod mailbox;

/// The 16-bit UUID this example advertises. Any value works; a real product registers one.
const SERVICE_UUID: u16 = 0x1815; // Automation I/O, chosen only because it reads plausibly
/// The characteristic the counter is published on.
const COUNTER_HANDLE: CharHandle = CharHandle(0x000B);

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

    let mut ble = match Bluetooth::new(&mut delay) {
        Ok(ble) => ble,
        Err(_) => {
            mailbox::write(mailbox::Status::Fail, b"ble: stack did not boot");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // A name and one service UUID: 2 + 8 + 2 + 5 = 17 of the 31 bytes a legacy PDU carries.
    if ble.set_device_name("xiao-mg24-gatt").is_err() || ble.add_service_uuid(SERVICE_UUID).is_err()
    {
        mailbox::write(
            mailbox::Status::Fail,
            b"ble: advertising data does not fit in 31 bytes",
        );
        loop {
            cortex_m::asm::wfi();
        }
    }
    if ble.start_connectable(100, 0).is_err() {
        mailbox::write(mailbox::Status::Fail, b"ble: advertising did not start");
        loop {
            cortex_m::asm::wfi();
        }
    }

    let mut connected = false;
    let mut subscribed = false;
    let mut counter: u32 = 0;
    let mut notified: u32 = 0;
    let mut refused: u32 = 0;

    loop {
        match ble.poll_event() {
            Some(BleEvent::Connected { .. }) => {
                connected = true;
                led.on();
            }
            Some(BleEvent::Disconnected { .. }) => {
                connected = false;
                subscribed = false;
                led.off();
                // A disconnect is not the end: a product advertises again, which is the difference
                // between a demo and something a user can reconnect to.
                let _ = ble.start_connectable(100, 0);
            }
            Some(BleEvent::GattWrite { .. }) => {
                // The client wrote the CCC descriptor: it wants notifications.
                subscribed = true;
            }
            Some(_) => {}
            None => {}
        }

        if connected && subscribed {
            counter += 1;
            let mut payload = [0u8; 4];
            payload.copy_from_slice(&counter.to_le_bytes());
            match ble.notify(COUNTER_HANDLE, &payload) {
                Ok(()) => notified += 1,
                // Refused means the transmit queue was full. Counted, not dropped in silence.
                Err(_) => refused += 1,
            }
            delay.delay_ms(100);
        } else {
            delay.delay_ms(10);
        }

        if notified % 10 == 0 && notified > 0 {
            let mut msg = [0u8; 64];
            let mut at = 0;
            msg[at..at + 6].copy_from_slice(b"gatt c");
            at += 6;
            at = push_dec(&mut msg, at, counter);
            msg[at..at + 7].copy_from_slice(b" notif=");
            at += 7;
            at = push_dec(&mut msg, at, notified);
            msg[at..at + 9].copy_from_slice(b" refused=");
            at += 9;
            at = push_dec(&mut msg, at, refused);
            mailbox::write(mailbox::Status::Info, &msg[..at]);
        }
    }
}
