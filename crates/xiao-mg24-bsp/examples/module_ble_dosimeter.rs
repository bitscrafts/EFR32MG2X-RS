//! The dosimeter's GATT service: what the phone reads, published from a real capture.
//!
//! The radio side is already proven on this board (advertising, connection, notify). This example is the part
//! that was missing: the dosimeter's own service, fed by the interleaved capture loop that fits in real time at
//! 19 MHz (99 %, `module_dosimeter`).
//!
//! ## What it publishes, and why it publishes *that*
//!
//! ```
//! service  e1d0e000-0000-4000-8000-00805f9b34fb   Noise Dosimeter
//!   e1d0e001  rms     uint16          notify 1 Hz   the window's RMS in converter counts, mid-scale removed
//!   e1d0e002  bands   uint32 x 8      notify 1 Hz   the eight Goertzel powers, raw
//!   e1d0e003  status  uint8 + uint16  read+notify   flags and the number of windows captured
//! ```
//!
//! **No dB, no dose percentage, and that is deliberate.** There is no acoustic calibration on this board, so
//! absolute dB(A) is not a number this firmware has. Publishing "72.4 dB" out of an uncalibrated microphone is
//! the exact failure this project keeps removing: a plausible number with nothing behind it. The raw counts and
//! powers are what the device actually measured, and the phone — which has floating point — turns them into dB
//! once a calibration exists. `status` bit 0 says whether it does, and today it is 0.
//!
//! The UUIDs are 128-bit because no adopted Bluetooth SIG service describes a noise dosimeter, so a generic
//! explorer will show them as hex. That is the honest trade here: the app is ours, and it knows the encoding.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_radio::gatt_db::{self, Characteristic, PROP_NOTIFY, PROP_READ};
use efr32mg24_radio::Bluetooth;
use panic_halt as _;
use xiao_mg24_bsp::hal::dwt;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

const TARGET_HZ: u32 = 16_000;
const TIMER_CLK_HZ: u32 = 19_000_000;
const WINDOW: usize = 256;
/// One second of audio: 62,5 windows, so the loop counts to 63 and notifies.
const WINDOWS_PER_SECOND: usize = 63;
/// How long to run before writing the record. The loop is endless; the record is the proof it ran.
/// How long the service runs. Sixty seconds at 1 Hz is the acceptance criterion (E1-BLE.2): long enough for a phone
/// to connect, discover, subscribe and receive a minute of samples without the run ending underneath it.
const SECONDS: u32 = 60;

const COEFFS: [f32; 8] = [
    1.999_387_962_f32,
    1.997_590_912_f32,
    1.990_369_453_f32,
    1.961_570_561_f32,
    1.847_759_065_f32,
    1.414_213_562_f32,
    0.0_f32,
    -1.414_213_562_f32,
];

/// `e1d0e000-0000-4000-8000-00805f9b34fb` in the stack's little-endian byte order.
const UUID_SERVICE: [u8; 16] = [
    0xfb, 0x34, 0x9b, 0x5f, 0x80, 0x00, 0x00, 0x80, 0x00, 0x40, 0x00, 0x00, 0x00, 0xe0, 0xd0, 0xe1,
];
/// `e1d0e001-…` — the window RMS.
const UUID_RMS: [u8; 16] = [
    0xfb, 0x34, 0x9b, 0x5f, 0x80, 0x00, 0x00, 0x80, 0x00, 0x40, 0x00, 0x00, 0x01, 0xe0, 0xd0, 0xe1,
];
/// `e1d0e002-…` — eight band powers, `u32` little-endian each.
const UUID_BANDS: [u8; 16] = [
    0xfb, 0x34, 0x9b, 0x5f, 0x80, 0x00, 0x00, 0x80, 0x00, 0x40, 0x00, 0x00, 0x02, 0xe0, 0xd0, 0xe1,
];
/// `e1d0e003-…` — flags and window count.
const UUID_STATUS: [u8; 16] = [
    0xfb, 0x34, 0x9b, 0x5f, 0x80, 0x00, 0x00, 0x80, 0x00, 0x40, 0x00, 0x00, 0x03, 0xe0, 0xd0, 0xe1,
];

/// Eight Goertzel filters advanced one sample at a time — the arithmetic that fits inside the wait.
struct Bands {
    s1: [f32; 8],
    s2: [f32; 8],
}

