//! Listen for BLE advertisements with RAIL, and report what arrives.
//!
//! The Bluetooth stack cannot be used on this part yet (see `BLE-BRINGUP-CHAIN.md`: it needs nvm3,
//! sleeptimer, the AES helper and PSA crypto). RAIL can, and the link layer is the part that actually
//! touches the antenna — so this example answers the question that matters for the board: *does the
//! radio receive a real frame from a real transmitter?*
//!
//! ## The channel
//!
//! BLE advertises on three channels that are not evenly spaced: 2402 MHz (channel 37), 2426 MHz (38) and
//! 2480 MHz (39), while the data channels sit 2 MHz apart from 2404. A single linear entry cannot express
//! that, so this example configures **one** logical channel mapped to 2402 MHz and listens there. The
//! host's advertisement from `scripts/host-advertise.sh` uses all three, so one is enough.
//!
//! ## What a "received" frame proves
//!
//! - the HFXO is running well enough for the radio to demodulate (RAIL brings it up);
//! - the channel configuration is right, because a wrong frequency receives nothing;
//! - the CRC of a real BLE advertisement matched — RAIL only hands over packets whose CRC passed unless
//!   told otherwise, so an arrival is a frame from a device that speaks the protocol, not noise.

#![no_std]
#![no_main]

use core::ptr;

use efr32mg24_hal::entry;
use efr32mg24_hal::prelude::*;
use efr32mg24_rail_sys as rail;
use panic_halt as _;

#[path = "../../xiao-mg24-bsp/examples/common/mailbox.rs"]
mod mailbox;

use mailbox::Status;

/// `RAIL_RX_PACKET_HANDLE_NEWEST` from rail_types.h, which bindgen does not export because it is a
/// macro cast to a pointer type.
const PACKET_HANDLE_NEWEST: rail::RAIL_RxPacketHandle_t = 3 as rail::RAIL_RxPacketHandle_t;
/// `RAIL_STATUS_NO_ERROR`.
const STATUS_OK: u32 = 0;

