//! A fall detector that broadcasts its verdict: the event, on the air, without a connection.
//!
//! This example is the project's two hardest pieces joined: the classifier that was calibrated on nine
//! real drops (`libraries/motion-detection`, no hardware, nine host tests) and the radio that boots from
//! three vendor blobs. Between them sits the lesson that cost a full session here — **the sample
//! interval is measured, never assumed** — because the classifier's windows are in microseconds and a
//! loop that claims 400 Hz while running at 158 poisons every threshold.
//!
//! ## The frame
//!
//! ```text
//! byte 0      frame version (0x01)
//! byte 1      flags: bit0 airborne, bit1 a fall was reported, bit2 an impact is open
//! bytes 2-3   falls since power-on, u16
//! bytes 4-5   last landing peak, milli-g, i16
//! bytes 6-7   last free-fall duration, ms, u16
//! bytes 8-9   seconds since the last fall, u16 (0xFFFF = never)
//! bytes 10-11 current |a|, milli-g, u16
//! -------------------------------------------------------
//! 12 bytes of payload, 19 of the 31 available
//! ```
//!
//! A receiver can therefore tell three things apart without connecting: nothing is happening, the
//! board is falling, or a fall was reported two seconds ago with a 3.9 g landing.
//!
//! ## Why this is advertising and not a notification
//!
//! The connectable path needs a GATT database and, with it, the bonding code and a crypto library that
//! is not vendored (see `docs/04-reference/BLE-GATT-GAP.md`). Advertising carries the same verdict with
//! none of that. For an alarm, broadcast is also the more useful topology: any number of receivers in
//! range get it, with no pairing and no state.
//!
//! ## What proves it
//!
//! Drop the board. The LED blinks three times, the mailbox reports `FALL peak=…`, and a phone running
//! nRF Connect shows the frame change — flags bit1 set, the counter incremented, the peak in bytes 4-5.
//! A second board running `ble_scanner` shows the same frame with an RSSI. Both are independent
//! receivers, which is the point: the classifier was already trusted, and this proves the radio reports
//! what the classifier decided.

#![no_std]
#![no_main]

use motion_detection::{Event, FallDetector};
use panic_halt as _;
use xiao_mg24_bsp::hal::dwt;
use xiao_mg24_bsp::hal::prelude::*;
use xiao_mg24_bsp::hal::sensors::lsm6ds3trc::{AccelOdr, AccelRange, Config, ImuError, Lsm6ds3trc};
use xiao_mg24_bsp::{entry, Module};

#[path = "common/mailbox.rs"]
mod mailbox;

const COMPANY_ID: u16 = 0x02E5;
const FRAME_VERSION: u8 = 0x01;

const FLAG_AIRBORNE: u8 = 0x01;
const FLAG_FALL: u8 = 0x02;
const FLAG_IMPACT: u8 = 0x04;

/// The loop period. The IMU is configured at 416 Hz; asking for 400 Hz leaves headroom so a late
/// iteration does not miss a sample. The *actual* interval is measured per sample and passed on.
const PERIOD_US: u32 = 2_500;

/// How long a reported event stays flagged in the advertising frame. Long enough for a receiver that
/// is watching every few hundred milliseconds to see it, short enough not to look current.
const EVENT_HOLD_MS: u32 = 5_000;