impl Bands {
    fn new() -> Self {
        Self {
            s1: [0.0; 8],
            s2: [0.0; 8],
        }
    }
    fn step(&mut self, sample: u16) {
        let x = sample as f32 - 2048.0;
        for b in 0..8 {
            let s0 = COEFFS[b] * self.s1[b] - self.s2[b] + x;
            self.s2[b] = self.s1[b];
            self.s1[b] = s0;
        }
    }
    /// The eight powers, and the RMS of the window that produced them.
    fn finish(&self, window: &[u16]) -> ([u32; 8], u16) {
        let mut powers = [0u32; 8];
        for b in 0..8 {
            let p = self.s1[b] * self.s1[b] + self.s2[b] * self.s2[b]
                - COEFFS[b] * self.s1[b] * self.s2[b];
            powers[b] = if p > 0.0 { p as u32 } else { 0 };
        }
        let mut acc = 0u64;
        for v in window {
            let d = *v as i32 - 2048;
            acc += (d * d) as u64;
        }
        let rms = if window.is_empty() {
            0
        } else {
            let mean = acc / window.len() as u64;
            // Integer square root: no libm on the target.
            let mut r: u64 = 0;
            let mut bit: u64 = 1 << 30;
            let mut n = mean;
            while bit > n {
                bit >>= 2;
            }
            while bit != 0 {
                if n >= r + bit {
                    n -= r + bit;
                    r = (r >> 1) + bit;
                } else {
                    r >>= 1;
                }
                bit >>= 2;
            }
            r.min(u16::MAX as u64) as u16
        };
        (powers, rms)
    }
}

struct Text {
    buf: [u8; 180],
    len: usize,
}

