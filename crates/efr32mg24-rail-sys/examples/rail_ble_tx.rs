//! Make the board advertise `xiao-mg24` with RAIL, following the sequence the vendor documents.
//!
//! `rail_ble.h` carries a worked example, and it is the authority for this file. Two calls from it were
//! missing here, and between them they explain every measurement that failed before this:
//!
//! ```c
//! gRailHandle = RAIL_Init(&railCfg, NULL);
//! RAIL_BLE_Init(gRailHandle);                   // loads the BLE radio configuration
//! RAIL_BLE_ConfigPhy1MbpsViterbi(gRailHandle);  // Viterbi is what this part supports
//! RAIL_BLE_ConfigChannelRadioParams(gRailHandle, 0x555555, 0x8E89BED6, 37, false);
//! RAIL_StartRx(gRailHandle, 0, NULL);           // physical 0 is logical 37
//! ```
//!
//! Without `RAIL_BLE_Init` the radio has no BLE protocol state at all: `RAIL_StartTx` returns success,
//! the radio enters TX and never leaves it (`rs=4`), and the board looks silent while the API reports
//! nothing wrong. Without `RAIL_BLE_ConfigChannelRadioParams` there is no CRC init and no advertising
//! access address, so even a transmission that completed would not be a BLE packet.
//!
//! There is no hand-built channel configuration here either. `RAIL_BLE_Init` loads the vendor's own,
//! which is the piece a hand-built one could never supply: the modem register tables.
//!
//! The frame is a legacy advertising PDU:
//!
//! ```text
//! 40 14 | E0 3E 13 FE FF 75 | 02 01 06 | 0A 09 "xiao-mg24"
//! ^^^^^   ^^^^^^^^^^^^^^^^^^   ^^^^^^^^   ^^^^^^^^^^^^^^^^^^^
//! header  AdvA, static random  flags      complete local name (9 bytes)
//! ```
//!
//! Header byte 0 is PDU type 0 (ADV_IND) with TxAdd set, because the address is random; byte 1 is the
//! payload length: 6 bytes of address + 14 bytes of advertising data = 20.

#![no_std]
#![no_main]

use core::ptr;

mod mailbox {
    //! A record the host reads over SWD while the core runs. The radio cannot print, and a serial port
    //! would be a second thing to be wrong: the mailbox is one word-magic and a length, nothing else.
    use core::ptr;

    pub const ADDR: usize = 0x2003_F000;
    const MAGIC: u32 = 0x4D42_4F58; // "MBOX"

    #[derive(Clone, Copy)]
    pub enum Status {
        Info = 0,
        Pass = 1,
        Fail = 2,
    }

    /// Publish a message. The magic is cleared first and set last, so a reader never sees a torn record.
    pub fn write(status: Status, bytes: &[u8]) {
        let len = bytes.len().min(200);
        unsafe {
            let base = ADDR as *mut u32;
            ptr::write_volatile(base, 0);
            let seq = ptr::read_volatile(base.add(1)).wrapping_add(1);
            ptr::write_volatile(base.add(1), seq);
            ptr::write_volatile(base.add(2), status as u32);
            ptr::write_volatile(base.add(3), len as u32);
            for (i, b) in bytes[..len].iter().enumerate() {
                ptr::write_volatile((ADDR as *mut u8).add(16 + i), *b);
            }
            ptr::write_volatile(base, MAGIC);
        }
    }

    /// Append a byte string, refusing to overflow the record.
    pub fn push(dst: &mut [u8], mut at: usize, s: &[u8]) -> usize {
        for &b in s {
            if at < dst.len() {
                dst[at] = b;
                at += 1;
            }
        }
        at
    }

    /// Append a decimal number without `format!`.
    pub fn push_dec(dst: &mut [u8], mut at: usize, mut v: u32) -> usize {
        let mut buf = [0u8; 10];
        let mut n = 0;
        if v == 0 {
            return push(dst, at, b"0");
        }
        while v > 0 {
            buf[n] = b'0' + (v % 10) as u8;
            v /= 10;
            n += 1;
        }
        while n > 0 {
            n -= 1;
            at = push(dst, at, &buf[n..n + 1]);
        }
        at
    }
}

use efr32mg24_hal::prelude::*;
use efr32mg24_rail_sys as rail;
use mailbox::Status;
use panic_halt as _;

const STATUS_OK: u32 = 0;
const RAIL_RF_STATE_TX: u32 = 0x04;

