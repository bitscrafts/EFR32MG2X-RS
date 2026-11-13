//! Give RAIL its channel configuration, then start the Bluetooth stack.
//!
//! The wall this probe attacks: `sli_bt_init_stack` reaches RAIL, RAIL asserts
//! `RAIL_ASSERT_INVALID_XTAL_FREQUENCY` (code 66) because nothing has told it the crystal is 39 MHz, and the default
//! `RAILCb_AssertFailed` is an infinite loop — so it presents as a hang with no evidence.
//!
//! RAIL gets the frequency from `RAIL_ChannelConfig_t.xtalFrequencyHz`, and the Bluetooth blobs never call
//! `RAIL_ConfigChannels` (zero imports across all three archives), so the application has to. The configuration is
//! the SDK's own generated output for this part at 39 MHz, emitted into Rust by `emit-ble-config.py` and reachable
//! as `efr32mg24_rail_sys::ble_config::config()`.
//!
//! The order is the SDK's flex utility's: `RAIL_Init`, then `RAIL_ConfigChannels`, then whoever needs the radio.
//!
//! Every stage publishes, so the last record names the stage that stalled.

#![no_std]
#![no_main]

#[path = "../../xiao-mg24-bsp/examples/common/mailbox.rs"]
mod mailbox;

use cortex_m_rt::entry;
use efr32mg24_hal::board::Board;
use efr32mg24_hal::prelude::*;
use panic_halt as _;

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

fn push_hex(out: &mut [u8], mut at: usize, value: u32) -> usize {
    let hex = b"0123456789ABCDEF";
    for shift in [28u32, 24, 20, 16, 12, 8, 4, 0] {
        out[at] = hex[((value >> shift) & 0xF) as usize];
        at += 1;
    }
    at
}

fn label(out: &mut [u8], mut at: usize, s: &[u8]) -> usize {
    for b in s {
        out[at] = *b;
        at += 1;
    }
    at
}

fn stage(tag: &[u8], a: u32, b: u32) {
    let mut m = [0u8; 64];
    let mut n = 0;
    n = label(&mut m, n, tag);
    n = label(&mut m, n, b" a=");
    n = push_dec(&mut m, n, a);
    n = label(&mut m, n, b" b=");
    n = push_dec(&mut m, n, b);
    mailbox::write(mailbox::Status::Pass, &m[..n]);
}

#[entry]
fn main() -> ! {
    let mut board = Board::take().expect("peripherals taken twice");
    let mut delay = board.delay();
    let _rtc = board.sysrtc();

    stage(b"boot", 0, 0);

    // 1. RAIL, with the Radio Configurator's BLE 39 MHz table.
    let handle = unsafe {
        let mut config: efr32mg24_rail_sys::RAIL_Config_t = core::mem::zeroed();
        efr32mg24_rail_sys::RAIL_Init(&mut config, None)
    };
    stage(b"railinit", handle as usize as u32, 0);

    let cfg = efr32mg24_rail_sys::ble_config::config();
    stage(b"xtal", cfg.xtalFrequencyHz, cfg.length);

    let chan_status = unsafe { efr32mg24_rail_sys::RAIL_ConfigChannels(handle, cfg, None) };
    stage(b"chancfg", chan_status as u32, 0);

    // A control: if RAIL accepted the configuration, asking it for the frequency again must agree, and the
    // assertion counter must still be zero.
    stage(
        b"preassert",
        efr32mg24_bluetooth_sys::rail_assert::last_assert()
            .map(|(c, _, _)| c)
            .unwrap_or(0),
        0,
    );

    // 2. The stack, with RAIL already configured.
    let boot = unsafe {
        efr32mg24_bluetooth_sys::stack_init::start(
            efr32mg24_bluetooth_sys::stack_init::default_configuration(),
        )
    };
    stage(b"bootapi", boot, 0);

    // 3. Look for system_boot.
    delay.delay_ms(100);
    let mut events = 0u32;
    let mut sys_boot = 0u32;
    let mut version = 0u32;
    for _ in 0..400 {
        efr32mg24_bluetooth_sys::sleeptimer::poll();
        let mut evt = core::mem::MaybeUninit::<efr32mg24_bluetooth_sys::ffi::sl_bt_msg_t>::uninit();
        let popped = unsafe { efr32mg24_bluetooth_sys::ffi::sl_bt_pop_event(evt.as_mut_ptr()) };
        if popped == 0 {
            events += 1;
            let e = unsafe { evt.assume_init() };
            if e.header == efr32mg24_bluetooth_sys::ffi::sl_bt_evt_system_boot_id {
                sys_boot = 1;
                // `sl_bt_evt_system_boot_s`: header, then major/minor/patch/build as u16s, then the bootloader
                // version. Reading the major version is enough to prove a real boot event and not a stray one.
                let bytes = core::ptr::addr_of!(e.data) as *const u8;
                version = unsafe { core::ptr::read_volatile(bytes.add(4)) } as u32;
            }
        }
        delay.delay_ms(1);
    }

    let assert = efr32mg24_bluetooth_sys::rail_assert::last_assert();

    let mut m = [0u8; 180];
    let mut n = 0;
    n = label(&mut m, n, b"chan=");
    n = push_dec(&mut m, n, chan_status as u32);
    n = label(&mut m, n, b" boot=");
    n = push_hex(&mut m, n, boot);
    n = label(&mut m, n, b" sysboot=");
    n = push_dec(&mut m, n, sys_boot);
    n = label(&mut m, n, b" evt=");
    n = push_dec(&mut m, n, events);
    n = label(&mut m, n, b" major=");
    n = push_dec(&mut m, n, version);
    match assert {
        Some((code, _, count)) => {
            n = label(&mut m, n, b" assert=");
            n = push_dec(&mut m, n, code);
            n = label(&mut m, n, b"/");
            n = push_dec(&mut m, n, count);
        }
        None => {
            n = label(&mut m, n, b" assert=none");
        }
    }

    mailbox::write(
        if sys_boot != 0 {
            mailbox::Status::Pass
        } else {
            mailbox::Status::Fail
        },
        &m[..n],
    );

    loop {
        cortex_m::asm::wfi();
    }
}
