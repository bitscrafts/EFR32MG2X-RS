//! Make the board advertise "xiao-mg24", with RAIL and no Bluetooth stack.
//!
//! The receive example proved the channel configuration is accepted and the radio reaches RX. This one
//! uses the same configuration to transmit an advertising PDU, which is the actual deliverable: a name a
//! phone can see.
//!
//! ## What the radio builds and what we build
//!
//! RAIL's BLE PHY configuration supplies the preamble, the advertising access address (0x8E89BED6), the
//! CRC initialisation and the whitening. The application supplies the **PDU**: the advertiser address
//! followed by the advertising data. So the frame written here is:
//!
//! ```text
//! AdvA (6 bytes)  |  AD structures
//! ```
//!
//! and the AD structures are the same shape `scripts/host-advertise.sh` writes:
//!
//! ```text
//! 02 01 06                  flags: LE general discoverable, BR/EDR not supported
//! 0A 09 "xiao-mg24"         complete local name
//! ```
//!
//! ## The clock, which was the whole problem
//!
//! `RAIL_ASSERT_INVALID_XTAL_FREQUENCY` (66) came from the radio being clocked by the internal HFRCO at
//! 19 MHz while the configuration expects 39 MHz. Starting the crystal was not enough — an oscillator
//! nothing selects clocks nothing — so this example starts it *and* selects it, then reports the radio
//! clock RAIL actually sees. If that number is not 39000 kHz, the transmission is not on the frequency
//! the receiver expects and nothing else in this file can fix that.

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

extern "C" {
    static RAIL_BLE_Phy1Mbps: *const rail::RAIL_ChannelConfig_t;
    static RAIL_BLE_Phy1MbpsViterbi: *const rail::RAIL_ChannelConfig_t;
}

const NAME: &[u8] = b"xiao-mg24";
const STATUS_OK: u32 = 0;

#[no_mangle]
pub extern "C" fn RAILCb_AssertFailed(
    _rail_handle: rail::RAIL_Handle_t,
    error_code: rail::RAIL_AssertErrorCodes_t,
) -> ! {
    let mut msg = [0u8; 40];
    let mut at = 0;
    mailbox::push(&mut msg, &mut at, b"tx: assert=");
    at = push_dec(&mut msg, at, error_code as u32);
    mailbox::write(Status::Fail, &msg[..at]);
    loop {
        core::hint::spin_loop();
    }
}