impl Text {
    fn new() -> Self {
        Self {
            buf: [0; 180],
            len: 0,
        }
    }
    fn s(&mut self, text: &[u8]) {
        for b in text {
            if self.len < self.buf.len() {
                self.buf[self.len] = *b;
                self.len += 1;
            }
        }
    }
    fn u(&mut self, mut v: u32) {
        if v == 0 {
            self.s(b"0");
            return;
        }
        let mut d = [0u8; 10];
        let mut n = 0;
        while v > 0 {
            d[n] = b'0' + (v % 10) as u8;
            v /= 10;
            n += 1;
        }
        while n > 0 {
            n -= 1;
            let b = d[n];
            self.s(&[b]);
        }
    }
    fn bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

#[entry]
fn main() -> ! {
    // SAFETY: this program's own mailbox slot in RAM.
    unsafe { core::ptr::write_volatile(0x2003_F000 as *mut u32, 0) };

    let mut module = match Module::take() {
        Some(m) => m,
        None => loop {
            core::hint::spin_loop();
        },
    };
    let mut delay = module.delay();
    // The stack, then the service.
    let mut ble = match Bluetooth::new(&mut delay) {
        Ok(b) => b,
        Err(_) => {
            mailbox::fail(b"ble: the stack did not start");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    let characteristics = [
        Characteristic {
            uuid: UUID_RMS,
            properties: PROP_READ | PROP_NOTIFY,
            max_len: 2,
        },
        Characteristic {
            uuid: UUID_BANDS,
            properties: PROP_READ | PROP_NOTIFY,
            max_len: 32,
        },
        Characteristic {
            uuid: UUID_STATUS,
            properties: PROP_READ | PROP_NOTIFY,
            max_len: 3,
        },
    ];
    let service = match gatt_db::add_service(UUID_SERVICE, &characteristics) {
        Ok(s) => s,
        Err(_) => {
            mailbox::fail(b"ble: the service was refused");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    // `characteristic()` returns an Option: a service can be added with fewer characteristics than asked for.
    let (rms_handle, bands_handle, status_handle) = match (
        service.characteristic(0),
        service.characteristic(1),
        service.characteristic(2),
    ) {
        (Some(r), Some(b), Some(s)) => (r, b, s),
        _ => {
            mailbox::fail(b"ble: the service came back short");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // The microphone is taken AFTER the stack, not before: `Board::converter()` sets the IADC's clock source,
    // and the stack reconfigures the clock tree when it starts. Taking the converter first meant its clock was
    // changed underneath the timer, the audio arrived at half the rate, and the loop read exactly 2x real time
    // (measured 2026-11-17: 195,8 M cycles for 95,8 M of audio).
    let mut mic = match module.microphone() {
        Ok(m) => m,
        Err(_) => {
            mailbox::fail(b"ble: converter will not start");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    // The pace is set AFTER the stack starts, on purpose: the stack configures clocks, and pacing first meant
    // the converter's clock changed underneath the timer — the loop then took exactly 2x real time, which is
    // what a halved CLK_SRC_ADC looks like. Measured 2026-11-17.
    if mic.pace_from_timer(TIMER_CLK_HZ, TARGET_HZ).is_err() {
        mailbox::fail(b"ble: converter will not disable for the trigger write");
        loop {
            cortex_m::asm::wfi();
        }
    }
    mic.arm();
    mic.start_timer();

    let _ = ble.set_device_name("xiao-mg24-dosimeter");
    let _ = ble.start_connectable(200, 0);

    // Calibrate the pace against the core clock instead of trusting the timer's nominal clock: one window is
    // 16 ms, so it must cost 304 000 cycles at 19 MHz. Whatever it actually costs scales the period linearly.
    {
        let mut window = [0u16; WINDOW];
        let t0 = dwt::cycles();
        let got = mic.capture_window(&mut window);
        let took = dwt::since(t0);
        if got == WINDOW && took > 0 {
            let wanted = ((WINDOW as u64) * 19_000_000u64 / TARGET_HZ as u64) as u32;
            let nominal = TIMER_CLK_HZ / TARGET_HZ;
            let corrected = ((nominal as u64 * took as u64) / wanted as u64) as u32;
            if corrected > 0 && corrected < 60000 {
                let _ = mic.retune_period(corrected);
            }
        }
    }

    let mut windows_total = 0u32;
    let mut notifications = 0u32;
    let mut published = 0u32;
    let mut last_rms = 0u16;
    let mut last_band0 = 0u32;

    // SAFETY: nothing else in this program touches DWT.
    unsafe { dwt::start() };
    let started = dwt::cycles();

    for second in 0..SECONDS {
        let mut window = [0u16; WINDOW];
        let mut bands_acc = [0u32; 8];
        let mut rms_of_second = 0u16;

        for _ in 0..WINDOWS_PER_SECOND {
            let mut bands = Bands::new();
            let mut got = 0usize;
            while got < WINDOW {
                match mic.read_paced() {
                    Some(v) => {
                        bands.step(v);
                        window[got] = v;
                        got += 1;
                    }
                    None => break,
                }
            }
            if got != WINDOW {
                continue;
            }
            let (powers, rms) = bands.finish(&window);
            for b in 0..8 {
                bands_acc[b] = bands_acc[b].saturating_add(powers[b]);
            }
            rms_of_second = rms;
            windows_total += 1;
        }

        // The numbers for this second, on the wire.
        let rms_value = rms_of_second.to_le_bytes();
        let mut bands_value = [0u8; 32];
        for b in 0..8 {
            bands_value[b * 4..b * 4 + 4].copy_from_slice(&bands_acc[b].to_le_bytes());
        }
        // bit 0: calibrated (0 — there is no acoustic reference on this board). Then the window count.
        let status_value = [
            0u8,
            (windows_total & 0xFF) as u8,
            ((windows_total >> 8) & 0xFF) as u8,
        ];

        // Count what was *published* and, separately, what reached a subscriber. Without a phone connected
        // `notify` legitimately fails, so the two numbers are different facts and the record keeps both.
        let _ = ble.write_attribute(rms_handle, &rms_value);
        published += 1;
        if ble.notify(rms_handle, &rms_value).is_ok() {
            notifications += 1;
        }
        let _ = ble.write_attribute(bands_handle, &bands_value);
        let _ = ble.notify(bands_handle, &bands_value);
        let _ = ble.write_attribute(status_handle, &status_value);
        let _ = ble.notify(status_handle, &status_value);

        last_rms = rms_of_second;
        last_band0 = bands_acc[0];

        // Drain the stack's events between seconds: a connection only stays alive while its events are read.
        let _ = second;
        while let Some(_event) = ble.poll_event() {}
    }

    let elapsed = dwt::since(started);
    let timer_running = mic.timer_active();

    let mut t = Text::new();
    t.s(b"ble: seconds=");
    t.u(SECONDS);
    t.s(b" windows=");
    t.u(windows_total);
    t.s(b" published=");
    t.u(published);
    t.s(b" notifies=");
    t.u(notifications);
    t.s(b" cycles=");
    t.u(elapsed);
    t.s(b" timer=");
    t.u(u32::from(timer_running));
    t.s(b" rms=");
    t.u(last_rms as u32);
    t.s(b" b0=");
    t.u(last_band0);

    // Five seconds of audio, 315 windows, and the RMS must be a real number (a dead analog path reads 0).
    // What can be proven without a phone: the capture ran for the full time, the service was populated, and
    // the microphone carried real values. Whether a subscriber received them is the phone's test, and the
    // record reports that count separately rather than pretending zero means failure.
    let ok = windows_total >= (WINDOWS_PER_SECOND as u32 * SECONDS).saturating_sub(SECONDS)
        && published > 0
        && timer_running
        && last_rms > 0;
    if ok {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
