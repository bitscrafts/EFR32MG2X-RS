//! Pairing from the host, with the failure recorded at the moment it happens.
//!
//! Two candidate reasons for D2f are on the table and one measurement separates them:
//!
//! ```
//! 1. bg_zalloc(64) draws from the host's system buffer pool (SL_BT_CONFIG_BUFFER_SIZE = 8192), not sl_malloc
//! 2. the allocation fails transiently during the exchange (the steady-state heap was 17 % used, 0 failures)
//! ```
//!
//! What this records: every Security event with its `event`/`connection`/`reason`, any event the API does not know
//! (the stack's `system_error` 0x1A and `sm_bonding_failed` 0x120B would land there), the heap's failure count and
//! the size of the last refused allocation, and the connection count. Pair from the host while this runs.
//!
//! A refusal recorded by `sl_malloc` with size 64 proves candidate 2 and blames our arena. A failure with no
//! `sl_malloc` refusal at all proves candidate 1 and moves the fix to the host's system buffer pool.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_radio::Bluetooth;
use panic_halt as _;
use xiao_mg24_bsp::prelude::*;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

extern "C" {
    /// The default NVM3 instance, opened by the stack at start-up.
    static mut nvm3_defaultHandle: *mut u8;
    /// Lists object keys: returns how many were written into the buffer.
    fn nvm3_enumObjects(handle: *mut u8, key_list: *mut u32, key_list_size: usize) -> usize;
}

/// How many objects the Bluetooth stack's bonding database holds in NVM3.
///
/// The bond is created (`Bonded: yes` from the peer) and after a reset the device answers HCI 0x1006 "PIN or Key
/// Missing". NVM3 itself persists (measured: a counter incremented across two resets), so the question is whether the
/// stack ever stored the bond. This counts its objects: a count that rises after pairing and survives a reset means
/// the store works and the *load* is the defect.
fn nvm3_object_count() -> u32 {
    let mut keys = [0u32; 64];
    let n = unsafe { nvm3_enumObjects(nvm3_defaultHandle, keys.as_mut_ptr(), keys.len()) };
    n as u32
}

/// How long the device stays available for a pairing attempt before it reports and stops.
const WINDOW_MS: u32 = 600_000;

struct Text {
    buf: [u8; 190],
    len: usize,
}

