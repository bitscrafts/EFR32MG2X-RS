//! fall-detector: a small project that watches the IMU and reports falls.
//!
//! This is written as a **project** rather than a one-file example because the
//! whole application is here and none of it is hardware plumbing:
//!
//! ```text
//!   main.rs       wiring only: board, clock, sampling loop, event report
//!   app.rs        the application: sampling policy, counters, self tests
//!   ui.rs         what the panel shows
//!   recorded.rs   320 real samples recorded from this board (the negative case)
//! ```
//!
//! The classifier itself is the `motion-detection` crate — hardware free, host
//! tested — because deciding that 2.5 g is an impact is product policy and does not
//! belong in a driver.
//!
//! ## Run
//!
//! ```text
//! cargo build --release -p efr32mg24-hal --features rt --example fall-detector
//! scripts/flash-lnx.sh <elf> --confirm
//! ```
//!
//! At boot it prints one mailbox line with the verdict of two self tests (real
//! handling data must not raise a fall; a constructed drop must), then watches:
//!
//! ```text
//! fall selftest real=clean synth=detected
//! ```
//!
//! On the panel: falls, impacts, state, peak and the live magnitude. The LED is on
//! while the device is in free fall.

#![no_std]
#![no_main]

use core::cell::RefCell;

use efr32mg24_hal::board::Board;
use efr32mg24_hal::bus::SharedBus;
use efr32mg24_hal::display::Ssd1306;
use efr32mg24_hal::entry;
use efr32mg24_hal::sensors::lsm6ds3trc::{AccelOdr, AccelRange, Config};
use efr32mg24_hal::sensors::Lsm6ds3trc;
use panic_halt as _;

mod app;
mod recorded;
mod ui;

#[path = "../common/mailbox.rs"]
mod mailbox;

/// 200 Hz at ~19 MHz SYSCLK. The accelerometer runs at 416 Hz, so nothing is
/// missed between samples that matters for a fall.
const SAMPLE_PERIOD_CYCLES: u32 = 95_000;
/// Milliseconds between samples, handed to the application.
const DT_MS: u32 = 5;

fn cycle_count() -> u32 {
    unsafe { (*cortex_m::peripheral::DWT::PTR).cyccnt.read() }
}

fn dwt_start() {
    let mut core = unsafe { cortex_m::Peripherals::steal() };
    core.DCB.enable_trace();
    core.DWT.enable_cycle_counter();
}

fn push_bytes(buf: &mut [u8], mut pos: usize, s: &[u8]) -> usize {
    for b in s {
        if pos < buf.len() {
            buf[pos] = *b;
            pos += 1;
        }
    }
    pos
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

    // Wiring: the IMU power gate is a board fact, not a driver fact.
    board.sensors_power().enable();
    cortex_m::asm::delay(4_000_000);

    let imu_bus = RefCell::new(board.imu_i2c());
    let config = Config {
        range: AccelRange::G8,
        odr: AccelOdr::Hz416,
    };
    let mut imu = match Lsm6ds3trc::with_config(SharedBus::new(&imu_bus), config) {
        Ok(i) => i,
        Err(_) => {
            mailbox::fail(b"fall-detector: imu init failed");
            loop {
                led.toggle();
                cortex_m::asm::delay(2_000_000);
            }
        }
    };

    let base_bus = RefCell::new(board.i2c());
    let mut display = Ssd1306::new(SharedBus::new(&base_bus));

    // ---- self tests, before the live loop --------------------------------
    let mut application = app::App::new();
    let verdict = application.self_test();
    let mut msg = [0u8; 64];
    let mut pos = push_bytes(&mut msg, 0, b"fall selftest real=");
    pos = push_bytes(
        &mut msg,
        pos,
        if verdict.real_data_clean {
            b"clean"
        } else {
            b"FALSEPOS"
        },
    );
    pos = push_bytes(&mut msg, pos, b" synth=");
    pos = push_bytes(
        &mut msg,
        pos,
        if verdict.synthetic_detected {
            b"detected"
        } else {
            b"MISSED"
        },
    );
    if verdict.passed() {
        mailbox::pass(&msg[..pos]);
    } else {
        mailbox::fail(&msg[..pos]);
    }

    dwt_start();

    let mut last_cycles = 0u32;
    let mut samples = 0u32;
    let mut in_air = false;

    loop {
        let now = cycle_count();
        if now.wrapping_sub(last_cycles) < SAMPLE_PERIOD_CYCLES {
            continue;
        }
        last_cycles = now;

        let Ok(motion) = imu.read() else {
            continue;
        };
        samples += 1;
        application.sample(motion.accel_mg, DT_MS);

        // The panel is slow; five times a second is enough to be read.
        if samples % 40 == 0 {
            let status = application.status();
            ui::render(&mut display, &status);
        }

        // The LED is the state that matters at a glance.
        let air = application.status().in_free_fall;
        if air != in_air {
            in_air = air;
            if air {
                led.on();
            } else {
                led.off();
            }
        }
    }
}
