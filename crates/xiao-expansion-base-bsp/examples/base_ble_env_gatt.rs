//! The BME680 on a phone: an Environmental Sensing service with the standard characteristics.
//!
//! Every other carrier example puts the readings in an advertising payload, which needs a decoder and shows a phone a
//! row of hex. This one publishes the same numbers as **standard GATT characteristics**:
//!
//! ```text
//! service  0x181A  Environmental Sensing
//!   0x2A6E  Temperature   sint16, 0.01 °C
//!   0x2A6F  Humidity      uint16, 0.01 %
//!   0x2A6D  Pressure      uint32, 0.1 Pa
//! ```
//!
//! Those UUIDs are why this works without an app: nRF Connect (and every other generic explorer) knows them, labels
//! them, and applies the units. A custom 128-bit characteristic would show the same bytes as hex and the reader would
//! have to know the encoding -- which is fine for a developer and useless for the person holding the phone.
//!
//! ## Why these are read *and* notify
//!
//! Read lets a client that connects once get the current values immediately; notify lets a client that stays connected
//! watch them change without polling. A fixed-length value with both properties is the shape the Environmental Sensing
//! service specifies, so a generic client does the right thing without being told.
//!
//! ## The rate
//!
//! One measurement every two seconds. The BME68x can go faster; a phone screen cannot usefully show more, and every
//! notification costs a connection event.

#![no_std]
#![no_main]

use efr32mg24_radio::gatt_db::{self, Characteristic16, PROP_NOTIFY, PROP_READ};
use panic_halt as _;
use xiao_expansion_base_bsp::hal::prelude::*;
use xiao_expansion_base_bsp::{entry, ExpansionBase};

#[path = "common/mailbox.rs"]
mod mailbox;

/// `0x181A` -- Environmental Sensing.
const SERVICE_ENVIRONMENTAL_SENSING: u16 = 0x181A;
/// `0x2A6E` -- Temperature, sint16 in hundredths of a degree Celsius.
const CHAR_TEMPERATURE: u16 = 0x2A6E;
/// `0x2A6F` -- Humidity, uint16 in hundredths of a percent.
const CHAR_HUMIDITY: u16 = 0x2A6F;
/// `0x2A6D` -- Pressure, uint32 in tenths of a pascal.
const CHAR_PRESSURE: u16 = 0x2A6D;

/// How often the sensor is read and the characteristics updated.
const PERIOD_MS: u32 = 2000;

