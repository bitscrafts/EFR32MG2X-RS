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

/// The 128-bit UUID of the GATT service this firmware actually serves.
///
/// The advertised `SERVICE_UUID` is the short form a scanner shows; the service a client can *use* is this one, built
/// at run time with `sl_bt_gattdb_*`. A client connected to the previous version of this example listed zero
/// characteristics, because the counter was published on `CharHandle(0x000B)` -- a handle that no attribute ever
/// occupied. The handle is now whatever the database returns.
const SERVICE_UUID_128: [u8; 16] = [
    0x9e, 0xca, 0xdc, 0x24, 0x0e, 0xe5, 0xa9, 0xe0, 0x93, 0xf3, 0xa3, 0xb5, 0x01, 0x00, 0x40, 0x6e,
];

/// The control characteristic: a client writes four bytes here and reads them back. It exists so the *write* path is
/// part of this example rather than an assumption: a service that can only notify proves half of GATT.
const CONTROL_UUID_128: [u8; 16] = [
    0x9e, 0xca, 0xdc, 0x24, 0x0e, 0xe5, 0xa9, 0xe0, 0x93, 0xf3, 0xa3, 0xb5, 0x03, 0x00, 0x40, 0x6e,
];

/// The counter characteristic: readable, and notifiable so a client can watch it move.
const COUNTER_UUID_128: [u8; 16] = [
    0x9e, 0xca, 0xdc, 0x24, 0x0e, 0xe5, 0xa9, 0xe0, 0x93, 0xf3, 0xa3, 0xb5, 0x02, 0x00, 0x40, 0x6e,
];

/// SYSRTC's counter, the one timer this board has been measured to run (32 968 Hz). The DWT cycle counter reads zero
/// here, so a duration measured in cycles is a duration measured in nothing.
fn sysrtc_ticks() -> u32 {
    // SAFETY: read-only counter at the SYSRTC base the HAL and RAIL both use.
    unsafe { core::ptr::read_volatile(0x500a_8018 as *const u32) }
}