/// SYSCLK as measured on this board (see the clocks chapter): 19 MHz, the HFRCO default.
const SYSCLK_HZ: u32 = 19_000_000;

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
    let mut module = Module::take().expect("peripherals taken twice");
    let mut delay = module.delay();
    let led = module.led();

    // The cycle counter is the only time base here that cannot lie about the clock.
    // SAFETY: nothing else in this example uses DWT.
    let boot = unsafe { dwt::start() };

    module.imu_power().on();
    delay.delay_ms(4);

    let mut imu = match Lsm6ds3trc::with_config(
        module.imu_bus(),
        Config {
            range: AccelRange::G8,
            odr: AccelOdr::Hz416,
        },
    ) {
        Ok(imu) => imu,
        Err(ImuError::NotAnLsm6ds3(id)) => {
            let mut buf = [0u8; 40];
            buf[..12].copy_from_slice(b"fall ble: 0x");
            let at = push_dec(&mut buf, 11, u32::from(id));
            mailbox::write(mailbox::Status::Fail, &buf[..at]);
            loop {
                cortex_m::asm::wfi();
            }
        }
        Err(_) => {
            mailbox::write(mailbox::Status::Fail, b"fall ble: no ack on the IMU bus");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let mut ble = match efr32mg24_radio::Bluetooth::new(&mut delay) {
        Ok(ble) => ble,
        Err(_) => {
            mailbox::write(mailbox::Status::Fail, b"fall ble: stack did not boot");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let mut detector = FallDetector::new();
    let mut last_sample = dwt::cycles();

    let mut falls: u16 = 0;
    let mut last_peak_mg: i16 = 0;
    let mut last_freefall_ms: u16 = 0;
    let mut last_fall_ms: u32 = 0;
    let mut ever_fell = false;
    let mut airborne = false;
    let mut impact_open = false;
    let mut last_adv_ms: u32 = 0;

    loop {
        let now = dwt::cycles();
        let dt_us = dwt::cycles_to_us(now.wrapping_sub(last_sample), SYSCLK_HZ);
        last_sample = now;
        let uptime_ms = dwt::cycles_to_us(dwt::since(boot), SYSCLK_HZ) / 1_000;

        let mut current_mg: u16 = 0;
        if let Ok(motion) = imu.read() {
            let accel = motion.accel_mg;
            let sq = accel[0] * accel[0] + accel[1] * accel[1] + accel[2] * accel[2];
            let mut mag = 0i32;
            while (mag + 1) * (mag + 1) <= sq {
                mag += 1;
            }
            current_mg = mag.clamp(0, i32::from(u16::MAX)) as u16;

            // The measured interval, not the intended one. This is the whole lesson of the timing
            // defeats in this project, in one call.
            match detector.update_us(accel, dt_us) {
                Some(Event::Fall {
                    peak_mg,
                    freefall_ms,
                }) => {
                    falls = falls.saturating_add(1);
                    last_peak_mg = peak_mg.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
                    last_freefall_ms = freefall_ms.min(u32::from(u16::MAX)) as u16;
                    last_fall_ms = uptime_ms;
                    ever_fell = true;
                    airborne = false;
                    impact_open = false;

                    // Three blinks: the alarm, without a buzzer on this board.
                    for _ in 0..3 {
                        led.on();
                        delay.delay_ms(60);
                        led.off();
                        delay.delay_ms(60);
                    }

                    let mut msg = [0u8; 64];
                    let mut at = 0;
                    msg[at..at + 10].copy_from_slice(b"FALL peak=");
                    at += 10;
                    at = push_dec(&mut msg, at, peak_mg.unsigned_abs());
                    msg[at..at + 4].copy_from_slice(b" ff=");
                    at += 4;
                    at = push_dec(&mut msg, at, freefall_ms);
                    msg[at..at + 3].copy_from_slice(b" n=");
                    at += 3;
                    at = push_dec(&mut msg, at, u32::from(falls));
                    mailbox::write(mailbox::Status::Pass, &msg[..at]);
                }
                Some(Event::FreeFall { ms }) => {
                    airborne = true;
                    last_freefall_ms = ms.min(u32::from(u16::MAX)) as u16;
                    let mut msg = [0u8; 48];
                    let mut at = 0;
                    msg[at..at + 12].copy_from_slice(b"AIRBORNE ms=");
                    at += 12;
                    at = push_dec(&mut msg, at, ms);
                    mailbox::write(mailbox::Status::Info, &msg[..at]);
                }
                Some(Event::Impact { peak_mg }) => {
                    impact_open = true;
                    last_peak_mg = peak_mg.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
                    let mut msg = [0u8; 48];
                    let mut at = 0;
                    msg[at..at + 12].copy_from_slice(b"impact peak=");
                    at += 12;
                    at = push_dec(&mut msg, at, peak_mg.unsigned_abs());
                    mailbox::write(mailbox::Status::Info, &msg[..at]);
                }
                None => {}
            }
        }

        // The flags are held for a few seconds so a receiver polling every 500 ms cannot miss a
        // verdict that lasted one sample. After that they decay, because a stale alarm is a false one.
        if ever_fell && uptime_ms.saturating_sub(last_fall_ms) > EVENT_HOLD_MS {
            // The fall flag decays but the counter does not: a receiver compares the counter to tell
            // "nothing new" from "a new fall".
        }

        // One advertisement every 100 ms, not one per sample: 400 advertising packets a second would
        // occupy all three channels for no extra information.
        if uptime_ms.saturating_sub(last_adv_ms) >= 100 {
            last_adv_ms = uptime_ms;
            let mut frame = [0u8; 12];
            frame[0] = FRAME_VERSION;
            let mut flags = 0u8;
            if airborne {
                flags |= FLAG_AIRBORNE;
            }
            if ever_fell && uptime_ms.saturating_sub(last_fall_ms) <= EVENT_HOLD_MS {
                flags |= FLAG_FALL;
            }
            if impact_open {
                flags |= FLAG_IMPACT;
            }
            frame[1] = flags;
            frame[2..4].copy_from_slice(&falls.to_le_bytes());
            frame[4..6].copy_from_slice(&last_peak_mg.to_le_bytes());
            frame[6..8].copy_from_slice(&last_freefall_ms.to_le_bytes());
            let since_fall = if ever_fell {
                uptime_ms.saturating_sub(last_fall_ms).min(0xFFFE) as u16
            } else {
                0xFFFF
            };
            frame[8..10].copy_from_slice(&since_fall.to_le_bytes());
            frame[10..12].copy_from_slice(&current_mg.to_le_bytes());

            if ble.set_manufacturer_data(COMPANY_ID, &frame).is_ok() {
                let _ = ble.start_advertising(100, 0);
            }

            // The LED is the fall state: solid while airborne, off otherwise. The alarm blink already
            // happened at the moment of the fall, so a steady light means "in the air now".
            if airborne {
                led.on();
            } else {
                led.off();
            }
        }

        // Sleep the remainder of the period. The next iteration measures the real interval, so a
        // missed deadline shows up in the classifier instead of silently stretching its windows.
        let elapsed_us = dwt::cycles_to_us(dwt::since(last_sample), SYSCLK_HZ);
        if elapsed_us < PERIOD_US {
            delay.delay_us(PERIOD_US - elapsed_us);
        }
    }
}
