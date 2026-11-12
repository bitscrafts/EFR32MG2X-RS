//! What can this RAIL library actually do?
//!
//! The channel configuration for BLE 1 Mbps came back null, and rail_ble.h says why that can happen:
//! the pointer "will be NULL if RAIL_BLE_SUPPORTS_1MBPS_NON_VITERBI is 0". A library built without a
//! protocol still exports the symbol — as a null pointer — so the failure is silent unless the support
//! flags are asked directly.
//!
//! This example asks them. It is the difference between "our code is wrong" and "this binary cannot do
//! what we are asking", and those two point at completely different next steps.

#![no_std]
#![no_main]

use efr32mg24_hal::entry;
use efr32mg24_hal::prelude::*;
use efr32mg24_rail_sys as rail;
use panic_halt as _;

#[path = "../../xiao-mg24-bsp/examples/common/mailbox.rs"]
mod mailbox;

use mailbox::Status;

extern "C" {
    /// `extern const RAIL_ChannelConfig_t *const RAIL_BLE_Phy1Mbps;`
    static RAIL_BLE_Phy1Mbps: *const rail::RAIL_ChannelConfig_t;
    /// `extern const RAIL_ChannelConfig_t *const RAIL_BLE_Phy1MbpsViterbi;`
    static RAIL_BLE_Phy1MbpsViterbi: *const rail::RAIL_ChannelConfig_t;
}

#[entry]
fn main() -> ! {
    let board = efr32mg24_hal::board::Board::take().expect("board");
    let mut delay = board.delay();

    // A capability answer does not need an initialised radio, but asking after init is the honest
    // order: an uninitialised library may report zeros for a different reason.
    let mut cfg: rail::RAIL_Config_t = unsafe { core::mem::zeroed() };
    let handle = unsafe { rail::RAIL_Init(&mut cfg, None) };
    if handle.is_null() {
        mailbox::write(Status::Fail, b"caps: RAIL_Init returned null");
        stop(&mut delay);
    }

    let ble_1m = unsafe { rail::RAIL_BLE_Supports1MbpsNonViterbi(handle) };
    let ble_1m_vit = unsafe { rail::RAIL_BLE_Supports1MbpsViterbi(handle) };
    let ble_2m = unsafe { rail::RAIL_BLE_Supports2MbpsNonViterbi(handle) };
    let ble_coded = unsafe { rail::RAIL_BLE_SupportsCodedPhy(handle) };
    let cfg_ptr_null = unsafe { RAIL_BLE_Phy1Mbps.is_null() };
    let cfg_vit_null = unsafe { RAIL_BLE_Phy1MbpsViterbi.is_null() };

    let mut msg = [0u8; 72];
    let mut at = 0;
    mailbox::push(&mut msg, &mut at, b"caps ble1m=");
    at = push_bit(&mut msg, at, ble_1m);
    mailbox::push(&mut msg, &mut at, b" vit=");
    at = push_bit(&mut msg, at, ble_1m_vit);
    mailbox::push(&mut msg, &mut at, b" 2m=");
    at = push_bit(&mut msg, at, ble_2m);
    mailbox::push(&mut msg, &mut at, b" coded=");
    at = push_bit(&mut msg, at, ble_coded);
    mailbox::push(&mut msg, &mut at, b" cfg_null=");
    at = push_bit(&mut msg, at, cfg_ptr_null);
    mailbox::push(&mut msg, &mut at, b" vit_null=");
    at = push_bit(&mut msg, at, cfg_vit_null);

    mailbox::write(Status::Info, &msg[..at]);

    // The crystal frequency each configuration was generated for. RAIL asserts
    // RAIL_ASSERT_INVALID_XTAL_FREQUENCY (66) when a configuration's value disagrees with the chip's
    // own, which is what happened when the BLE receive example tried to use the vendor config: the
    // question then is *which* number each side believes, and the config carries its own.
    let mut msg2 = [0u8; 64];
    let mut at2 = 0;
    mailbox::push(&mut msg2, &mut at2, b"xtal viterbi=");
    let vit_xtal = unsafe {
        if RAIL_BLE_Phy1MbpsViterbi.is_null() {
            0
        } else {
            (*RAIL_BLE_Phy1MbpsViterbi).xtalFrequencyHz
        }
    };
    at2 = push_dec(&mut msg2, at2, vit_xtal);
    mailbox::push(&mut msg2, &mut at2, b" plain=");
    let plain_xtal = unsafe {
        if RAIL_BLE_Phy1Mbps.is_null() {
            0
        } else {
            (*RAIL_BLE_Phy1Mbps).xtalFrequencyHz
        }
    };
    at2 = push_dec(&mut msg2, at2, plain_xtal);
    mailbox::write(Status::Info, &msg2[..at2]);

    // Does the crystal start at all? RAIL's assertion about crystal frequency is only meaningful if we
    // know which side is wrong.
    let hfxo_started = unsafe { efr32mg24_emlib_sys::emlib_hfxo_start() };
    let hfxo_hz = unsafe { efr32mg24_emlib_sys::emlib_hfxo_freq() };
    let sys_hz = unsafe { efr32mg24_emlib_sys::emlib_sysclk_hz() };

    let mut mx = [0u8; 64];
    let mut ax = 0;
    mailbox::push(&mut mx, &mut ax, b"hfxo started=");
    ax = push_bit(&mut mx, ax, hfxo_started);
    mailbox::push(&mut mx, &mut ax, b" hfxo=");
    ax = push_dec(&mut mx, ax, hfxo_hz / 1000);
    mailbox::push(&mut mx, &mut ax, b"kHz sys=");
    ax = push_dec(&mut mx, ax, sys_hz / 1000);
    mailbox::push(&mut mx, &mut ax, b"kHz");
    mailbox::write(Status::Info, &mx[..ax]);

    loop {
        delay.delay_ms(1000);
    }
}

fn stop(delay: &mut efr32mg24_hal::delay::Delay) -> ! {
    loop {
        delay.delay_ms(1000);
    }
}

fn push_dec(out: &mut [u8], mut at: usize, mut value: u32) -> usize {
    if value == 0 {
        out[at] = b'0';
        return at + 1;
    }
    let mut digits = [0u8; 10];
    let mut n = 0;
    while value > 0 && n < 10 {
        digits[n] = b'0' + (value % 10) as u8;
        value /= 10;
        n += 1;
    }
    while n > 0 {
        n -= 1;
        out[at] = digits[n];
        at += 1;
    }
    at
}

fn push_bit(out: &mut [u8], at: usize, value: bool) -> usize {
    out[at] = if value { b'1' } else { b'0' };
    at + 1
}
