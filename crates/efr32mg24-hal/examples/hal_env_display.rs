//! Environment display: temperature, pressure and humidity from the BME68x on
//! the Expansion Base, drawn on the base's OLED — **through the HAL only**.
//!
//! This is the HAL twin of `efr32mg24-pac/examples/pac_env_display.rs`, and the
//! pair is the differential test: both write the same values to the same
//! mailbox, so any disagreement between them is a HAL bug, not a sensor quirk.
//!
//! The example names no PAC type. Everything it touches comes from
//! `efr32mg24_hal`:
//!
//! - [`Board`] — peripherals plus the bus clocks already enabled,
//! - [`Ssd1306`] — the panel driver, generic over the I2C transport,
//! - [`Bme68x`] — the sensor driver with the Bosch compensation inside.
//!
//! Verification, as in the PAC twin: the mailbox carries the readings and the
//! number of bytes the panel acknowledged. The pixels need an eye; the numbers
//! do not.

#![no_std]
#![no_main]

use core::cell::RefCell;

use efr32mg24_hal::board::Board;
use efr32mg24_hal::bus::SharedBus;
use efr32mg24_hal::display::{append, format_fixed, Ssd1306};
use efr32mg24_hal::entry;
use efr32mg24_hal::sensors::Bme68x;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Pump the sensor while it converts. The driver does not own a timer, so the
/// caller supplies the delay — here a plain NOP loop, because the refresh rate
/// does not matter for a display.
fn pump() {
    cortex_m::asm::delay(200_000);
}

#[entry]
fn main() -> ! {
    let board = match Board::take() {
        Some(b) => b,
        None => loop {
            cortex_m::asm::nop();
        },
    };
    let led = board.led();
    // One pair of wires, two drivers: the cell is shared, borrows are checked at
    // run time and the drivers never nest them.
    let bus = RefCell::new(board.i2c());

    let mut oled = match Ssd1306::new(SharedBus::new(&bus)) {
        Some(o) => o,
        None => {
            mailbox::fail(b"oled ack=0 (nothing at 0x3C)");
            loop {
                led.toggle();
                cortex_m::asm::delay(2_000_000);
            }
        }
    };

    let mut sensor = match Bme68x::new(SharedBus::new(&bus)) {
        Ok(s) => s,
        Err(_) => {
            oled.clear();
            oled.text(0, 0, b"SENSOR ERROR");
            oled.text(0, 3, b"CHECK 0X76");
            oled.flush();
            mailbox::fail(b"bme68x not detected");
            loop {
                led.toggle();
                cortex_m::asm::delay(2_000_000);
            }
        }
    };

    loop {
        match sensor.read_with(pump) {
            Ok(r) => {
                let mut l0 = [0u8; 20];
                let n0 = append(&mut l0, 0, b"TEMP ");
                let n0 = n0 + format_fixed(&mut l0[n0..], r.temperature_centi_c, 1);

                let mut l1 = [0u8; 20];
                let n1 = append(&mut l1, 0, b"PRES ");
                let n1 = n1 + format_fixed(&mut l1[n1..], r.pressure_hpa * 100, 0);
                let n1 = append(&mut l1, n1, b" HP");

                let mut l2 = [0u8; 20];
                let n2 = append(&mut l2, 0, b"HUM ");
                let n2 = n2 + format_fixed(&mut l2[n2..], r.humidity_centi_pct, 1);
                let n2 = append(&mut l2, n2, b" %");

                oled.clear();
                // Pages 0, 3 and 6: one blank page between lines.
                oled.text(0, 0, &l0[..n0]);
                oled.text(0, 3, &l1[..n1]);
                oled.text(0, 6, &l2[..n2]);
                oled.flush();

                let mut msg = [0u8; 64];
                let mut pos = append(&mut msg, 0, b"hal T=");
                pos += format_fixed(&mut msg[pos..], r.temperature_centi_c, 2);
                pos = append(&mut msg, pos, b"C P=");
                pos += format_fixed(&mut msg[pos..], r.pressure_hpa * 100, 0);
                pos = append(&mut msg, pos, b"hPa H=");
                pos += format_fixed(&mut msg[pos..], r.humidity_centi_pct, 1);
                pos = append(&mut msg, pos, b"% d=");
                pos += format_fixed(&mut msg[pos..], oled.acked as i32, 0);

                let plausible = (-2000..=6000).contains(&r.temperature_centi_c)
                    && (800..=1100).contains(&r.pressure_hpa)
                    && (0..=10000).contains(&r.humidity_centi_pct);
                if plausible && oled.acked > 500 {
                    mailbox::pass(&msg[..pos]);
                } else {
                    mailbox::fail(&msg[..pos]);
                }
            }
            Err(_) => mailbox::fail(b"hal sensor read failed"),
        }

        // ~2 s between refreshes, with the LED blinking as a liveness signal.
        for _ in 0..4 {
            led.toggle();
            cortex_m::asm::delay(2_500_000);
        }
    }
}
