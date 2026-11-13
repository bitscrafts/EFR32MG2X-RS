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
    let handle = unsafe { rail::RAIL_Init(&mut cfg, None) };
    if handle.is_null() {
        mailbox::write(Status::Fail, b"tx: RAIL_Init returned null");
        stop(&mut delay);
    }

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

    // The power amplifier has no configuration until it is given one.
    let pa = rail::RAIL_TxPowerConfig_t {
        mode: rail::RAIL_TxPowerMode_t_enum_RAIL_TX_POWER_MODE_2P4_HP as u8,
        voltage: 3300,
        rampTime: 10,
    };
    let pa_st = unsafe { rail::RAIL_ConfigTxPower(handle, &pa) };
    let dbm_st = unsafe { rail::RAIL_SetTxPowerDbm(handle, 100) };

    // AdvA: the six bytes above the unique id, with the two most significant bits set to `11`, which is
    // what makes it a static random address. A phone shows the packet under this address.
    let uid = unsafe { efr32mg24_emlib_sys::emlib_unique_id() };
    let mut addr = [0u8; 6];
    for (i, b) in addr.iter_mut().enumerate() {
        *b = (uid >> (8 * (5 - i))) as u8;
    }
    addr[0] |= 0xC0;

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

    {
        let mut m = [0u8; 64];
        let mut at = 0;
        at = mailbox::push(&mut m, at, b"tx: pdu=");
        at = mailbox::push_dec(&mut m, at, pdu_len as u32);
        at = mailbox::push(&mut m, at, b" pa=");
        at = mailbox::push_dec(&mut m, at, pa_st as u32);
        at = mailbox::push(&mut m, at, b" dbm=");
        at = mailbox::push_dec(&mut m, at, dbm_st as u32);
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

        // No RAIL_SetFixedLength here on purpose: in BLE the length of a packet comes from the PDU
        // header (byte 1, 0x14), and forcing a fixed length contradicts it. Measured with it set: the
        // radio reported every transmission complete and a scanner saw nothing, which is what a frame
        // whose length disagrees with its own header looks like from the air.

        mailbox::write(Status::Info, b"tx: writing fifo");
        let written =
            unsafe { rail::RAIL_WriteTxFifo(handle, pdu.as_ptr() as *const _, pdu_len, true) };
        {
            let mut wm = [0u8; 32];
            let mut wa = 0;
            wa = mailbox::push(&mut wm, wa, b"tx: w=");
            wa = mailbox::push_dec(&mut wm, wa, written as u32);
            wa = mailbox::push(&mut wm, wa, b" st=");
            wa = mailbox::push_dec(&mut wm, wa, unsafe { rail::RAIL_GetRadioState(handle) }
                as u32);
            wa = mailbox::push(&mut wm, wa, b" ho=");
            wa = mailbox::push_dec(
                &mut wm,
                wa,
                unsafe { rail::RAIL_IsTxHoldOffEnabled(handle) } as u32,
            );
            mailbox::write(Status::Info, &wm[..wa]);
        }
        // A real advertiser rotates the three advertising channels (physical 0..2 are logical 37..39),
        // so a scanner listening on any of them sees the device. One fixed channel is invisible to a
        // scanner whose window is on another.
        let st = unsafe {
            rail::RAIL_StartTx(
                handle,
                (iterations % 3) as u16,
                rail::RAIL_TX_OPTIONS_DEFAULT,
                ptr::null(),
            )
        };
        if st as u32 == STATUS_OK && written == pdu_len {
            count = count.wrapping_add(1);
        } else {
            failures = failures.wrapping_add(1);
        }

        let mut rs = unsafe { rail::RAIL_GetRadioState(handle) } as u32;
        let tw = efr32mg24_hal::dwt::cycles();
        while (rs & RAIL_RF_STATE_TX) != 0 && efr32mg24_hal::dwt::since(tw) < 39_000 * 20 {
            rs = unsafe { rail::RAIL_GetRadioState(handle) } as u32;
        }

        mailbox::write(Status::Info, b"tx: polled");

        let mut tm = [0u8; 64];
        let mut ta = 0;
        ta = mailbox::push(&mut tm, ta, b"tx: iter=");
        ta = mailbox::push_dec(&mut tm, ta, iterations);
        ta = mailbox::push(&mut tm, ta, b" sent=");
        ta = mailbox::push_dec(&mut tm, ta, count);
        ta = mailbox::push(&mut tm, ta, b" fail=");
        ta = mailbox::push_dec(&mut tm, ta, failures);
        ta = mailbox::push(&mut tm, ta, b" st=");
        ta = mailbox::push_dec(&mut tm, ta, st as u32);
        ta = mailbox::push(&mut tm, ta, b" rs=");
        ta = mailbox::push_dec(&mut tm, ta, rs);
        mailbox::write(Status::Pass, &tm[..ta]);

        // 100 ms at the crystal's rate, on the cycle counter, because the HAL delay is calibrated for a
        // clock that is no longer the system clock.
        let t0 = efr32mg24_hal::dwt::cycles();
        while efr32mg24_hal::dwt::since(t0) < 39_000 * 100 {
            core::hint::spin_loop();
        }
    }
}