impl Text {
    fn new() -> Self {
        Self {
            buf: [0; 190],
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
    fn hex(&mut self, v: u32) {
        let mut started = false;
        for shift in (0..8).rev() {
            let nib = (v >> (shift * 4)) & 0xF;
            if nib != 0 || started || shift == 0 {
                started = true;
                let c = if nib < 10 {
                    b'0' + nib as u8
                } else {
                    b'a' + (nib - 10) as u8
                };
                self.s(&[c]);
            }
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

    let mut ble = match Bluetooth::new(&mut delay) {
        Ok(b) => b,
        Err(_) => {
            mailbox::fail(b"pairprobe: the stack did not start");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    // Never discard these codes: a refused start_connectable leaves a device that looks alive and is silent,
    // which is exactly how an afternoon gets spent scanning for a board that never advertised.
    // One variable at a time: with the security manager present but never configured, does a phone connect?
    // Before this session the SM was compiled out and the phone connected; if it connects now with SM present but
    // unconfigured, the act of configuring it is what breaks the link, and that is a much sharper question.
    const CONFIGURE_SM: bool = true;
    let (sm_cfg, sm_bond) = if CONFIGURE_SM {
        ble.enable_pairing_with(0x84, 0x03)
    } else {
        (0xFF, 0xFF)
    };
    let rc_name = ble.set_device_name("xiao-pair-probe").is_ok();
    let rc_adv = ble.start_connectable(200, 0).is_ok();
    let mut startup = Text::new();
    startup.s(b"pairprobe: flags=0x84 sm_cfg=0x");
    startup.hex(sm_cfg as u32);
    startup.s(b" sm_bond=0x");
    startup.hex(sm_bond as u32);
    startup.s(b" name=");
    startup.u(rc_name as u32);
    startup.s(b" adv=");
    startup.u(rc_adv as u32);
    mailbox::info(startup.bytes());

    let mut connections: u32 = 0;
    let mut sec_events: u32 = 0;
    let mut sec_event: u32 = 0;
    let mut sec_reason: u32 = 0;
    let mut unknown: u32 = 0;
    let mut last_unknown: u32 = 0;

    // Live counters, rewritten once a second: a person pairing from a phone cannot wait for a window to close,
    // and a record that only exists at the end is a record nobody can watch.
    for tick in 0..WINDOW_MS {
        while let Some(event) = ble.poll_event() {
            match event {
                efr32mg24_radio::BleEvent::Connected { .. } => connections += 1,
                efr32mg24_radio::BleEvent::Disconnected { reason, .. } => {
                    sec_event = 0x1000 + reason as u32;
                }
                efr32mg24_radio::BleEvent::Security { event, reason, .. } => {
                    sec_events += 1;
                    sec_event = event as u32;
                    sec_reason = reason as u32;
                }
                efr32mg24_radio::BleEvent::Unknown { event_id } => {
                    unknown += 1;
                    last_unknown = event_id;
                }
                _ => {}
            }
        }
        delay.delay_ms(1);
        if tick % 1_000 == 0 {
            let heap = efr32mg24_radio::heap_stats();
            let largest = efr32mg24_radio::heap_largest_free();
            let mut live = Text::new();
            let ec = efr32mg24_radio::psa_ec_counts();
            live.s(b"live: ec=");
            live.u(ec.0 as u32);
            live.s(b",");
            live.u(ec.1 as u32);
            live.s(b",");
            live.u(ec.2 as u32);
            let lf = efr32mg24_radio::psa_last_failure();
            live.s(b" lf=");
            live.u(lf.0);
            live.s(b"/");
            live.u(lf.1);
            live.s(b" nvm=");
            live.u(nvm3_object_count());
            live.s(b" bc=");
            live.u(efr32mg24_radio::psa_last_call());
            let gen = efr32mg24_radio::psa_generate_stats();
            live.s(b" gen=");
            live.u(gen.0);
            live.s(b" uns=0x");
            live.hex(gen.1);
            live.s(b" ref=0x");
            live.hex(efr32mg24_radio::psa_import_refused());
            let imp = efr32mg24_radio::psa_import_stats();
            live.s(b" imp=");
            live.u(imp.0);
            live.s(b"/");
            live.u(imp.1);
            live.s(b" cmac=");
            live.u(efr32mg24_radio::cmac_kat() as u32);
            live.s(b" t=");
            live.u(tick / 1_000);
            live.s(b" sm=0x");
            live.hex(sm_cfg as u32);
            live.s(b"/0x");
            live.hex(sm_bond as u32);
            live.s(b" conn=");
            live.u(connections);
            live.s(b" sec=");
            live.u(sec_events);
            live.s(b" sev=");
            live.u(sec_event);
            live.s(b" rsn=");
            live.u(sec_reason);
            live.s(b" unk=");
            live.u(unknown);
            live.s(b" uid=0x");
            live.hex(last_unknown);
            live.s(b" largest=");
            live.u(largest as u32);
            live.s(b" inuse=");
            live.u(heap.0 as u32);
            live.s(b" fails=");
            live.u(heap.1);
            mailbox::info(live.bytes());
        }
    }

    let heap = efr32mg24_radio::heap_stats();
    let largest = efr32mg24_radio::heap_largest_free();
    let audit = efr32mg24_radio::heap_audit();

    let mut t = Text::new();
    t.s(b"pairprobe: conn=");
    t.u(connections);
    t.s(b" sec=");
    t.u(sec_events);
    t.s(b" sev=");
    t.u(sec_event);
    t.s(b" rsn=");
    t.u(sec_reason);
    t.s(b" unk=");
    t.u(unknown);
    t.s(b" uid=0x");
    t.hex(last_unknown);
    t.s(b" fails=");
    t.u(heap.1);
    t.s(b" largest=");
    t.u(largest as u32);
    t.s(b" inuse=");
    t.u(heap.0 as u32);
    t.s(b" bad=");
    t.u(audit.0);

    // The evidence is the record, not a verdict: with no host pairing during the window every count is zero and
    // that is a correct measurement of "nothing was attempted".
    if heap.1 == 0 && audit.0 == 0 {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
