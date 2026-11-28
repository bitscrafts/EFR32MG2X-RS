//! A transmitter that never drains returns `Err(Timeout)` instead of hanging.
//!
//! ## Why this exists
//!
//! `write_byte` used to spin on `STATUS.TXFL` with no limit: a block whose clock nobody enabled, whose pins nobody
//! routed, or whose transmitter never got room would stop the program there for ever -- and `write` returned `()`, so
//! the caller had no way to tell. (Outside review, f20.)
//!
//! ## How the transmitter is made to stall
//!
//! The first attempt used the loopback (`CFG0.LOOPBK`) and no draining, on the theory that two full FIFOs would stall
//! the transmitter. Measured against the board: `written=240 refused_at=240` -- this part's loopback does not
//! back-pressure, it overruns. The path is therefore reached with `Board::eusart0_unclocked_for_test`, behind the
//! `test-hooks` feature: a block whose `CLKEN1` bit is clear reads zero in every status register, so `TXFL` never sets
//! and the write must be refused.
//!
//! ## Bench state, 2026-11-28: no record yet, and the reason is one level below the fix
//!
//! With `Board::eusart0_unclocked_for_test` the image boots, invalidates the mailbox and then writes nothing at all:
//! twelve reads, `magic 0`, no record. The constructor does not return -- `Eusart0::new_at` waits on the block it is
//! configuring, and a block with no clock never reports. So the hook reaches the driver but not the write path, and the
//! timeout is still unentered.
//!
//! What that means for the next attempt: the instrument has to hand out a block that is **configured but ungated** --
//! build it with the clock on, then clear `CLKEN1.EUSART0` before the first write -- instead of building it ungated.
//! That is a second hook, and it is written down in the handoff rather than half-built here.
//!
//! ## The control, and the near-miss
//!
//! The same 240 bytes on `USART0`, whose clock is on, must all be accepted. Without that run a `Timeout` would prove
//! only that something failed, not that the bound is what failed.

#![no_std]
#![no_main]

use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::record::Record;
use efr32mg2x_hal::usart::Config;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

const BAUD: u32 = 115_200;
/// More than the transmitter and receiver FIFOs can hold together, so a stalled transmitter must be reported.
const BURST: u32 = 32;

#[cortex_m_rt::entry]
fn main() -> ! {
    mailbox::invalidate();
    let board = match Board::take() {
        Some(board) => board,
        None => halt(),
    };

    // ---- the measurement: a block whose clock is off must refuse the first byte --------------------------------
    let mut stalled = match board.eusart0_tx_disabled_for_test(Config::new(BAUD)) {
        Ok(uart) => uart,
        Err(_) => {
            mailbox::fail(b"tx-timeout: the test hook refused");
            halt();
        }
    };
    let mut refused_at = BURST;
    let mut written = 0u32;
    let mut timeout_err = false;
    for index in 0..BURST {
        match stalled.write_byte((index & 0xFF) as u8) {
            Ok(()) => written += 1,
            Err(e) => {
                refused_at = index;
                timeout_err = e == efr32mg2x_hal::usart::Error::Timeout;
                break;
            }
        }
    }

    // ---- the control: the same burst on a block whose clock is on, which must all be accepted -------------------
    // `EUSART0` is already claimed by the hook above, so the control takes the USART0 -- the other block with a bounded
    // write, and the one a product actually uses.
    let mut control_ok = 0u32;
    if let Ok(mut usart) = board.usart0(Config::new(BAUD)) {
        for index in 0..BURST {
            if usart.write_byte((index & 0xFF) as u8).is_ok() {
                control_ok += 1;
            }
        }
    }

    let mut l = Record::<128>::new();
    l.s(b"tx-timeout: control=");
    l.u(control_ok);
    l.s(b" written=");
    l.u(written);
    l.s(b" refused_at=");
    l.u(refused_at);
    l.s(b" of=");
    l.u(BURST);
    l.s(b" timeout=");
    l.u(u32::from(timeout_err));

    // The control must accept every byte; the stalled run must be refused before the burst ends, and not at zero --
    // a refusal on the first byte would mean the loopback never worked and nothing was measured.
    if control_ok == BURST && refused_at > 0 && refused_at <= BURST && timeout_err {
        mailbox::pass(l.bytes());
    } else {
        mailbox::fail(l.bytes());
    }

    halt()
}

fn halt() -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}