/// Two hex digits, lowercase. Small enough to inline and it keeps the write report readable as raw bytes.
fn push_hex_byte(out: &mut [u8], at: usize, byte: u8) -> usize {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    out[at] = HEX[(byte >> 4) as usize];
    out[at + 1] = HEX[(byte & 0xF) as usize];
    at + 2
}

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
        Err(e) => {
            let mut msg = [0u8; 32];
            let mut at = 0;
            msg[at..at + 12].copy_from_slice(b"ble: boot st");
            at += 12;
            at = push_dec(&mut msg, at, u32::from(e.to_u16()));
            mailbox::write(mailbox::Status::Fail, &msg[..at]);
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // Build what this device serves, before advertising it. A service that is advertised but never added is a device
    // a client connects to and finds empty -- which is what the first version of this example did.
    //
    // **A missing database is not a reason to stop advertising.** The dynamic GATT database component cannot be
    // enabled yet (see `config/sl_component_catalog.h`: the stack answers `SL_STATUS_TIMEOUT` at boot when it is), so
    // this example publishes what happened and keeps going. The alternative -- refusing to run -- would hide a working
    // connection and advertising path behind a component that is missing for an unrelated reason, and the mailbox
    // would say only "failed".
    use efr32mg24_bluetooth_sys::gatt_db::{Characteristic, PROP_NOTIFY, PROP_READ, PROP_WRITE};
    let (counter_handle, control_handle) = match efr32mg24_bluetooth_sys::gatt_db::add_service(
        SERVICE_UUID_128,
        &[
            // Four bytes of counter, kept by the stack: a client can read it and subscribe to it.
            Characteristic {
                uuid: COUNTER_UUID_128,
                properties: PROP_READ | PROP_NOTIFY,
                // Eight, not four: the notification carries the counter *and* the value a client wrote, and a
                // notification larger than the attribute's maximum length is truncated by the stack -- which is one
                // of the ways an echo test can look like a write that never arrived.
                max_len: 8,
            },
            // Four bytes a client owns: writing them here is what proves the write path, and reading them back is
            // what proves the write reached an attribute rather than a callback.
            Characteristic {
                uuid: CONTROL_UUID_128,
                properties: PROP_READ | PROP_WRITE,
                max_len: 4,
            },
        ],
    ) {
        Ok(handles) => (handles.characteristic(0), handles.characteristic(1)),
        Err(e) => {
            // Publish the code, not just the fact. The number is the stack's own status, so a reader can look it up:
            // 15 is `SL_STATUS_NOT_SUPPORTED`, which names the missing component exactly.
            let mut msg = [0u8; 40];
            let mut at = 0;
            // `b"gatt: no attrs s"` is sixteen bytes; asking for fifteen panics, and a panic here leaves the mailbox
            // holding the *previous* run's record, which reads as "the new firmware did nothing".
            msg[at..at + 16].copy_from_slice(b"gatt: no attrs s");
            at += 16;
            at = push_dec(&mut msg, at, u32::from(e.to_u16()));
            mailbox::write(mailbox::Status::Info, &msg[..at]);
            (None, None)
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

    // Say so, once, before waiting. Everything after this point is driven by a central: without a marker, "advertising
    // and waiting for a connection" and "hung before the loop" leave the mailbox holding whatever the previous run
    // wrote, and the two are indistinguishable from outside. Measured: the first run of this example wrote nothing at
    // all, and the only way to tell it had succeeded was to read the code.
    // **Time the delay provider, before anything else can be blamed for the cadence.** The loop paces itself with
    // `delay_ms(100)` and manages three notifications where a hundred were expected; if the provider is the reason,
    // the ratio below shows it in one record, and if it is not, the delay is eliminated in one measurement.
    // Behind `diag`: this measures the delay provider against SYSRTC at boot, which is worth 110 ms when somebody is
    // chasing a cadence and worth nothing in a firmware that ships. The numbers it produced are in BLE-STATUS.md.
    #[cfg(feature = "diag")]
    let (d10ms, d100ms) = {
        let t_a = sysrtc_ticks();
        delay.delay_ms(10);
        let b = sysrtc_ticks();
        delay.delay_ms(100);
        (b.wrapping_sub(t_a), sysrtc_ticks().wrapping_sub(b))
    };
    #[cfg(not(feature = "diag"))]
    let (d10ms, d100ms) = (0u32, 0u32);

    // One record, both facts: the advertising state and whether there is anything to serve. Written as two records
    // the second overwrote the first, and a reader saw "advertising, waiting" with no hint that the device had no
    // attributes -- which is the defect this example exists to expose.
    {
        let mut msg = [0u8; 96];
        let mut at = 0;
        msg[at..at + 20].copy_from_slice(b"gatt: adv waiting a=");
        at += 20;
        at = push_dec(&mut msg, at, if counter_handle.is_some() { 1 } else { 0 });
        #[cfg(feature = "diag")]
        {
            msg[at..at + b" d1".len()].copy_from_slice(b" d1");
            at += b" d1".len();
            at = push_dec(&mut msg, at, d10ms);
            msg[at..at + 4].copy_from_slice(b" d10");
            at += 4;
            at = push_dec(&mut msg, at, d100ms);
        }
        #[cfg(not(feature = "diag"))]
        let _ = (d10ms, d100ms);
        mailbox::write(mailbox::Status::Info, &msg[..at]);
    }

    let mut connected = false;
    let mut subscribed = false;
    let mut counter: u32 = 0;
    let mut notified: u32 = 0;
    let mut refused: u32 = 0;
    // Subscription history. The mailbox holds one record, and a client's last act before leaving is to turn
    // notifications *off* -- so the event that proves the subscription worked was overwritten by the one that ends it.
    // These counters make the summary on disconnect the whole story: whether a client subscribed, and whether the
    // notifications it asked for actually went out.
    let mut subs_on: u32 = 0;
    let mut iterations: u32 = 0;
    let mut control_writes: u32 = 0;
    let mut control_value = [0u8; 4];
    let mut t_subscribed: u32 = 0;
    let mut t_unsubscribed: u32 = 0;
    let mut subs_off: u32 = 0;

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
                // The summary, written where a tester will look after the client has gone: what the client asked for
                // and what the board managed to send. A row of zeros here and a row of numbers are the difference
                // between "the connection worked" and "the service worked".
                {
                    // 160, not 64: this is the widest record in the tree (nine label/value pairs, two of them
                    // five-digit ticks) and the first version was 64, which panicked on the bounds check and left the
                    // mailbox holding the *previous* record -- so the summary looked like it had never been written.
                    let mut msg = [0u8; 160];
                    let mut at = 0;
                    msg[at..at + 9].copy_from_slice(b"gatt: end");
                    at += 9;
                    msg[at..at + 4].copy_from_slice(b" on=");
                    at += 4;
                    at = push_dec(&mut msg, at, subs_on);
                    msg[at..at + 4].copy_from_slice(b" of=");
                    at += 4;
                    at = push_dec(&mut msg, at, subs_off);
                    msg[at..at + 5].copy_from_slice(b" ntf=");
                    at += 5;
                    at = push_dec(&mut msg, at, notified);
                    msg[at..at + b" rfs=".len()].copy_from_slice(b" rfs=");
                    at += b" rfs=".len();
                    at = push_dec(&mut msg, at, refused);
                    msg[at..at + b" wr=".len()].copy_from_slice(b" wr=");
                    at += b" wr=".len();
                    at = push_dec(&mut msg, at, control_writes);
                    at += 5;
                    at = push_dec(&mut msg, at, refused);
                    msg[at..at + 4].copy_from_slice(b" it=");
                    at += 4;
                    at = push_dec(&mut msg, at, iterations);
                    msg[at..at + 3].copy_from_slice(b" t0");
                    at += 3;
                    at = push_dec(&mut msg, at, t_subscribed);
                    msg[at..at + 3].copy_from_slice(b" t1");
                    at += 3;
                    at = push_dec(&mut msg, at, t_unsubscribed);
                    let (calls, cycles) = efr32mg24_bluetooth_sys::stack_init::pump_cost();
                    msg[at..at + 4].copy_from_slice(b" pc=");
                    at += 4;
                    at = push_dec(&mut msg, at, calls);
                    msg[at..at + 5].copy_from_slice(b" pcy=");
                    at += 5;
                    at = push_dec(&mut msg, at, cycles);
                    mailbox::write(mailbox::Status::Info, &msg[..at]);
                }
                // A disconnect is not the end: a product advertises again, which is the difference
                // between a demo and something a user can reconnect to.
                let _ = ble.start_connectable(100, 0);
            }
            Some(BleEvent::GattWrite {
                attribute, value, ..
            }) => {
                // Two different writes arrive here: the client's own value on the control characteristic, and the
                // CCC descriptor behind the notify one. Counting them together would hide a client that subscribes
                // without ever writing anything, so they are told apart by handle.
                {
                    // Every write event, with the handle it names and how many bytes it carried. The control branch
                    // below is what the example uses; this record is what tells a reader which events the stack
                    // actually raises, and it is how an eleven-for-one count was explained rather than guessed at.
                    let mut msg = [0u8; 64];
                    let mut at = 0;
                    msg[at..at + b"gatt ev a=".len()].copy_from_slice(b"gatt ev a=");
                    at += b"gatt ev a=".len();
                    at = push_dec(&mut msg, at, u32::from(attribute));
                    msg[at..at + b" n=".len()].copy_from_slice(b" n=");
                    at += b" n=".len();
                    at = push_dec(&mut msg, at, value.len() as u32);
                    msg[at..at + b" v=".len()].copy_from_slice(b" v=");
                    at += b" v=".len();
                    for byte in value.iter().take(4) {
                        at = push_hex_byte(&mut msg, at, *byte);
                    }
                    mailbox::write(mailbox::Status::Info, &msg[..at]);
                }
                if Some(CharHandle(attribute)) == control_handle {
                    // Store it in the stack's attribute so a **read** returns what was written. A write that only
                    // lands in a local variable answers `write` and fails `read`, and only a client does both.
                    let mut stored = [0u8; 4];
                    let n = value.len().min(4);
                    stored[..n].copy_from_slice(&value[..n]);
                    control_value = stored;
                    if let Some(handle) = control_handle {
                        let _ = ble.write_attribute(handle, &stored);
                    }
                    control_writes += 1;
                    let mut msg = [0u8; 48];
                    let mut at = 0;
                    msg[at..at + 9].copy_from_slice(b"gatt: wr ");
                    at += 9;
                    for byte in stored {
                        at = push_hex_byte(&mut msg, at, byte);
                    }
                    mailbox::write(mailbox::Status::Info, &msg[..at]);
                } else {
                    // The CCC descriptor: the client wants notifications.
                    subscribed = true;
                }
            }
            Some(BleEvent::Subscribed {
                notifications,
                characteristic,
                status_flags,
                client_config,
                ..
            }) => {
                // The stack reports the subscription itself, which is a better signal than inferring it from a write:
                // it fires for a client that uses the documented CCC path, and it says whether listening was turned
                // on or off. Recorded, because it is the only trace a client leaves on the device side.
                subscribed = notifications;
                if notifications {
                    subs_on += 1;
                    t_subscribed = sysrtc_ticks();
                } else {
                    subs_off += 1;
                    t_unsubscribed = sysrtc_ticks();
                }
                let mut msg = [0u8; 48];
                let mut at = 0;
                msg[at..at + 9].copy_from_slice(b"gatt: sub");
                at += 9;
                at = push_dec(&mut msg, at, u32::from(characteristic));
                msg[at..at + 3].copy_from_slice(b" sf");
                at += 3;
                at = push_dec(&mut msg, at, u32::from(status_flags));
                msg[at..at + 3].copy_from_slice(b" cc");
                at += 3;
                at = push_dec(&mut msg, at, u32::from(client_config));
                msg[at..at + 3].copy_from_slice(if notifications { b" on" } else { b" of" });
                at += 3;
                mailbox::write(mailbox::Status::Info, &msg[..at]);
            }
            Some(_) => {}
            None => {}
        }

        iterations += 1;
        if connected && subscribed && counter_handle.is_some() {
            counter += 1;
            // **Eight bytes: the counter and the client's own value.** A client that writes four bytes and then
            // sees them arrive in a notification has proof the bytes reached the board -- a read-back alone can be
            // answered from the host's cache, which makes it a circular test.
            let mut payload = [0u8; 8];
            payload[..4].copy_from_slice(&counter.to_le_bytes());
            payload[4..].copy_from_slice(&control_value);
            match ble.notify(counter_handle.expect("guarded above"), &payload) {
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
            // What the board believes the client wrote. The only way to tell "the write arrived" from "the host
            // answered its own cache" is to have the board say the bytes out loud.
            msg[at..at + b" cv=".len()].copy_from_slice(b" cv=");
            at += b" cv=".len();
            for byte in control_value {
                at = push_hex_byte(&mut msg, at, byte);
            }
            mailbox::write(mailbox::Status::Info, &msg[..at]);
        }
    }
}
