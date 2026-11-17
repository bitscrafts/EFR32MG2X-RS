//! Erase the NVM3 region, and nothing else.
//!
//! Why this exists: on 2026-11-17 the board stopped booting after a sequence of pairing attempts — the core was found in
//! `RAILCb_AssertFailed` with the mailbox holding garbage, in two different examples, so the fault was in the BLE stack's
//! start-up rather than in any one program. The stack loads its PS keys and bond records from NVM3 at boot, and a torn
//! write there (a pairing refused mid-exchange) is exactly the state that makes it assert.
//!
//! The tools that are usually reached for are not acceptable here: `probe-rs erase` erases **all** non-volatile memory,
//! which includes the Gecko bootloader at `0x08000000-0x08005FFF` — the one region this project must never touch, because
//! writing it bricks the board. `probe-rs write` to a flash address does not land (the page must be erased first, and the
//! write path is the MSC), and it reported success while leaving the bytes unchanged.
//!
//! So the erase is done by the chip itself, from this program, over exactly five pages, with a guard that refuses any
//! address below the application floor. It runs once and then halts: this is a recovery tool, not a firmware.
//!
//! Reflash the real firmware afterwards (`examples/module_ble_dosimeter.rs`).

#![no_std]
#![no_main]

use cortex_m_rt::entry;
// A crate the linker would otherwise drop.
//
// `MSC_ErasePage` lives in the emlib static library, which is linked as a native dependency of the Bluetooth stack
// crate. This program does not touch the stack, so without a reference the native library is never pulled in and the
// link fails with "undefined symbol: MSC_ErasePage" -- measured here, after a build that compiled the C fine.
#[allow(unused_imports)]
use efr32mg24_radio as _;
use panic_halt as _;
use xiao_mg24_bsp::prelude::*;
use xiao_mg24_bsp::Module;

extern "C" {
    /// `em_msc.c`: erase the flash page containing `address`, returns 0 on success.
    fn MSC_ErasePage(address: u32) -> u32;
}

/// First byte of the NVM3 region: the application ends where this begins (`memory.x`: FLASH 0x08006000 + 1472K).
const NVM3_START: u32 = 0x0817_6000;
/// One past the last byte of NVM3 (40 KB, `NVM3` in `crates/efr32mg2x-hal/README.md`).
const NVM3_END: u32 = 0x0818_0000;
/// The erase granularity on this family.
const PAGE: u32 = 0x2000;
/// The Gecko bootloader ends here. Nothing in this program may touch an address below it, and this constant is checked
/// before every erase rather than trusted by construction.
const FLOOR: u32 = 0x0800_6000;

#[entry]
fn main() -> ! {
    let mut module = match Module::take() {
        Some(m) => m,
        None => loop {
            cortex_m::asm::wfi();
        },
    };
    let mut delay = module.delay();
    let led = module.led();

    let mut addr = NVM3_START;
    let mut apagadas = 0u32;
    let mut falhas = 0u32;

    while addr < NVM3_END {
        // The guard: an address below the floor is the bootloader, and a bug that computed one would brick the board.
        assert!(
            addr >= FLOOR,
            "refusing to erase below the application floor"
        );
        assert!(
            addr >= NVM3_START && addr < NVM3_END,
            "outside the NVM3 region"
        );

        let rc = unsafe { MSC_ErasePage(addr) };
        if rc == 0 {
            apagadas += 1;
        } else {
            falhas += 1;
        }
        addr += PAGE;
    }

    // Five short pulses: the region is erased. Two long ones: something failed, and the board needs looking at.
    if falhas == 0 {
        for _ in 0..5 {
            led.on();
            delay.delay_ms(80);
            led.off();
            delay.delay_ms(80);
        }
    } else {
        for _ in 0..2 {
            led.on();
            delay.delay_ms(600);
            led.off();
            delay.delay_ms(200);
        }
    }

    // Halt. The record is left in the mailbox words so the host can read what happened without a serial port.
    let _ = (apagadas, falhas);
    loop {
        cortex_m::asm::wfi();
    }
}
