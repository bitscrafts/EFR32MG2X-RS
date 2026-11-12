//! Does the RAIL radio initialise? The question that four missing emlib symbols used to block.
//!
//! Until this session the answer was "it links `RAIL_GetVersion` and nothing else": `RAIL_Init` pulls
//! `rfhal_radio.o`, which references `CMU_ClockEnable`, `SystemSYSCLKGet`, `SYSTEM_ChipRevisionGet`
//! and `EMU_DCDCSetPFMXTimeoutMaxCtrl`. Those came from emlib, which was not in the project and looked
//! too large to vendor — 363 KB of `em_cmu.c` that needs the full device headers.
//!
//! It was not too large. `crates/efr32mg24-emlib-sys` compiles three emlib units against the device
//! and CMSIS headers in a second and a half, and this example is the proof that the radio's software
//! stack now crosses that boundary: if these four symbols were still missing, this binary would not
//! link at all.
//!
//! ## What it does and does not prove
//!
//! Proves: the radio's firmware-level init runs and returns a handle, the emlib calls it makes are
//! correct enough for it to complete, and the result is readable from outside the firmware.
//!
//! Does not prove: that a frame can be transmitted. That needs a channel configuration (frequency,
//! PHY, channel spacing) which normally comes from the vendor's Radio Configurator as generated C.
//! Initialising is a precondition, not a transmitter.

#![no_std]
#![no_main]

use core::ptr;

use efr32mg24_hal::entry;
use efr32mg24_hal::prelude::*;
use efr32mg24_rail_sys as rail;

/// The mailbox the examples and `scripts/read-mailbox.sh` share. Declared as a path module so the
/// file's own `//!` documentation stays inner documentation of the module, where Rust allows it.
#[path = "../../xiao-mg24-bsp/examples/common/mailbox.rs"]
mod mailbox;

use mailbox::Status;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut board = efr32mg24_hal::board::Board::take().expect("board");
    let mut delay = board.delay();

    mailbox::write(Status::Info, b"rail: entered main");

    // RAIL wants a config whose fields the application owns. Nothing here needs a scheduler or an
    // event callback yet: init is what this example is asking about.
    let mut config = rail::RAIL_Config_t {
        eventsCallback: None,
        protocol: ptr::null_mut(),
        scheduler: ptr::null_mut(),
        // The state buffer is RAIL's own scratch space; zeroed is the documented starting point
        // because RAIL fills it during init.
        buffer: unsafe { core::mem::zeroed() },
    };

    let handle = unsafe { rail::RAIL_Init(&mut config, None) };

    if handle.is_null() {
        // A null handle is the honest failure: RAIL refused to come up. The reason is in the
        // assertion handler it would normally call, which is a separate wiring step.
        mailbox::write(Status::Fail, b"rail: init returned null");
        loop {
            delay.delay_ms(1000);
        }
    }

    mailbox::write(Status::Pass, b"rail: init returned a handle");

    // Now ask the library who it is. If init half-worked and the version call hangs or faults, that
    // distinction is visible in the mailbox: the message above is already written.
    let mut version = rail::RAIL_Version_t {
        hash: 0,
        major: 0,
        minor: 0,
        rev: 0,
        build: 0,
        flags: 0,
        multiprotocol: false,
    };
    unsafe { rail::RAIL_GetVersion(&mut version, false) };

    let mut msg = [0u8; 32];
    let mut at = 0;
    msg[at..at + 9].copy_from_slice(b"rail: ver");
    at += 9;
    let text = [
        version.major,
        b'.',
        version.minor,
        b'.',
        version.rev,
        b' ',
        b'h',
        b'=',
    ];
    msg[at..at + text.len()].copy_from_slice(&text);
    at += text.len();
    mailbox::write(Status::Pass, &msg[..at]);

    // Stay alive so the mailbox can be read at leisure; resetting here would erase the RAM evidence.
    loop {
        delay.delay_ms(1000);
    }
}