#[entry]
fn main() -> ! {
    let mut board = ExpansionBase::take().expect("peripherals taken twice");
    let mut delay = board.delay();
    let led = board.led();

    // **The sensor first.** If the carrier is not fitted there is nothing to publish, and a board that advertises an
    // empty service is worse than one that says why it cannot.
    let mut environment = match board.environment() {
        Ok(sensor) => sensor,
        Err(_) => {
            mailbox::write(
                mailbox::Status::Fail,
                b"env gatt: no BME68x (is the carrier fitted?)",
            );
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let mut ble = match efr32mg24_radio::Bluetooth::new(&mut delay) {
        Ok(ble) => ble,
        Err(_) => {
            mailbox::write(mailbox::Status::Fail, b"env gatt: stack did not boot");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // The service: three standard characteristics, each read *and* notify, each with a stack-owned value buffer so a
    // read is answered by the stack and does not depend on this loop being awake at that moment.
    let characteristics = [
        Characteristic16 {
            uuid: CHAR_TEMPERATURE,
            properties: PROP_READ | PROP_NOTIFY,
            max_len: 2,
        },
        Characteristic16 {
            uuid: CHAR_HUMIDITY,
            properties: PROP_READ | PROP_NOTIFY,
            max_len: 2,
        },
        Characteristic16 {
            uuid: CHAR_PRESSURE,
            properties: PROP_READ | PROP_NOTIFY,
            max_len: 4,
        },
    ];

    let service = match gatt_db::add_service_uuid16(SERVICE_ENVIRONMENTAL_SENSING, &characteristics)
    {
        Ok(handles) => handles,
        Err(error) => {
            let mut msg = [0u8; 40];
            let mut at = 0;
            at = push(&mut msg, at, b"env gatt: service ");
            at = push_dec(&mut msg, at, u32::from(error.to_u16()));
            mailbox::write(mailbox::Status::Fail, &msg[..at]);
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let temperature = service.characteristic(0);
    let humidity = service.characteristic(1);
    let pressure = service.characteristic(2);
    if temperature.is_none() || humidity.is_none() || pressure.is_none() {
        mailbox::write(mailbox::Status::Fail, b"env gatt: incomplete service");
        loop {
            cortex_m::asm::wfi();
        }
    }

    // Name and the service UUID in the advertising packet, so a phone that filters by service finds this board and a
    // phone that does not still sees a name it recognises.
    let _ = ble.set_device_name("xiao-mg24-env");
    // **Connectable, not merely visible.** `start_advertising` sends a non-connectable PDU: a phone sees the name,
    // tries to connect, and the attempt never completes. The first version of this example did that and the symptom was
    // a central stuck at "Attempting to connect" with nothing arriving -- not an error, which is why it read as a host
    // problem for longer than it should have.
    if ble.start_connectable(200, 0).is_err() {
        mailbox::write(mailbox::Status::Fail, b"env gatt: advertising refused");
        loop {
            cortex_m::asm::wfi();
        }
    }

    // The handles the stack assigned, in the record, **before** the radio starts: a database that built but serves
    // nothing and a database that never built look identical from the client side, and if anything after this point
    // fails the numbers are still there to read.
    let mut msg = [0u8; 64];
    let mut at = 0;
    at = push(&mut msg, at, b"env gatt: svc=");
    at = push_dec(&mut msg, at, u32::from(service.service));
    at = push(&mut msg, at, b" h0=");
    at = push_dec(
        &mut msg,
        at,
        temperature.map_or(0, |h| u32::from(h.value())),
    );
    at = push(&mut msg, at, b" h1=");
    at = push_dec(&mut msg, at, humidity.map_or(0, |h| u32::from(h.value())));
    at = push(&mut msg, at, b" h2=");
    at = push_dec(&mut msg, at, pressure.map_or(0, |h| u32::from(h.value())));
    at = push(&mut msg, at, b" adv");
    mailbox::write(mailbox::Status::Info, &msg[..at]);

    let mut updates: u32 = 0;
    let mut connections: u32 = 0;
    loop {
        // **Re-advertise after a disconnect.** A connectable advertiser stops when a central connects, and an example
        // that does not restart it is reachable exactly once -- which is a product that works for the first phone and
        // never again. This was measured the hard way: after one successful connection the board went silent and every
        // later attempt failed, while the mailbox still said "advertising".
        while let Some(event) = ble.poll_event() {
            match event {
                efr32mg24_radio::BleEvent::Connected { .. } => {
                    connections += 1;
                    mailbox::write(mailbox::Status::Info, b"env gatt: client connected");
                }
                efr32mg24_radio::BleEvent::Disconnected { .. } => {
                    let _ = ble.start_connectable(200, 0);
                    mailbox::write(
                        mailbox::Status::Info,
                        b"env gatt: disconnected, advertising again",
                    );
                }
                _ => {}
            }
        }

        let reading = match environment.read_with(|| delay.delay_ms(1)) {
            Ok(reading) => reading,
            Err(_) => {
                delay.delay_ms(PERIOD_MS);
                continue;
            }
        };

        // Fixed point in the units the standard characteristics specify: 0.01 °C, 0.01 %, 0.1 Pa. The conversion is
        // here, once, rather than in whoever reads it.
        let temperature_value = (reading.temperature_centi_c as i16).to_le_bytes();
        let humidity_raw = reading.humidity_centi_pct.clamp(0, 10_000) as u16;
        let humidity_value = humidity_raw.to_le_bytes();
        let pressure_value = ((reading.pressure_hpa as i64) * 100).clamp(0, u32::MAX as i64) as u32;
        let pressure_value = pressure_value.to_le_bytes();

        // Two steps per characteristic, and both are needed: `write_attribute` updates the value a *read* returns,
        // `notify` sends it to a subscriber. Doing only one leaves half the clients blind.
        for (handle, value) in [
            (temperature, &temperature_value[..]),
            (humidity, &humidity_value[..]),
            (pressure, &pressure_value[..]),
        ] {
            if let Some(handle) = handle {
                let _ = ble.write_attribute(handle, value);
                let _ = ble.notify(handle, value);
            }
        }

        updates += 1;
        if updates % 15 == 0 {
            let mut msg = [0u8; 64];
            let mut at = 0;
            at = push(&mut msg, at, b"env gatt: t=");
            at = push_dec(&mut msg, at, reading.temperature_centi_c.unsigned_abs());
            at = push(&mut msg, at, b"c p=");
            at = push_dec(&mut msg, at, reading.pressure_hpa.unsigned_abs());
            at = push(&mut msg, at, b"hpa rh=");
            at = push_dec(&mut msg, at, humidity_raw as u32);
            at = push(&mut msg, at, b" n=");
            at = push_dec(&mut msg, at, updates);
            at = push(&mut msg, at, b" c=");
            at = push_dec(&mut msg, at, connections);
            mailbox::write(mailbox::Status::Info, &msg[..at]);
        }

        led.toggle();
        delay.delay_ms(PERIOD_MS);
    }
}

/// Append a literal and advance past it, deriving the length from the literal.
fn push(msg: &mut [u8], at: usize, literal: &[u8]) -> usize {
    msg[at..at + literal.len()].copy_from_slice(literal);
    at + literal.len()
}

/// Append a decimal number and advance past it.
fn push_dec(msg: &mut [u8], at: usize, mut value: u32) -> usize {
    let mut digits = [0u8; 10];
    let mut n = 0;
    if value == 0 {
        msg[at] = b'0';
        return at + 1;
    }
    while value > 0 {
        digits[n] = b'0' + (value % 10) as u8;
        value /= 10;
        n += 1;
    }
    let mut at = at;
    for i in (0..n).rev() {
        msg[at] = digits[i];
        at += 1;
    }
    at
}
