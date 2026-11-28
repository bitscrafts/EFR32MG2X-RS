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
const BURST: u32 = 1024;

#[cortex_m_rt::entry]
fn main() -> ! {
    mailbox::invalidate();
    // Every stage writes the record, so the last one present is the line where the program stops. Without this the
    // failure is a `magic 0` and four blind iterations -- which is exactly what happened on 2026-11-28.
    mailbox::info(b"stage 1: board taken");
    let board = match Board::take() {
        Some(board) => board,
        None => halt(),
    };

    // USART0 is the block whose `flush` waits for `STATUS.TXC`, the transmitter's own "I have finished". A transmitter
    // that was never enabled never says it -- that is the defect, and that is the wait whose bound has to be shown.
    mailbox::info(b"stage 2: building USART0");
    let mut usart = match board.usart0(Config::new(BAUD)) {
        Ok(usart) => usart,
        Err(_) => {
            mailbox::fail(b"tx-timeout: USART0 refused");
            halt();
        }
    };

    // ---- control: eight bytes out of an enabled transmitter, then flush, which must return Ok --------------------
    // `flush` right after construction is not a control at all: `TXC` is "the transmission finished", and a block that
    // never transmitted never set it -- measured, `control_flush=0` on the first attempt at this.
    mailbox::info(b"stage 3: control write and flush, transmitter enabled");
    let control_flush = usart
        .write(&[0xA5, 0x5A, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06])
        .and_then(|()| usart.flush())
        .is_ok();

    // ---- the measurement: the same call on a transmitter that was never enabled ----------------------------------
    mailbox::info(b"stage 4: clearing TXEN");
    usart.disable_transmitter_for_test();
    // A write clears `TXC`; without one, `flush` would see the flag the control left behind and return `Ok` on a
    // disabled transmitter -- measured, `flush_timeout=0`. With the transmitter disabled the write is accepted and
    // discarded, `TXC` never sets again, and the flush has to time out.
    mailbox::info(b"stage 5: write and flush on the disabled transmitter");
    let _ = usart.write(&[0xA5, 0x5A, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C]);
    let flush_err = match usart.flush() {
        Ok(()) => 0u32,
        Err(e) => u32::from(e == efr32mg2x_hal::usart::Error::Timeout),
    };

    mailbox::info(b"stage 6: flush returned");
    let mut l = Record::<128>::new();
    l.s(b"tx-timeout: ");
    l.s(b" control_flush=");
    l.u(u32::from(control_flush));
    l.s(b" flush_timeout=");
    l.u(flush_err);

    // The control must return `Ok` on a live transmitter, and the same call on a disabled one must be a `Timeout`. Both
    // halves are needed: without the first, a `Timeout` would prove only that something failed.
    if control_flush && flush_err == 1 {
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