#[entry]
fn main() -> ! {
    let board = efr32mg24_hal::board::Board::take().expect("board");
    let mut delay = board.delay();

    mailbox::write(Status::Info, b"rx: entered main");

    // ---------------------------------------------------------------- crystal
    // The board runs from the internal HFRCO at 19 MHz, which is fine for a blink and useless to a
    // 2.4 GHz synthesizer. RAIL asserts RAIL_ASSERT_INVALID_XTAL_FREQUENCY (66) when the channel
    // configuration's crystal frequency disagrees with the chip's, and before this call the chip had
    // never started its crystal at all.
    // Register writes to the clock unit; the vendor function is `unsafe` by declaration.
    if !unsafe { efr32mg24_emlib_sys::emlib_hfxo_start() } {
        mailbox::write(Status::Fail, b"rx: HFXO did not start");
        stop(&mut delay);
    }
    let hfxo_hz = unsafe { efr32mg24_emlib_sys::emlib_hfxo_freq() };
    let mut m = [0u8; 48];
    let mut a = 0;
    mailbox::push(&mut m, &mut a, b"rx: hfxo=");
    a = push_dec(&mut m, a, hfxo_hz / 1000);
    mailbox::push(&mut m, &mut a, b"kHz");
    mailbox::write(Status::Info, &m[..a]);

    // ---------------------------------------------------------------- radio
    let mut cfg: rail::RAIL_Config_t = unsafe { core::mem::zeroed() };
    let handle = unsafe { rail::RAIL_Init(&mut cfg, None) };
    if handle.is_null() {
        mailbox::write(Status::Fail, b"rx: RAIL_Init returned null");
        stop(&mut delay);
    }

    // The RAIL library ships the whole channel configuration for BLE 1 Mbps — frequencies, channel
    // numbering, CRC init, whitening and the modem registers — as `RAIL_BLE_Phy1Mbps`. A first version
    // of this example built a channel config by hand with a null modem config, and RAIL answered with
    // an assertion (RAIL_ASSERT_FAILED_INVALID_CHANNEL_CONFIG, 65) instead of pretending: a channel
    // entry without a modem config names a frequency but not a modulation, and the radio will not
    // listen on an undefined PHY.
    extern "C" {
        /// `extern const RAIL_ChannelConfig_t *const RAIL_BLE_Phy1Mbps;` from rail_ble.h. It is a
        /// pointer: the symbol holds the address of the config, not the config itself.
        static RAIL_BLE_Phy1Mbps: *const rail::RAIL_ChannelConfig_t;
        /// The same for the Viterbi decoder variant.
        static RAIL_BLE_Phy1MbpsViterbi: *const rail::RAIL_ChannelConfig_t;
    }

    // Measured with `rail_caps`: this RAIL build reports `ble1m=0 vit=1`, i.e. it supports the 1 Mbps
    // PHY with the Viterbi decoder and not the non-Viterbi one. The signal on the air is the same BLE
    // 1 Mbps GFSK either way — the difference is how the receiver decodes it — so an advertisement from
    // a phone or from the host adapter is received by this configuration.
    let ble_chan_cfg = unsafe { RAIL_BLE_Phy1MbpsViterbi };
    if ble_chan_cfg.is_null() {
        // Falling back is only honest if the other block exists; saying which one is in use keeps the
        // reader from having to guess what the board heard with.
        let plain = unsafe { RAIL_BLE_Phy1Mbps };
        if plain.is_null() {
            mailbox::write(Status::Fail, b"rx: no BLE phy config in this RAIL build");
            stop(&mut delay);
        }
        mailbox::write(Status::Info, b"rx: using non-viterbi phy");
        unsafe { rail::RAIL_ConfigChannels(handle, plain, None) };
        unsafe { rail::RAIL_BLE_ConfigPhy1Mbps(handle) };
    } else {
        mailbox::write(Status::Info, b"rx: using viterbi phy");
        unsafe { rail::RAIL_ConfigChannels(handle, ble_chan_cfg, None) };
        unsafe { rail::RAIL_BLE_ConfigPhy1MbpsViterbi(handle) };
    }

    // Receive buffer. RAIL writes the frame here; 512 bytes is far more than any advertisement, and the
    // size is in/out so the value after the call is what RAIL actually took.
    let mut rx_buf = [0u8; 512];
    let mut rx_size: u16 = 512;
    unsafe { rail::RAIL_SetRxFifo(handle, rx_buf.as_mut_ptr(), &mut rx_size) };

    let st = unsafe { rail::RAIL_StartRx(handle, 37, ptr::null()) };
    if st as u32 != STATUS_OK {
        report_u32(&mut delay, b"rx: StartRx=", st as u32);
        stop(&mut delay);
    }

    mailbox::write(Status::Info, b"rx: listening on 2402");

    // ---------------------------------------------------------------- receive loop
    let mut seen = 0u16;
    loop {
        let mut info: rail::RAIL_RxPacketInfo_t = unsafe { core::mem::zeroed() };
        let pkt = unsafe { rail::RAIL_GetRxPacketInfo(handle, PACKET_HANDLE_NEWEST, &mut info) };
        if pkt.is_null() {
            // Nothing complete yet: reading RAM over SWD is slow compared with the radio, so this loop
            // is mostly this branch.
            core::hint::spin_loop();
            continue;
        }

        let mut details: rail::RAIL_RxPacketDetails_t = unsafe { core::mem::zeroed() };
        let _ = unsafe { rail::RAIL_GetRxPacketDetails(handle, pkt, &mut details) };

        // Report: logical channel, RSSI, length and the first bytes of the frame.
        let mut msg = [0u8; 72];
        let mut at = 0;
        mailbox::push(&mut msg, &mut at, b"rx ch=");
        at = push_dec(&mut msg, at, details.channel as u32);
        mailbox::push(&mut msg, &mut at, b" rssi=");
        at = push_i8(&mut msg, at, details.rssi);
        mailbox::push(&mut msg, &mut at, b" len=");
        at = push_dec(&mut msg, at, info.packetBytes as u32);
        mailbox::push(&mut msg, &mut at, b" d=");
        let n = core::cmp::min(info.packetBytes as usize, 12);
        let data = if info.firstPortionData.is_null() {
            &[][..]
        } else {
            unsafe {
                core::slice::from_raw_parts(info.firstPortionData, info.firstPortionBytes as usize)
            }
        };
        for byte in data.iter().take(n) {
            at = push_hex8(&mut msg, at, *byte);
        }
        mailbox::write(Status::Pass, &msg[..at]);

        seen = seen.wrapping_add(1);
        let _ = unsafe { rail::RAIL_ReleaseRxPacket(handle, pkt) };

        // Keep listening: one packet proves the path, several prove it repeats.
        let _ = seen;
    }
}

/// RAIL calls this when one of its internal assertions fails.
///
/// The library ships a weak default that loops forever, which is indistinguishable from a hang: the
/// first version of this example stopped after "entered main" and the PC was sitting in
/// `RAILCb_AssertFailed` with nothing to say. Overriding it turns that silence into an error code.
#[no_mangle]
pub extern "C" fn RAILCb_AssertFailed(
    _rail_handle: rail::RAIL_Handle_t,
    error_code: rail::RAIL_AssertErrorCodes_t,
) -> ! {
    let mut msg = [0u8; 40];
    let mut at = 0;
    mailbox::push(&mut msg, &mut at, b"rx: assert=");
    at = push_dec(&mut msg, at, error_code as u32);
    mailbox::write(Status::Fail, &msg[..at]);
    loop {
        core::hint::spin_loop();
    }
}

fn stop(delay: &mut efr32mg24_hal::delay::Delay) -> ! {
    loop {
        delay.delay_ms(1000);
    }
}

fn report_u32(delay: &mut efr32mg24_hal::delay::Delay, prefix: &[u8], value: u32) {
    let mut msg = [0u8; 48];
    let mut at = 0;
    mailbox::push(&mut msg, &mut at, prefix);
    at = push_dec(&mut msg, at, value);
    mailbox::write(Status::Fail, &msg[..at]);
    let _ = delay;
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

fn push_i8(out: &mut [u8], mut at: usize, value: i8) -> usize {
    if value < 0 {
        out[at] = b'-';
        at += 1;
        return push_dec(out, at, (-(value as i32)) as u32);
    }
    push_dec(out, at, value as u32)
}

fn push_hex8(out: &mut [u8], mut at: usize, value: u8) -> usize {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    out[at] = HEX[(value >> 4) as usize];
    out[at + 1] = HEX[(value & 0xF) as usize];
    at + 2
}
