//! fall-detector: a small project that watches the IMU and reports falls.
//!
//! This is written as a **project** rather than a one-file example because the
//! whole application is here and none of it is hardware plumbing:
//!
//! ```text
//!   main.rs       wiring only: board, clock, sampling loop, event report
//!   recording.rs  the 15 s ring buffer, so the demo run is also a dataset
//!   app.rs        the application: sampling policy, counters, self tests
//!   ui.rs         what the panel shows
//!   recorded.rs   320 real samples of handling (the negative case)
//!   falls.rs      2004 real samples containing two genuine drops (the positive case)
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
use efr32mg24_hal::bus::Bus;
use efr32mg24_hal::bus::SharedBus;
use efr32mg24_hal::display::Ssd1306;
use efr32mg24_hal::entry;
use efr32mg24_hal::pin::Port;
use efr32mg24_hal::sensors::lsm6ds3trc::{AccelOdr, AccelRange, Config};
use efr32mg24_hal::sensors::Lsm6ds3trc;
use motion_detection::Event;
use panic_halt as _;

mod app;
mod events;
mod falls;
mod falls_400hz;
mod recorded;
mod recording;
mod ui;

#[path = "../common/mailbox.rs"]
mod mailbox;

/// 400 Hz at ~19 MHz SYSCLK, just under the sensor's 416 Hz output rate.
///
/// Measured at 200 Hz, two of five real drops showed the impact but **no free fall**:
/// a 20–30 ms free fall falls between samples. Halving the period halves that blind
/// spot, which is the difference between reporting a drop and reporting a knock.
const SAMPLE_PERIOD_CYCLES: u32 = 47_500;
/// The shortest interval the loop aims for: 400 Hz, just under the sensor's 416 Hz.
///
/// It is a floor, not a promise. The measured interval is what the detector and the
/// recording get, because the display flush alone blocks for tens of milliseconds.
const DT_US: u32 = 2_500;

/// CPU cycles per microsecond at the frequency the clock module selects.
const CYCLES_PER_US: u32 = 19;

/// How often the panel is refreshed, in samples. The flush is 1090 bytes on a bit-banged
/// bus — about 100 ms of blocked CPU — so refreshing every sample would cap the loop at a
/// handful of hertz. Two seconds costs 5 % of the loop and nobody looks at it faster.
const DISPLAY_EVERY: u32 = 800;

/// How long after an event the panel is left alone: a flush in the middle of a fall would
/// stall the sampler exactly when the samples matter most.
const DISPLAY_QUIET_MS: u32 = 1_200;

fn cycle_count() -> u32 {
    efr32mg24_hal::dwt::cycles()
}

fn dwt_start() {
    // SAFETY: nothing else in this project uses the cycle counter.
    unsafe { efr32mg24_hal::dwt::start() };
}

fn push_hex8(buf: &mut [u8], mut pos: usize, value: u32) -> usize {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for shift in (0..8).rev() {
        if pos < buf.len() {
            buf[pos] = HEX[((value >> (shift * 4)) & 0xF) as usize];
            pos += 1;
        }
    }
    pos
}

