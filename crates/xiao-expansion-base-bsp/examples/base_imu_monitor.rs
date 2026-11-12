//! IMU monitor: live acceleration on the panel, and a 30 s ring buffer for capture.
//!
//! ## Why 30 seconds
//!
//! The first capture example kept ten seconds. By the time the samples were read
//! over SWD the movement had scrolled out of the window and the buffer held
//! nothing but a board sitting still — a measurement that looks like a successful
//! capture and contains no event. Thirty seconds (3000 samples at 100 Hz, 18 KB of
//! the 252 KB now available) survives the round trip.
//!
//! ## Why the panel
//!
//! The values have to be visible *while the board is being moved*: that is how a
//! person knows the sensor sees the shake, before any classifier is trained on it.
//! The display shows the three axes, the magnitude, the peak since boot and the
//! number of samples taken, so a stalled loop or a disconnected sensor is visible
//! immediately instead of being inferred from an empty buffer later.
//!
//! Nothing here decides anything yet: it measures and shows. The classifier is
//! trained on what this records.

#![no_std]
#![no_main]

use core::cell::RefCell;

use efr32mg24_hal::board::Board;
use efr32mg24_hal::bus::Bus;
use efr32mg24_hal::bus::{I2cBus, SharedBus};
use efr32mg24_hal::display::{append, format_fixed, Ssd1306};
use efr32mg24_hal::entry;
use efr32mg24_hal::pin::Port;
use efr32mg24_hal::sensors::lsm6ds3trc::{AccelOdr, AccelRange, Config};
use efr32mg24_hal::sensors::Lsm6ds3trc;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// 200 Hz at ~19 MHz SYSCLK. The accelerometer runs at 416 Hz, so this samples
/// slower than the sensor produces but four times faster than a fall needs.
const SAMPLE_PERIOD_CYCLES: u32 = 95_000;
/// 3000 samples × 200 Hz = 15 s.
const CAPTURE_LEN: usize = 3000;

#[repr(align(32))]
struct Samples([[i16; 3]; CAPTURE_LEN]);

static mut CAPTURE: Samples = Samples([[0; 3]; CAPTURE_LEN]);
static mut TOTAL: u32 = 0;

// The cycle counter is the HAL's, not a peripheral an example reaches for.
fn cycle_count() -> u32 {
    efr32mg24_hal::dwt::cycles()
}

fn dwt_start() {
    // SAFETY: nothing else in this example uses DWT.
    unsafe { efr32mg24_hal::dwt::start() };
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

/// Integer square root — `f32::sqrt` needs libm, which a `no_std` binary does not
/// link by default. Newton's method over integers, exact for the range of values
/// an accelerometer produces.
fn isqrt(value: u64) -> u32 {
    if value == 0 {
        return 0;
    }
    let mut x = value;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + value / x) / 2;
    }
    x as u32
}

/// One display line: label, then the value in mg, right-aligned enough to read.
fn line(out: &mut [u8], label: &[u8], value: i32) -> usize {
    let mut pos = append(out, 0, label);
    pos += format_fixed(&mut out[pos..], value * 100, 0);
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

    board.output_on(Port::D, 5, false).on();
    cortex_m::asm::delay(4_000_000);

    let imu_bus = RefCell::new(board.i2c_on(Bus::new(Port::B, 1 << 2, 1 << 3)));
    // ±8 g and 416 Hz: at ±2 g/104 Hz an impact clips or falls between samples,
    // which is exactly why the first two capture attempts held no fall.
    let config = Config {
        range: AccelRange::G8,
        odr: AccelOdr::Hz416,
    };
    let mut imu = match Lsm6ds3trc::with_config(SharedBus::new(&imu_bus), config) {
        Ok(i) => i,
        Err(_) => {
            mailbox::fail(b"imu init failed on the monitor");
            loop {
                led.toggle();
                cortex_m::asm::delay(2_000_000);
            }
        }
    };

    let base_bus = RefCell::new(board.i2c_on(Bus::new(Port::C, 1 << 4, 1 << 5)));
    // `None` when the panel does not answer: the capture still runs, so a missing
    // display cannot cost the measurement.
    let mut oled = Ssd1306::new(SharedBus::new(&base_bus));

    dwt_start();

    let base = unsafe { core::ptr::addr_of!(CAPTURE.0) as u32 };
    let mut msg = [0u8; 64];
    let mut pos = push_bytes(&mut msg, 0, b"mon base=0x");
    pos = push_hex8(&mut msg, pos, base);
    pos = push_bytes(&mut msg, pos, b" n=");
    pos = push_dec(&mut msg, pos, CAPTURE_LEN as u32);
    pos = push_bytes(&mut msg, pos, b" 200Hz pm8g");
    mailbox::pass(&msg[..pos]);

    let mut index = 0usize;
    let mut last_cycles = 0u32;
    let mut peak = 0i32;
    let mut frames = 0u32;
    let mut shown = [0i32; 3];
    let mut shown_mag = 0i32;
    let mut shown_peak = 0i32;
    let mut shown_total = 0u32;

    loop {
        let now = cycle_count();
        if now.wrapping_sub(last_cycles) < SAMPLE_PERIOD_CYCLES {
            continue;
        }
        last_cycles = now;

        let Ok(motion) = imu.read() else {
            continue;
        };
        let a = motion.accel_mg;
        let magnitude = isqrt(
            (a[0] as i64 * a[0] as i64 + a[1] as i64 * a[1] as i64 + a[2] as i64 * a[2] as i64)
                as u64,
        ) as i32;
        if magnitude > peak {
            peak = magnitude;
        }

        unsafe {
            CAPTURE.0[index] = [a[0] as i16, a[1] as i16, a[2] as i16];
            TOTAL = TOTAL.wrapping_add(1);
        }
        index += 1;
        if index == CAPTURE_LEN {
            index = 0;
        }

        // Refresh the panel every tenth of a second — faster than a person reads.
        frames += 1;
        if frames % 10 == 0 && (a != shown || magnitude != shown_mag || total() != shown_total) {
            shown = a;
            shown_mag = magnitude;
            shown_peak = peak;
            shown_total = total();

            if let Some(ref mut d) = oled {
                d.clear();
                let mut buf = [0u8; 24];
                let n = line(&mut buf, b"AX ", a[0]);
                d.text(0, 0, &buf[..n]);
                let n = line(&mut buf, b"AY ", a[1]);
                d.text(0, 1, &buf[..n]);
                let n = line(&mut buf, b"AZ ", a[2]);
                d.text(0, 2, &buf[..n]);
                let n = line(&mut buf, b"|A| ", magnitude);
                d.text(0, 3, &buf[..n]);
                let n = line(&mut buf, b"PK ", peak);
                d.text(0, 4, &buf[..n]);
                let mut l6 = [0u8; 24];
                let mut p6 = append(&mut l6, 0, b"N ");
                p6 = push_dec(&mut l6, p6, shown_total);
                d.text(0, 6, &l6[..p6]);
                d.flush();
            }
        }

        // Heartbeat: one blink per second, independent of the panel.
        if frames % 100 == 0 {
            led.toggle();
        }
    }
}

fn total() -> u32 {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(TOTAL)) }
}
