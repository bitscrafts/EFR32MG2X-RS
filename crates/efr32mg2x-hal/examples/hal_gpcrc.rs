//! Is the CRC accelerator right? Check it against a number computed elsewhere.
//!
//! ## Why this example exists
//!
//! A CRC that is wrong is worse than no CRC: it is a check that rejects good images and accepts bad ones, and nothing
//! reports it. The accelerator on this part computes the MSB-first convention (poly `0x04C11DB7`), which is **not** the
//! CRC-32 that `zlib`, PNG or Ethernet mean — so "the CRC matches" is not a claim that can be made without saying which
//! CRC.
//!
//! The reference here is an independent implementation in `libraries/gpcrc-core`, host-tested against the published
//! vectors of both conventions. The value this example compares against is the published CRC-32/MPEG-2 check value for
//! `b"123456789"` with seed `0xFFFFFFFF`: **`0x0376E6E7`**. If the accelerator disagrees with a published number, one of
//! the two is wrong and the record says which number came out.
//!
//! ## The three controls
//!
//! A single matching number would also be produced by a driver that returned a constant. So the record carries:
//!
//! - `diff=1` — a second, different buffer gives a different CRC;
//! - `seed=1` — the same buffer with a different seed gives a different CRC, which is what proves `CMD.INIT` actually
//!   loads the seed (`INIT` alone does not, and the resulting CRC looks entirely plausible);
//! - and the verdict requires all of them, plus the match against the published value.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use efr32mg2x_hal as _;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::gpcrc::CRCPoly;
use efr32mg2x_hal::record::Record;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// The seed both common CRC-32 conventions use.
const SEED: u32 = 0xFFFF_FFFF;

/// The published check value of the reflected CRC-32 (`zlib`, PNG, Ethernet) for `b"123456789"`.
///
/// Measured 2026-11-25: the accelerator returns `0x340BC6D9` raw, and `!0x340BC6D9` is exactly this number. So the engine
/// computes the **reflected** CRC-32 and stops one step short of the convention every tool publishes -- the final
/// exclusive-or is the caller's. The doc comment in `src/gpcrc.rs` said MSB-first before this was measured, and the part
/// disagreed with the document.
const WANT_ZLIB: u32 = 0xCBF4_3926;

/// The same value before the final exclusive-or, which is what comes out of `DATA`.
const WANT_RAW: u32 = 0x340B_C6D9;

#[entry]
fn main() -> ! {
    mailbox::invalidate();

    let board = match Board::take() {
        Some(board) => board,
        None => loop {
            core::hint::spin_loop();
        },
    };

    let mut crc = match board.gpcrc(CRCPoly::Crc32) {
        Ok(crc) => crc,
        Err(_) => {
            // Taken already, or the board refused it. Either way there is no engine to measure.
            mailbox::fail(b"gpcrc: accelerator not handed out");
            halt();
        }
    };

    let check = crc.crc(SEED, b"123456789");
    // A different message, same seed.
    let other = crc.crc(SEED, b"123456780");
    // Same message, different seed: this is the control for `CMD.INIT`.
    let other_seed = crc.crc(0x0000_0000, b"123456789");
    let seed_ok = crc.seed_changes_the_result();

    let mut l = Record::<140>::new();
    l.s(b"gpcrc: poly=");
    l.s(crc.poly().as_str().as_bytes());
    l.s(b" raw=");
    l.h32(check);
    l.s(b" zlib=");
    l.h32(!check);
    l.s(b" want=");
    l.h32(WANT_ZLIB);
    l.s(b" diff=");
    l.bit(other != check);
    l.s(b" seed=");
    l.bit(seed_ok && other_seed != check);

    // Three things at once: the engine matches the published number after the caller's final xor, it is not a constant
    // (`diff`), and the seed reaches it (`seed`). Any one alone can pass on a driver that is wrong in the other two ways.
    let passed = !check == WANT_ZLIB
        && check == WANT_RAW
        && other != check
        && seed_ok
        && other_seed != check;
    if passed {
        mailbox::pass(l.bytes());
    } else {
        mailbox::fail(l.bytes());
    }

    core::mem::forget(crc);
    halt();
}

fn halt() -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}
