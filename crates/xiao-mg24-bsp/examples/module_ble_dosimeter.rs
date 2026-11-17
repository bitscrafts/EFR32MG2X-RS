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
use xiao_mg24_bsp::prelude::*;
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
    /// Feed one sample, **already centred on the window's mean**.
    ///
    /// The converter's bias is not mid-scale: measured 2026-11-17 with `module_mic_stats`, `mean=2391` in one window and
    /// `2414` in another, against a mid-scale of 2048. Subtracting a fixed 2048 therefore fed a constant ~343 counts
    /// into every filter in the bank, which is not sound and shows up worst in the lowest band, where it is most of the
    /// energy. The caller centres each sample on its own window's mean before this sees it.
    fn step(&mut self, x: f32) {
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
        // Centred on the window's own mean, for the same reason the filter is: 2048 is not this converter's bias, and
        // the level it produced was mostly the bias (measured: `rms=343` for a signal of 8).
        let mean = if window.is_empty() {
            2048
        } else {
            (window.iter().map(|v| *v as u64).sum::<u64>() / window.len() as u64) as i32
        };
        let mut acc = 0u64;
        for v in window {
            let d = *v as i32 - mean;
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

/// What the device itself knows about the current link.
///
/// `sec` is the highest `security_mode` reported by `sl_bt_evt_connection_parameters` on this connection (0 none,
/// 1 unauthenticated encryption, which is what Just Works gives). It exists because nothing measured before it proved
/// that a reconnect after a reset was *encrypted*: `Bonded: yes` in `bluetoothctl info` is the host's own storage, and
/// `gatttool` at its default `--sec-level=low` receives these notifications without ever asking the link to encrypt
/// (the characteristics need no security), so they proved the GATT server and not the bond.
struct Link {
    sec: u8,
    /// The status the stack returned for the last `increase_security`. `0` means the request was accepted;
    /// anything else is the reason the link is not being encrypted, which is worth having in the record.
    security_request: u8,
    /// ATT_MTU; 23 is the default until `sl_bt_evt_gatt_mtu_exchanged` says otherwise.
    mtu: u16,
    /// `sl_bt_evt_sm_confirm_bonding` events seen since boot. Zero while the flag that requests them is set would mean
    /// the stack does not raise it with the external database (`sl_bt_api.h`, bonding field forced to 0xff).
    cfb: u32,
    /// Disconnects seen, and advertising restarts issued. They must be equal: a disconnect that is popped by a drain
    /// that does not restart advertising leaves the device silent until a power cycle ("Connection refused (111)").
    disc: u32,
    adv: u32,
    /// Value handles of rms, bands, status, and whether the client has subscribed to each. A notify to an
    /// unsubscribed characteristic fails with `INVALID_STATE`, so there is no reason to issue it.
    handles: [u16; 3],
    subscribed: [bool; 3],
}

impl Link {
    const fn new() -> Self {
        Self {
            sec: 0,
            security_request: 0xFF,
            mtu: 23,
            cfb: 0,
            disc: 0,
            adv: 0,
            handles: [0; 3],
            subscribed: [false; 3],
        }
    }

    fn note(&mut self, event: &efr32mg24_radio::BleEvent) {
        use efr32mg24_radio::BleEvent;
        match event {
            BleEvent::Subscribed {
                characteristic,
                notifications,
                ..
            } => {
                for i in 0..3 {
                    if self.handles[i] == *characteristic {
                        self.subscribed[i] = *notifications;
                    }
                }
            }
            BleEvent::ConnectionParameters { security_mode, .. } => {
                self.sec = self.sec.max(*security_mode);
            }
            BleEvent::Mtu { mtu, .. } => self.mtu = *mtu,
            BleEvent::ConfirmBonding { .. } => self.cfb += 1,
            // A new connection starts unencrypted at the default MTU; keeping the previous link's values would make a
            // stale `sec=1` look like proof for a link that never encrypted.
            BleEvent::Disconnected { .. } | BleEvent::Connected { .. } => {
                self.sec = 0;
                self.mtu = 23;
                // Subscriptions belong to a connection (a bonded client's come back through its own CCCD write).
                self.subscribed = [false; 3];
            }
            _ => {}
        }
    }
}

/// Drain the stack's event queue and act on what it says. Returns true if a disconnect was seen.
///
/// **One handler for both drain sites.** The re-advertise used to live only in the per-sample drain, while the
/// per-second drain threw every event away: a disconnect popped there left the device silent until a power cycle,
/// which is the "Connection refused (111)" the first re-advertise was written to fix. `AdvertisingTimeout` is handled
/// for the same reason (the set stops, nothing restarts it).
fn service_stack(ble: &mut Bluetooth, link: &mut Link) -> bool {
    use efr32mg24_radio::BleEvent;
    let mut disconnected = false;
    while let Some(event) = ble.poll_event() {
        link.note(&event);
        match event {
            BleEvent::Connected { connection, .. } => {
                // Ask for encryption the moment the link is up. A bonded peripheral that never asks stays bonded and
                // unencrypted: measured 2026-11-17, `sec=0` on a link whose pairing had completed, with the host
                // reporting `Bonded: yes` throughout. `sl_bt_api.h:10646` is explicit that this command is what
                // encrypts the connection and, when there is no bond yet, starts the bonding.
                link.security_request = ble.increase_security(connection);
            }
            BleEvent::Disconnected { .. } => {
                link.disc += 1;
                disconnected = true;
                if ble.start_connectable(200, 0).is_ok() {
                    link.adv += 1;
                }
            }
            // Not counted in `adv`, which pairs one-to-one with `disc`.
            BleEvent::AdvertisingTimeout { .. } => {
                let _ = ble.start_connectable(200, 0);
            }
            _ => {}
        }
    }
    disconnected
}

struct Text {
    buf: [u8; 256],
    len: usize,
}

impl Text {
    fn new() -> Self {
        Self {
            buf: [0; 256],
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

    // Provisioning: holding the user button during boot forgets every bond.
    //
    // Without this the device is unrecoverable in the field: a peripheral that holds a bond for a phone which has
    // since forgotten it answers the next pairing with the old long-term key, the phone cancels with
    // `AuthenticationFailed`, and nothing on the device can be asked to forget -- with an external bonding database
    // the stack's own delete commands answer NOT_AVAILABLE, and with the builtin one there is no way in without a
    // debugger. Measured both ways on 2026-11-17. Three seconds is long enough not to happen by accident and short
    // enough to be usable with the device in a hand.
    {
        let button = module.hal().input_on(xiao_mg24_bsp::hal::pin::Port::C, 1);
        let mut held = 0u32;
        while button.is_low() {
            delay.delay_ms(10);
            held += 1;
            if held >= 300 {
                #[cfg(feature = "external-bonding")]
                let removed = efr32mg24_radio::bonding_clear_all();
                #[cfg(not(feature = "external-bonding"))]
                let removed = {
                    let _ = ble.delete_bondings();
                    1u32
                };
                // Say so on the LED: three short pulses means the bonds are gone.
                let mut led = module.led();
                for _ in 0..3 {
                    led.on();
                    delay.delay_ms(60);
                    led.off();
                    delay.delay_ms(60);
                }
                let _ = removed;
                break;
            }
        }
    }

    let _ = ble.set_device_name("xiao-mg24-dosimeter");

    // Pairing, and what it costs the phone.
    //
    // The bond now survives a reset of the MCU (measured 2026-11-17 with the external bonding database: the host
    // reconnects `Bonded: yes` after `probe-rs reset`, without pairing again), which is the condition this was
    // waiting for. Before that, a device that pairs and then forgets is worse for the phone than one that never
    // pairs at all: the user has to forget the device before every session.
    //
    // `enable_pairing_with(0x84, 0x03)`: flags 0x84 = bonding required + MITM ("authenticated"), IO capability 0x03 =
    // NoInputNoOutput, i.e. Just Works. That keeps the phone's dialog simple; a passkey would be stricter but this
    // product has no display to show one on.
    let _ = ble.enable_pairing_with(0x84, 0x03);
    // With the builtin database the stack needs to be told how many bonds to keep. With the external one that command
    // is documented as NOT_AVAILABLE and the count lives in `bonding_ext`.
    #[cfg(not(feature = "external-bonding"))]
    let _ = ble.store_bonding_configuration(8, 1);

    // No clock boost and no pace calibration here, on purpose. The core runs at 39.18 MHz whatever `boost_to_80_mhz`
    // does: 2 350 795 993 DWT cycles over 60 s with the boost, 2 350 470 467 without it. That is the 39 MHz HFXO the
    // stack selects in `Bluetooth::new` (`stack_init.rs`), and the boost only retunes HFRCODPLL, which no longer clocks
    // the core. The calibration read `dwt::cycles()` before `dwt::start()`, so it measured nothing (or, with DWT on,
    // assumed a 19 MHz core and would have doubled the period); `windows=3780` in 60 s (63/s) shows it changed nothing.
    let _ = ble.start_connectable(200, 0);

    let mut windows_total = 0u32;
    let mut notifications = 0u32;
    let mut published = 0u32;
    let mut last_rms = 0u16;
    let mut last_band0 = 0u32;
    let mut link = Link::new();
    link.handles = [
        rms_handle.value(),
        bands_handle.value(),
        status_handle.value(),
    ];

    // SAFETY: nothing else in this program touches DWT.
    unsafe { dwt::start() };
    let started = dwt::cycles();

    // The loop runs until the board is reset: a product does not stop measuring after a minute. `SECONDS` is the
    // reporting period, not a lifetime -- and a device that stops advertising after a minute is unusable from the
    // phone, because the bond is only worth something while the peripheral is there to use it.
    for second in 0.. {
        if second % SECONDS == 0 && second > 0 {
            // Publish a summary of the last period, then carry on.
            let mut t = Text::new();
            t.s(b"ble: seconds=");
            t.u(second);
            t.s(b" windows=");
            t.u(windows_total);
            t.s(b" published=");
            t.u(published);
            t.s(b" notifies=");
            t.u(notifications);
            t.s(b" cycles=");
            t.u(dwt::since(started));
            t.s(b" timer=");
            t.u(mic.timer_active() as u32);
            t.s(b" rms=");
            t.u(last_rms as u32);
            t.s(b" b0=");
            t.u(last_band0);
            t.s(b" sreq=");
            t.u(link.security_request as u32);
            t.s(b" sec=");
            t.u(link.sec as u32);
            t.s(b" mtu=");
            t.u(link.mtu as u32);
            t.s(b" cfb=");
            t.u(link.cfb);
            t.s(b" disc=");
            t.u(link.disc);
            t.s(b" adv=");
            t.u(link.adv);
            // Key-table health: `rst` keys restored at boot must be the same number after every reset, `free` slots
            // must stay at 10 or more, and `dfail` (destroy calls that errored) must be 0.
            t.s(b" rst=");
            t.u(efr32mg24_radio::psa_lookup_stats().2);
            t.s(b" free=");
            t.u(efr32mg24_radio::psa_free_slots());
            t.s(b" stale=");
            t.u(efr32mg24_radio::stale_bonds_dropped());
            t.s(b" dkeys=");
            t.u(efr32mg24_radio::destroyed_bond_keys());
            t.s(b" dfail=");
            t.u(efr32mg24_radio::psa_destroy_failures());
            mailbox::info(t.bytes());
        }
        let mut window = [0u16; WINDOW];
        let mut bands_acc = [0u32; 8];
        let mut rms_of_second = 0u16;

        for _ in 0..WINDOWS_PER_SECOND {
            let mut bands = Bands::new();
            let mut got = 0usize;
            while got < WINDOW {
                match mic.read_paced() {
                    Some(v) => {
                        window[got] = v;
                        got += 1;
                        // Service the stack while sampling, not only between seconds. A connection lives on the
                        // stack's own scheduling, and the SMP exchange of a pairing needs the application to drain
                        // the BGAPI queue often enough to answer it: measured 2026-11-17, the product example failed
                        // to pair three times out of three while the probe example, whose loop has nothing else to do,
                        // paired first time. One call per sample costs a compare in the common case.
                        if service_stack(&mut ble, &mut link) {
                            notifications = 0;
                        }
                    }
                    None => break,
                }
            }
            if got != WINDOW {
                continue;
            }
            // Feed the filter bank only now, with the window's mean removed. The mean is not known until the window
            // is complete, and feeding the raw samples would push the converter's bias (measured at 2391 and at 2414 in
            // two windows, against a mid-scale of 2048) through every filter as a constant input.
            let mean = (window.iter().map(|v| *v as u64).sum::<u64>() / WINDOW as u64) as i32;
            for v in window.iter() {
                bands.step(*v as i32 as f32 - mean as f32);
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
        //
        // Same handler as the per-sample drain, so a disconnect popped here is not lost.
        let _ = second;
        if service_stack(&mut ble, &mut link) {
            notifications = 0;
        }
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
