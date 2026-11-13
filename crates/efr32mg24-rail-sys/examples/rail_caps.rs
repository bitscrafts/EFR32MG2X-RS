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

    // What the radio clock actually is, as RAIL sees it.
    //
    // The first measurement asked `CMU_ClockFreqGet(cmuClock_HFXO)`, which returns 0 on this part —
    // emlib's switch has no case for that clock id, so the zero said nothing about the crystal. RAIL's
    // own answer is the one that matters, because RAIL is what asserts
    // RAIL_ASSERT_INVALID_XTAL_FREQUENCY when it disagrees with a channel configuration.
    let radio_clock_hz = unsafe { rail::RAIL_GetRadioClockFreqHz(handle) };
    let hfxo_started = unsafe { efr32mg24_emlib_sys::emlib_hfxo_start() };

    let mut mx = [0u8; 72];
    let mut ax = 0;
    mailbox::push(&mut mx, &mut ax, b"radioClock=");
    ax = push_dec(&mut mx, ax, radio_clock_hz / 1000);
    mailbox::push(&mut mx, &mut ax, b"kHz cfg=");
    let cfg_xtal = unsafe {
        if ble_1m_vit {
            (*RAIL_BLE_Phy1MbpsViterbi).xtalFrequencyHz
        } else {
            0
        }
    };
    ax = push_dec(&mut mx, ax, cfg_xtal / 1000);
    mailbox::push(&mut mx, &mut ax, b"kHz hfxo_rdy=");
    ax = push_bit(&mut mx, ax, hfxo_started);
    mailbox::write(Status::Info, &mx[..ax]);

    // What channels does the vendor configuration actually expose? Listening on "37" is only right if
    // this table numbers the advertising channels 37/38/39; RAIL's logical numbering is not guaranteed to
    // match the Bluetooth channel index.
    let mut mc = [0u8; 96];
    let mut ac = 0;
    unsafe {
        if !RAIL_BLE_Phy1MbpsViterbi.is_null() {
            let cfg = &*RAIL_BLE_Phy1MbpsViterbi;
            let n = core::cmp::min(cfg.length, 4) as usize;
            for i in 0..n {
                let e = &*cfg.configs.add(i);
                mailbox::push(&mut mc, &mut ac, b"e");
                ac = push_dec(&mut mc, ac, i as u32);
                mailbox::push(&mut mc, &mut ac, b" base=");
                ac = push_dec(&mut mc, ac, e.baseFrequency / 1_000_000);
                mailbox::push(&mut mc, &mut ac, b"MHz sp=");
                ac = push_dec(&mut mc, ac, e.channelSpacing / 1_000);
                mailbox::push(&mut mc, &mut ac, b"k ch=");
                ac = push_dec(&mut mc, ac, e.channelNumberStart as u32);
                mailbox::push(&mut mc, &mut ac, b"-");
                ac = push_dec(&mut mc, ac, e.channelNumberEnd as u32);
                mailbox::push(&mut mc, &mut ac, b" ");
            }
        }
    }
    mailbox::write(Status::Info, &mc[..ac]);

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