fn push_dec(buf: &mut [u8], mut pos: usize, mut value: u32) -> usize {
    let mut digits = [0u8; 10];
    let mut n = 0;
    if value == 0 {
        digits[0] = b'0';
        n = 1;
    }
    while value > 0 {
        digits[n] = b'0' + (value % 10) as u8;
        value /= 10;
        n += 1;
    }
    while n > 0 {
        n -= 1;
        if pos < buf.len() {
            buf[pos] = digits[n];
            pos += 1;
        }
    }
    pos
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
    let led = board.output_on(Port::A, 7, true);

    // Wiring: the IMU power gate is a board fact, not a driver fact.
    board.output_on(Port::D, 5, false).on();
    cortex_m::asm::delay(4_000_000);

    let imu_bus = RefCell::new(board.i2c_on(Bus::new(Port::B, 1 << 2, 1 << 3)));
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

    let base_bus = RefCell::new(board.i2c_on(Bus::new(Port::C, 1 << 4, 1 << 5)));
    let mut display = Ssd1306::new(SharedBus::new(&base_bus));

    // An alarm you can hear: the panel shows the count, the buzzer says it happened.
    let mut buzzer = board.buzzer_on(Port::C, 3);
    let mut alarm = app::Alarm::new();

    // ---- self tests, before the live loop --------------------------------
    let mut application = app::App::new();
    let verdict = application.self_test();
    let mut msg = [0u8; 64];
    let mut pos = push_bytes(&mut msg, 0, b"cap=0x");
    pos = push_hex8(&mut msg, pos, recording::base_address());
    pos = push_bytes(&mut msg, pos, b" real=");
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
            b"ok"
        } else {
            b"MISS"
        },
    );
    pos = push_bytes(&mut msg, pos, b" 200Hz=");
    pos = push_dec(&mut msg, pos, verdict.real_drops_detected);
    pos = push_bytes(&mut msg, pos, b"/2 400Hz=");
    pos = push_dec(&mut msg, pos, verdict.drops_400hz_detected);
    pos = push_bytes(&mut msg, pos, b"/4");
    if verdict.passed() {
        mailbox::pass(&msg[..pos]);
    } else {
        mailbox::fail(&msg[..pos]);
    }

    dwt_start();

    let mut last_cycles = 0u32;
    let mut last_sample_cycles = 0u32;
    let mut samples = 0u32;
    let mut uptime_ms = 0u32;
    let mut last_event_ms = 0u32;
    let mut in_air = false;
    let mut ring_index = 0usize;
    let mut last_falls = 0u32;

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
        // The interval that actually elapsed, measured with the same cycle counter that
        // paces the loop. Assuming DT_US here is what made the timers run 2.5x fast.
        let dt_us = now.wrapping_sub(last_sample_cycles) / CYCLES_PER_US;
        last_sample_cycles = now;
        uptime_ms = uptime_ms.wrapping_add(dt_us / 1000);
        // Log what the classifier decided, in RAM, where a later probe read can still
        // find it: the waveform ring only remembers ten seconds, and a test performed by
        // hand does not care when the host is ready.
        match application.sample_us(motion.accel_mg, dt_us) {
            Some(Event::FreeFall { ms }) => {
                last_event_ms = uptime_ms;
                events::push(uptime_ms, events::Kind::FreeFall, ms as u16, 0);
            }
            Some(Event::Impact { peak_mg }) => {
                last_event_ms = uptime_ms;
                events::push(uptime_ms, events::Kind::Impact, 0, peak_mg as u16);
            }
            Some(Event::Fall {
                peak_mg,
                freefall_ms,
            }) => {
                last_event_ms = uptime_ms;
                events::push(
                    uptime_ms,
                    events::Kind::Fall,
                    freefall_ms as u16,
                    peak_mg as u16,
                );
            }
            None => {}
        }

        // A detected fall starts the alarm; the pattern advances one sample at a time
        // so classification never pauses.
        // Trigger on the fall *count* rather than on the last event: two falls in a
        // row must sound twice, and `last_event` would stay `Fall` the whole time.
        let falls = application.status().falls;
        if falls != last_falls {
            last_falls = falls;
            alarm.trigger();
        }

        let sounding = alarm.tick(dt_us);
        if let Some(b) = buzzer.as_mut() {
            if sounding {
                b.sound();
            } else {
                b.silence();
            }
        }

        // Every classified sample is also recorded: this is what makes the demo run
        // usable as training data afterwards.
        recording::push_timed(motion.accel_mg, ring_index, dt_us);
        ring_index += 1;
        if ring_index == recording::CAPTURE_LEN {
            ring_index = 0;
        }

        // The panel is slow; five times a second is enough to be read.
        let quiet = uptime_ms.wrapping_sub(last_event_ms) > DISPLAY_QUIET_MS;
        if samples % DISPLAY_EVERY == 0 && quiet && !application.status().in_free_fall {
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
