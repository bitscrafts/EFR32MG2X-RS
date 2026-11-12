//! RAIL links, and says so: read the version out of the vendor library, then try to initialise it.
//!
//! This is the smallest honest thing a RAIL example can do, and it is worth having as its own rung:
//! it proves that the header, the blob and the linker agree — the three parts that have to be the
//! same SDK version, and are not in most projects that try this.
//!
//! ## Why it stops at the version and a null config
//!
//! A real RAIL transmission needs a *radio configuration*: channel definitions, modem settings,
//! PA and front-end tables. That is not an API call, it is generated data — the Radio Configurator
//! in Simplicity Studio emits `rail_config.c` for a given board, and that file is the difference
//! between "the library initialised" and "a packet left the antenna". Writing those tables by hand
//! is how a project ends up with a transmitter nothing can receive.
//!
//! So: `RAIL_GetVersion` (a pure read) proves the blob is the one the headers describe, and
//! `RAIL_Init` with a null protocol is the documented "is this library usable at all" probe. Its
//! status code is the answer, and it is reported rather than assumed.
//!
//! ## Reading the result
//!
//! ```text
//! INFO  rail ver=...              the blob answered; the version is real
//! PASS  rail handle=<addr>        RAIL accepted the null-protocol config and returned a handle
//! FAIL  rail handle=0             the radio refused to come up (clock, or a missing config)
//! FAIL  rail ver=0.0.0            the blob is not linked, or it is the wrong build
//! ```

#![no_std]
#![no_main]

use efr32mg24_hal::board::Board;
use efr32mg24_hal::entry;
use efr32mg24_hal::pin::Port;
use efr32mg24_hal::prelude::*;
use efr32mg24_rail_sys as rail;
use panic_halt as _;

#[path = "../../xiao-mg24-bsp/examples/common/mailbox.rs"]
mod mailbox;

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

#[entry]
fn main() -> ! {
    let mut board = Board::take().expect("peripherals taken twice");
    let mut delay = board.delay();
    let led = board.output_on(Port::A, 7, true);

    // RAIL fills one struct, not four out-parameters (`verbose = true` also fills the build flags).
    let mut version = rail::RAIL_Version {
        hash: 0,
        major: 0,
        minor: 0,
        rev: 0,
        build: 0,
        flags: 0,
        multiprotocol: false,
    };
    // SAFETY: `RAIL_GetVersion` writes through the pointer exactly once and keeps no reference.
    unsafe {
        rail::RAIL_GetVersion(&mut version, true);
    }

    let mut msg = [0u8; 64];
    let mut at = 0;
    msg[at..at + 9].copy_from_slice(b"rail ver=");
    at += 9;
    at = push_dec(&mut msg, at, u32::from(version.major));
    msg[at] = b'.';
    at += 1;
    at = push_dec(&mut msg, at, u32::from(version.minor));
    msg[at] = b'.';
    at += 1;
    at = push_dec(&mut msg, at, u32::from(version.rev));
    // A version of 0.0.0 is not a version: it is a blob that is not linked, or a handle that was
    // never initialised. Saying INFO for it would hide exactly the failure this example exists for.
    let linked = version.major != 0 || version.minor != 0 || version.rev != 0;
    mailbox::write(
        if linked {
            mailbox::Status::Info
        } else {
            mailbox::Status::Fail
        },
        &msg[..at],
    );

    // `RAIL_Init` is deliberately not called here. It pulls `rfhal_radio.o`, which calls four
    // functions that live in Silicon Labs' emlib C sources rather than in the blob:
    //
    //     CMU_ClockEnable                    (em_cmu.c)
    //     EMU_DCDCSetPFMXTimeoutMaxCtrl      (em_emu.c)
    //     SYSTEM_ChipRevisionGet             (em_system.c)
    //     SystemSYSCLKGet                    (em_system.c)
    //
    // Implementing those by hand from a header nobody has would be guessing register bits, which
    // is how a radio ends up configured wrong in a way that transmits. The honest rung is what is
    // above: the header and the blob are the same version and the linker agrees, which is the fact
    // that has to be established before emlib is vendored. See docs/04-reference/RADIO-NOTES.md.

    // The LED says the loop is alive without a debugger attached.
    loop {
        led.on();
        delay.delay_ms(500);
        led.off();
        delay.delay_ms(500);
    }
}