#[entry]
fn main() -> ! {
    let board = efr32mg24_hal::board::Board::take().expect("board");
    let mut delay = board.delay();

    mailbox::write(Status::Info, b"tx: entered main");

    // ---------------------------------------------------------------- crystal, selected
    if !unsafe { efr32mg24_emlib_sys::emlib_hfxo_start() } {
        mailbox::write(Status::Fail, b"tx: HFXO did not start");
        stop(&mut delay);
    }
    unsafe { efr32mg24_emlib_sys::emlib_clock_select_hfxo() };

    // ---------------------------------------------------------------- radio
    let mut cfg: rail::RAIL_Config_t = unsafe { core::mem::zeroed() };
    let handle = unsafe { rail::RAIL_Init(&mut cfg, None) };
    if handle.is_null() {
        mailbox::write(Status::Fail, b"tx: RAIL_Init null");
        stop(&mut delay);
    }

    let radio_clock_khz = unsafe { rail::RAIL_GetRadioClockFreqHz(handle) } / 1000;
    let mut cm = [0u8; 40];
    let mut ca = 0;
    mailbox::push(&mut cm, &mut ca, b"tx: radioclk=");
    ca = push_dec(&mut cm, ca, radio_clock_khz);
    mailbox::write(Status::Info, &cm[..ca]);

    // Viterbi is the PHY this RAIL build supports (`rail_caps`: ble1m=0 vit=1). The air interface is
    // the same BLE 1 Mbps either way.
    let chan_cfg = unsafe { RAIL_BLE_Phy1MbpsViterbi };
    let plain = unsafe { RAIL_BLE_Phy1Mbps };
    let st = if !chan_cfg.is_null() {
        unsafe { rail::RAIL_ConfigChannels(handle, chan_cfg, None) }
    } else if !plain.is_null() {
        unsafe { rail::RAIL_ConfigChannels(handle, plain, None) }
    } else {
        mailbox::write(Status::Fail, b"tx: no BLE phy config");
        stop(&mut delay);
    };
    if st as u32 != STATUS_OK {
        report(&mut delay, b"tx: ConfigChannels=", st as u32);
    }
    let st = if chan_cfg.is_null() {
        unsafe { rail::RAIL_BLE_ConfigPhy1Mbps(handle) }
    } else {
        unsafe { rail::RAIL_BLE_ConfigPhy1MbpsViterbi(handle) }
    };
    // Advisory only: the channel configuration already carries the PHY, so RAIL answers
    // INVALID_STATE (2) to a redundant PHY selection. That is not a reason to stop — treating it as
    // fatal hid the transmit path behind a status code that meant "already done".
    if st as u32 != STATUS_OK && st as u32 != 2 {
        report(&mut delay, b"tx: ConfigPhy=", st as u32);
    }

    // ---------------------------------------------------------------- frame
    // AdvA: the six bytes above the UID's own tail, marked as a static random address by clearing the
    // two most significant bits of the top byte. A test beacon has no claim to a public address, and
    // claiming one would be a lie about a registered global identifier.
    let uid = unsafe { efr32mg24_emlib_sys::emlib_unique_id() };
    let mut pdu = [0u8; 40];
    let mut n = 0;
    for i in 0..6 {
        pdu[n] = ((uid >> (8 * (5 - i))) & 0xFF) as u8;
        n += 1;
    }
    pdu[5] |= 0xC0; // static random address

    pdu[n] = 0x02; // flags
    pdu[n + 1] = 0x01;
    pdu[n + 2] = 0x06;
    n += 3;
    pdu[n] = (NAME.len() + 1) as u8; // complete local name
    pdu[n + 1] = 0x09;
    n += 2;
    pdu[n..n + NAME.len()].copy_from_slice(NAME);
    n += NAME.len();
    let pdu_len = n;

    // TX buffer: RAIL copies from it, so it has to stay alive for as long as the radio uses it.
    let mut tx_buf = [0u8; 64];
    // RAIL_SetTxFifo(handle, buffer, initial length, total size): 0 bytes queued, 64 available.
    unsafe { rail::RAIL_SetTxFifo(handle, tx_buf.as_mut_ptr(), 0, 64) };
    let written =
        unsafe { rail::RAIL_WriteTxFifo(handle, pdu.as_ptr() as *const _, pdu_len as u16, true) };
    if written == 0 {
        mailbox::write(Status::Fail, b"tx: WriteTxFifo wrote 0");
        stop(&mut delay);
    }

    let mut m = [0u8; 48];
    let mut a = 0;
    mailbox::push(&mut m, &mut a, b"tx: pdu=");
    a = push_dec(&mut m, a, pdu_len as u32);
    mailbox::push(&mut m, &mut a, b"B clock=");
    a = push_dec(&mut m, a, radio_clock_khz);
    mailbox::write(Status::Info, &m[..a]);

    // ---------------------------------------------------------------- transmit, repeatedly
    // One advertisement is lost to any receiver that happens to be elsewhere, so this repeats on BLE
    // channel 37 (the configuration's logical channel 0) every ~100 ms, which is what a real advertiser
    // does. The mailbox is refreshed with the count so a reader can tell "transmitted 412 times" from
    // "transmitted once and stopped".
    let mut count = 0u32;
    let mut failures = 0u32;
    loop {
        // The FIFO is consumed by the transmission, so the frame has to be written again for every
        // advertisement. The first version wrote it once and called StartTx in a loop: it reported
        // `sent=1` and then nothing, which a scanner cannot tell apart from a radio that is not
        // transmitting — and is exactly what happened.
        let written = unsafe {
            rail::RAIL_WriteTxFifo(handle, pdu.as_ptr() as *const _, pdu_len as u16, true)
        };
        let st =
            unsafe { rail::RAIL_StartTx(handle, 0, rail::RAIL_TX_OPTIONS_DEFAULT, ptr::null()) };
        if st as u32 == STATUS_OK && written == pdu_len as u16 {
            count = count.wrapping_add(1);
        } else {
            failures = failures.wrapping_add(1);
        }

        let mut tm = [0u8; 56];
        let mut ta = 0;
        mailbox::push(&mut tm, &mut ta, b"tx: sent=");
        ta = push_dec(&mut tm, ta, count);
        mailbox::push(&mut tm, &mut ta, b" fail=");
        ta = push_dec(&mut tm, ta, failures);
        mailbox::push(&mut tm, &mut ta, b" st=");
        ta = push_dec(&mut tm, ta, st as u32);
        mailbox::write(Status::Pass, &tm[..ta]);

        delay.delay_ms(100);
    }
}

fn stop(delay: &mut efr32mg24_hal::delay::Delay) -> ! {
    loop {
        delay.delay_ms(1000);
    }
}

fn report(delay: &mut efr32mg24_hal::delay::Delay, prefix: &[u8], value: u32) {
    let mut msg = [0u8; 48];
    let mut at = 0;
    mailbox::push(&mut msg, &mut at, prefix);
    at = push_dec(&mut msg, at, value);
    mailbox::write(Status::Fail, &msg[..at]);
    let _ = delay;
    stop(delay);
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