/// RAIL calls this when an internal assertion fails. Its weak default spins with no information; this
/// one puts the assertion code in the mailbox, which is how `assert=65` and `assert=66` were found.
#[no_mangle]
pub extern "C" fn RAILCb_AssertFailed(
    _rail_handle: rail::RAIL_Handle_t,
    error_code: rail::RAIL_AssertErrorCodes_t,
) -> ! {
    let mut m = [0u8; 32];
    let mut at = 0;
    at = mailbox::push(&mut m, at, b"tx: assert=");
    at = mailbox::push_dec(&mut m, at, error_code as u32);
    mailbox::write(Status::Fail, &m[..at]);
    loop {
        core::hint::spin_loop();
    }
}

/// Packets the RADIO reports as sent, counted in the events callback. The loop's own counter counts
/// calls to `RAIL_StartTx`; this one counts interrupts raised by the radio, and the two disagreeing is
/// exactly the case where the API looks fine and the air stays empty.
static mut TX_PACKET_SENT: u32 = 0;

/// RAIL events. Runs in interrupt context.
unsafe extern "C" fn on_rail_event(_handle: rail::RAIL_Handle_t, events: rail::RAIL_Events_t) {
    // bit 24 of RAIL_Events_t is RAIL_EVENT_TX_PACKET_SENT
    if (events & (1u64 << 24)) != 0 {
        TX_PACKET_SENT = TX_PACKET_SENT.wrapping_add(1);
    }
}

extern "C" {
    /// Loads the PA power curves into RAIL (`RAIL_InitTxPowerCurvesAlt`) and optionally enables PA
    /// calibration. Documented inside pa_conversions_efr32.c: without this call the dBm-to-raw conversion
    /// finds no curve and returns 255, "invalid", so the amplifier never gets a level -- while every
    /// transmission still completes at the modem.
    fn sl_rail_util_pa_init();
    /// The vendor's own 2.4 GHz PA configuration for this chip family (mode, voltage, ramp time).
    fn sl_rail_util_pa_get_tx_power_config_2p4ghz() -> *mut rail::RAIL_TxPowerConfig_t;
}

/// RAIL's power-amplifier callback. The Bluetooth stack provides its own policy; a beacon does not need
/// one, but the symbol has to exist or the link fails.
#[no_mangle]
pub extern "C" fn RAILCb_PaAutoModeDecision(_handle: rail::RAIL_Handle_t) {}

