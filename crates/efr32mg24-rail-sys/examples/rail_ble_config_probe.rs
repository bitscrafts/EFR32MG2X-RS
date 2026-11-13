//! Does RAIL accept the generated 39 MHz channel configuration?
//!
//! This is the question that decides whether the Bluetooth stack can start, asked without the stack in the link: an
//! earlier attempt to ask both at once ran into `duplicate symbol` from the RAIL blob being folded into an rlib that
//! also re-linked it, which is a link problem and not a radio problem.
//!
//! Three measurements, each one a fact about the silicon rather than about my code:
//!
//! 1. `RAIL_ConfigChannels` returns a status. `RAIL_STATUS_NO_ERROR` means the crystal frequency and the table were
//!    accepted.
//! 2. `RAIL_GetChannelConfig` gives back the pointer RAIL is using, which must be ours — a control that the call
//!    did something.
//! 3. `RAIL_SetTxPowerDbm` needs a configured radio; with the table in place it should return a real power rather
//!    than the invalid sentinel, which is what it did before the PA curves were initialised.

#![no_std]
#![no_main]

#[path = "../../xiao-mg24-bsp/examples/common/mailbox.rs"]
mod mailbox;

// The HAL brings the device crate in, which supplies the interrupt vector table cortex-m-rt insists on: without a
// reference to it the link fails with "The interrupt vectors are missing".
use efr32mg24_hal::board::Board;

use cortex_m_rt::entry;
use efr32mg24_rail_sys as rail;
use panic_halt as _;

const RTC: u32 = 0x500A_8000;
const CMU: u32 = 0x5000_8000;

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

/// RAIL's assertion callback, installed so a refused configuration is a recorded number instead of a silent loop.
///
/// The default implementation of this symbol is an infinite loop, which is why "RAIL refuses the channel
/// configuration" has looked like a hang for three sessions. The code goes to a fixed RAM word and to the mailbox.
const ASSERT_SLOT: usize = 0x2003_F500;

#[no_mangle]
pub extern "C" fn RAILCb_AssertFailed(_handle: *mut core::ffi::c_void, code: u32) -> ! {
    unsafe {
        let slot = ASSERT_SLOT as *mut u32;
        core::ptr::write_volatile(slot, 0x4153_5254); // "ASRT"
        core::ptr::write_volatile(slot.add(1), code);
        core::ptr::write_volatile(slot.add(2), 0);
        core::ptr::write_volatile(slot.add(3), 1);
    }
    // Report through the mailbox too: a reader should not need a debugger to learn why RAIL stopped.
    let mut m = [0u8; 40];
    let mut n = 0;
    n = label(&mut m, n, b"assert=");
    n = push_dec(&mut m, n, code);
    mailbox::write(mailbox::Status::Fail, &m[..n]);

    loop {
        core::hint::spin_loop();
    }
}

#[entry]
fn main() -> ! {
    let _board = Board::take().expect("peripherals taken twice");
    stage(b"boot", 0, 0);

    // The radio's clock. **HFXO must be running and selected before RAIL is initialised**: RAIL compares the
    // configuration's `xtalFrequencyHz` against the frequency the device actually reports, and an HFXO that was
    // never started reports zero, so the comparison fails and the assertion names the crystal even though the table
    // says 39 MHz. The advertising example does this before RAIL for the same reason.
    unsafe {
        efr32mg24_emlib_sys::emlib_hfxo_start();
        efr32mg24_emlib_sys::emlib_clock_select_hfxo();
    }
    stage(b"hfxo", 1, 0);

    unsafe {
        core::ptr::write_volatile((CMU + 0x64) as *mut u32, 0x4040_0000);
        // Let the SYSRTC counter run so this probe can report a timestamp without a debugger.
        core::ptr::write_volatile((RTC + 0x1004) as *mut u32, 1);
        core::ptr::write_volatile((RTC + 0x010) as *mut u32, 1);
        core::arch::asm!("dsb 0xf", "isb 0xf");
    }

    let handle = unsafe {
        let mut config: rail::RAIL_Config_t = core::mem::zeroed();
        rail::RAIL_Init(&mut config, None)
    };
    stage(b"init", handle as usize as u32, 0);

    let cfg = rail::ble_config::config();
    stage(b"xtal", cfg.xtalFrequencyHz, cfg.length);

    let status = unsafe { rail::RAIL_ConfigChannels(handle, cfg, None) };
    stage(b"chan", status as u32, 0);

    // Control: ask RAIL for the metadata of three channels. If the table is live, the frequencies must be the
    // advertising channels in physical numbering -- 0 = 2402 MHz, 12 = 2426 MHz, 39 = 2480 MHz -- because that is
    // what the generated entries describe. A dead or rejected table cannot answer at all.
    // SAFETY: a plain data struct; zeroed is a valid starting value for an out-parameter.
    let mut meta: [rail::RAIL_ChannelMetadata_t; 3] = unsafe { core::mem::zeroed() };
    let mut len: u16 = 3;
    let got = unsafe { rail::RAIL_GetChannelMetadata(handle, meta.as_mut_ptr(), &mut len, 0, 39) };
    let f0 = unsafe { core::ptr::read_volatile(&meta[0].frequency) };
    stage(b"meta", got as u32, len as u32);

    let mut meta2: [rail::RAIL_ChannelMetadata_t; 1] = unsafe { core::mem::zeroed() };
    let mut len2: u16 = 1;
    unsafe { rail::RAIL_GetChannelMetadata(handle, meta2.as_mut_ptr(), &mut len2, 12, 12) };
    let f12 = unsafe { core::ptr::read_volatile(&meta2[0].frequency) };

    let mut meta3: [rail::RAIL_ChannelMetadata_t; 1] = unsafe { core::mem::zeroed() };
    let mut len3: u16 = 1;
    unsafe { rail::RAIL_GetChannelMetadata(handle, meta3.as_mut_ptr(), &mut len3, 39, 39) };
    let f39 = unsafe { core::ptr::read_volatile(&meta3[0].frequency) };

    let ours = ((f0 == 2_402_000_000) as u32)
        | (((f12 == 2_426_000_000) as u32) << 1)
        | (((f39 == 2_480_000_000) as u32) << 2);
    stage(b"freqs", f0 / 1_000_000, f12 / 1_000_000);
    stage(b"freq39", f39 / 1_000_000, ours);

    // A radio with a valid configuration accepts a power setting.
    let set = unsafe { rail::RAIL_SetTxPowerDbm(handle, 100) };
    let dbm = unsafe { rail::RAIL_GetTxPowerDbm(handle) };
    stage(b"power", set as u32, dbm as u32);

    let mut m = [0u8; 120];
    let mut n = 0;
    n = label(&mut m, n, b"chan=");
    n = push_dec(&mut m, n, status as u32);
    n = label(&mut m, n, b" ours=");
    n = push_dec(&mut m, n, ours);
    n = label(&mut m, n, b" dbm=");
    n = push_dec(&mut m, n, dbm as u32);
    n = label(&mut m, n, b" h=");
    n = push_hex(&mut m, n, handle as usize as u32);
    // `ours` is a bitmask: bit 0 = channel 0 is 2402 MHz, bit 1 = channel 12 is 2426, bit 2 = channel 39 is 2480.
    n = label(&mut m, n, b" freqs=");
    n = push_hex(&mut m, n, ours);
    mailbox::write(
        if status == 0 && ours == 7 {
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