fn stop(delay: &mut efr32mg24_hal::delay::Delay) -> ! {
    loop {
        delay.delay_ms(500);
    }
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    let mut board = efr32mg24_hal::board::Board::take().unwrap();
    let mut delay = board.delay();

    // The radio is clocked from the crystal, not from the internal oscillator. Starting it is not enough:
    // it has to be selected, or RAIL keeps running from the HFRCO and refuses the BLE configuration
    // (RAIL_ASSERT_INVALID_XTAL_FREQUENCY, measured as `radioClock=19000kHz cfg=39000kHz`).
    unsafe {
        efr32mg24_emlib_sys::emlib_hfxo_start();
        efr32mg24_emlib_sys::emlib_clock_select_hfxo();
    }
    // Selecting SYSCLK stops SysTick, so the HAL's delay is rebuilt against the new clock. It is still
    // calibrated for 19 MHz, which is the known delay defect: the interval below is measured on the
    // cycle counter instead.
    let mut delay = board.delay();

    let mut cfg: rail::RAIL_Config_t = unsafe { core::mem::zeroed() };
    cfg.eventsCallback = Some(on_rail_event);
    let handle = unsafe { rail::RAIL_Init(&mut cfg, None) };
    if handle.is_null() {
        mailbox::write(Status::Fail, b"tx: RAIL_Init returned null");
        stop(&mut delay);
    }

    // Ask for the send-complete event explicitly rather than relying on defaults.
    unsafe { rail::RAIL_ConfigEvents(handle, 1u64 << 24, 1u64 << 24) };

    // The documented sequence, in the documented order.
    unsafe { rail::RAIL_BLE_Init(handle) };
    let phy = unsafe { rail::RAIL_BLE_ConfigPhy1MbpsViterbi(handle) };
    // CRC init and advertising access address from the specification; logical channel 37.
    let ch = unsafe {
        rail::RAIL_BLE_ConfigChannelRadioParams(handle, 0x0055_5555, 0x8E89_BED6, 37, false)
    };

    if phy as u32 != STATUS_OK || ch as u32 != STATUS_OK {
        let mut m = [0u8; 48];
        let mut at = 0;
        at = mailbox::push(&mut m, at, b"tx: phy=");
        at = mailbox::push_dec(&mut m, at, phy as u32);
        at = mailbox::push(&mut m, at, b" ch=");
        at = mailbox::push_dec(&mut m, at, ch as u32);
        mailbox::write(Status::Fail, &m[..at]);
        stop(&mut delay);
    }

    // One packet per transmission, not a continuous stream. In FIFO_MODE a radio that runs out of data
    // keeps transmitting, which is what `rs=4` measured before this was set.
    let dc = rail::RAIL_DataConfig_t {
        txSource: 0, // TX_PACKET_DATA
        rxSource: 0, // RX_PACKET_DATA
        txMethod: 0, // PACKET_MODE
        rxMethod: 0, // PACKET_MODE
    };
    unsafe { rail::RAIL_ConfigData(handle, &dc) };

    // The power amplifier, in the order the vendor's own plugin expects:
    //   1. sl_rail_util_pa_init()  -- loads the PA curves into RAIL
    //   2. the plugin's own 2.4 GHz configuration (mode HIGHEST, 3300 mV, 10 us ramp)
    //   3. the dBm value
    // Skipping step 1 was measured as RAIL_SetTxPowerDbm returning success and RAIL_GetTxPowerDbm reading
    // back -32768 (invalid power) with a raw level of 0: the transmitter drives nothing and every packet
    // still completes at the modem.
    unsafe { sl_rail_util_pa_init() };
    let pa_cfg = unsafe { sl_rail_util_pa_get_tx_power_config_2p4ghz() };
    let pa_st = if pa_cfg.is_null() {
        1
    } else {
        unsafe { rail::RAIL_ConfigTxPower(handle, pa_cfg) }
    };
    // 20 dBm: the highest this PA supports (the xg24 HIGHEST mode). 10 dBm measured -83 dBm at the
    // scanning host, which is a weak advertisement for a phone to catch reliably.
    let dbm_st = unsafe { rail::RAIL_SetTxPowerDbm(handle, 200) };

    // AdvA: a fixed STATIC RANDOM address, sent least-significant byte first, which is what the scanner
    // shows reversed as C0:DE:00:00:00:01. Random rather than public on purpose: a public address claims
    // an IEEE OUI this project does not own, and the static-random rule (top two bits of the most
    // significant byte set to 11) is exactly satisfied by 0xC0.
    let addr: [u8; 6] = [0x01, 0x00, 0x00, 0x00, 0xDE, 0xC0];

    let mut pdu = [0u8; 40];
    let mut n = 0;
    pdu[n] = 0x40; // ADV_IND, TxAdd=1 (random address)
    n += 1;
    pdu[n] = 20; // payload length: 6 address + 14 advertising data
    n += 1;
    pdu[n..n + 6].copy_from_slice(&addr);
    n += 6;
    pdu[n..n + 3].copy_from_slice(&[0x02, 0x01, 0x06]); // flags: LE general discoverable, no BR/EDR
    n += 3;
    pdu[n..n + 2].copy_from_slice(&[0x0A, 0x09]); // complete local name, 9 bytes
    n += 2;
    pdu[n..n + 9].copy_from_slice(b"xiao-mg24");
    n += 9;
    let pdu_len = n as u16;

    let mut path: rail::RAIL_AntennaSel_t = unsafe { core::mem::zeroed() };
    let path_st = unsafe { rail::RAIL_GetRfPath(handle, &mut path) };
    let read_dbm = unsafe { rail::RAIL_GetTxPowerDbm(handle) };
    let raw_level = unsafe { rail::RAIL_GetTxPower(handle) } as u32;

    // Kept in their own words so the loop's counters cannot overwrite them before the host reads.
    unsafe {
        core::ptr::write_volatile(0x2003_F100 as *mut u32, read_dbm as u32);
        core::ptr::write_volatile(0x2003_F104 as *mut u32, path_st as u32);
        core::ptr::write_volatile(0x2003_F108 as *mut u32, path as u32);
        core::ptr::write_volatile(0x2003_F10C as *mut u32, raw_level);
    }

    {
        let mut m = [0u8; 64];
        let mut at = 0;
        at = mailbox::push(&mut m, at, b"tx: pdu=");
        at = mailbox::push_dec(&mut m, at, pdu_len as u32);
        at = mailbox::push(&mut m, at, b" pa=");
        at = mailbox::push_dec(&mut m, at, pa_st as u32);
        at = mailbox::push(&mut m, at, b" dbm=");
        at = mailbox::push_dec(&mut m, at, dbm_st as u32);
        at = mailbox::push(&mut m, at, b" rd=");
        at = mailbox::push_dec(&mut m, at, read_dbm as u32);
        at = mailbox::push(&mut m, at, b" path=");
        at = mailbox::push_dec(&mut m, at, path_st as u32);
        at = mailbox::push(&mut m, at, b"/");
        at = mailbox::push_dec(&mut m, at, path as u32);
        mailbox::write(Status::Info, &m[..at]);
    }

    // The bytes the radio was actually given, copied to a fixed address so the host can read them back
    // off the chip. A source constant proves nothing about what reached the FIFO.
    unsafe {
        let dst = 0x2003_F080 as *mut u8;
        for (i, b) in pdu[..pdu_len as usize].iter().enumerate() {
            core::ptr::write_volatile(dst.add(i), *b);
        }
    }

    let mut tx_buf = [0u8; 64];
    unsafe { rail::RAIL_SetTxFifo(handle, tx_buf.as_mut_ptr(), 0, 64) };
    mailbox::write(Status::Info, b"tx: fifo ok");

    unsafe { efr32mg24_hal::dwt::start() };

    let mut count = 0u32;
    let mut failures = 0u32;
    let mut iterations = 0u32;
    loop {
        iterations = iterations.wrapping_add(1);

        // A real advertiser sends on the three advertising channels in turn, back to back, and then
        // waits. One channel per 100 ms round -- the earlier shape -- gives each channel a packet only
        // every 300 ms, which a phone's scan window can miss for a long time. The radio parameters are
        // re-applied per channel because the whitening seed is the channel number.
        // RAIL's channel argument is the PHYSICAL channel: `railChannel = 0` is `logicalChannel = 37`
        // (rail_ble.h, the repeated-transmit example). Physical N is 2402 + 2N MHz, so the three
        // advertising channels are (0, 37), (12, 38) and (39, 39). Passing 37/38/39 as we did transmitted
        // on 2476/2478/2480 MHz -- data channels except the last -- which is why only one third of the
        // packets were on a channel any scanner looks at.
        for (channel, logical) in [(0u16, 37u16), (12, 38), (39, 39)] {
            unsafe {
                rail::RAIL_BLE_ConfigChannelRadioParams(
                    handle,
                    0x0055_5555,
                    0x8E89_BED6,
                    logical,
                    false,
                )
            };
            let prep = unsafe { rail::RAIL_PrepareChannel(handle, channel) };

            let written =
                unsafe { rail::RAIL_WriteTxFifo(handle, pdu.as_ptr() as *const _, pdu_len, true) };

            // No RAIL_SetFixedLength here: in BLE the length comes from the PDU header (byte 1, 0x14),
            // and forcing a fixed length contradicts it.
            let st = unsafe {
                rail::RAIL_StartTx(handle, channel, rail::RAIL_TX_OPTIONS_DEFAULT, ptr::null())
            };

            if st as u32 == STATUS_OK && written == pdu_len {
                count = count.wrapping_add(1);
            } else {
                failures = failures.wrapping_add(1);
            }
            if prep as u32 != STATUS_OK {
                failures = failures.wrapping_add(1);
            }
        }

        let mut tm = [0u8; 64];
        let mut ta = 0;
        ta = mailbox::push(&mut tm, ta, b"tx: rounds=");
        ta = mailbox::push_dec(&mut tm, ta, iterations);
        ta = mailbox::push(&mut tm, ta, b" sent=");
        ta = mailbox::push_dec(&mut tm, ta, count);
        ta = mailbox::push(&mut tm, ta, b" fail=");
        ta = mailbox::push_dec(&mut tm, ta, failures);
        ta = mailbox::push(&mut tm, ta, b" hw=");
        ta = mailbox::push_dec(&mut tm, ta, unsafe { TX_PACKET_SENT });
        mailbox::write(Status::Pass, &tm[..ta]);

        // 30 ms at the crystal's rate, measured on the cycle counter because the HAL's Delay is
        // calibrated for a clock that is no longer the system clock.
        let t0 = efr32mg24_hal::dwt::cycles();
        while efr32mg24_hal::dwt::since(t0) < 39_000 * 30 {
            core::hint::spin_loop();
        }
    }
}
