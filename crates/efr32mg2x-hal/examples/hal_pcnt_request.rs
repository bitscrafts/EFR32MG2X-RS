//! What `EN` and `SYNCBUSY` actually say around `CORERST`, counted instead of assumed.
//!
//! `src/pcnt.rs` bounds its wait for `EN.DISABLING` and then gives up quietly. Making that give-up a `Result` was tried
//! on 2026-11-28 and the board answered at once: `init` -- which is `CORERST` followed by that wait -- failed on the
//! first thing the example did, with a bound of 200 000 iterations of `delay(64)`, about 0.7 s at 19 MHz, against a
//! handshake documented at 30 us.
//!
//! Two explanations fit, and they need different fixes:
//!
//! * `DISABLING` only means something while `EN` has been high, so the wait belongs after `disable()`, and `init`
//!   needs no wait at all;
//! * or the bit is genuinely stuck, and the wait has to poll `SYNCBUSY` instead -- which the module already trusts for
//!   the command register.
//!
//! This records the raw words at the three moments that tell them apart, plus the iterations the wait really spent, so
//! "never" and "slow" cannot be confused. Expected record shape:
//!
//!     pcnt-req: en0=… sync0=… en1=… sync1=… en2=… sync2=… spins=… clk=…

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::record::Record;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

// A fault must say so: this instrument now writes to registers the driver owns while the block is enabled, and a
// silent stop there is the same ambiguity that cost four runs on the transmit instrument.
#[path = "common/fault_record.rs"]
mod fault_record;

#[entry]
fn main() -> ! {
    mailbox::invalidate();
    mailbox::info(b"pcnt-req: board");

    let board = match Board::take() {
        Some(board) => board,
        None => fail(b"pcnt-req: the board was already taken"),
    };
    let mut pcnt = match board.pcnt() {
        Ok(pcnt) => pcnt,
        Err(_) => fail(b"pcnt-req: the counter is not available"),
    };

    // The driver's own `init`, which now writes the SET/CLR aliases this PAC cannot express. Then the same reading.
    pcnt.init();

    // SAFETY: `init` has returned, so the driver is idle; the borrow lasts for this block and no driver method runs
    // inside it.
    let (en0, sync0) = unsafe {
        let block = pcnt.raw_block();
        (block.en().read().bits(), block.syncbusy().read().bits())
    };

    // One command through the driver, then the same reading: this is the pair that used to sit pending for ever.
    let _ = pcnt.start();
    cortex_m::asm::delay(4_000);
    // SAFETY: as above, `start` has returned.
    let (en1, sync1) = unsafe {
        let block = pcnt.raw_block();
        (block.en().read().bits(), block.syncbusy().read().bits())
    };
    let _ = pcnt.stop();

    // ---- second experiment: the same command, with the branch sourced from ULFRCO ---------------------------------
    // ULFRCO is the one oscillator on this part that has no enable and always runs. If `SYNCBUSY.CMD` clears under it,
    // what is missing is LFRCO oscillating; if it sticks here too, the block's command path is what is wrong and the
    // clock was never the subject. Two outcomes, two different fixes, one boot.
    let sync3 = unsafe {
        let cmu = pcnt.raw_cmu();
        cmu.clken0().modify(|_r, w| w.ulfrco().set_bit());
        cmu.em23grpaclkctrl().modify(|_r, w| w.clksel().ulfrco());
        let block = pcnt.raw_block();
        block.cmd().write(|w| w.cntrst().set_bit());
        let mut spins = 0u32;
        while block.syncbusy().read().cmd().bit_is_set() {
            spins += 1;
            if spins > 200_000 {
                break;
            }
            cortex_m::asm::delay(64);
        }
        block.syncbusy().read().bits()
    };

    // ---- third experiment: answered the hard way, twice ------------------------------------------------------
    // The hypothesis was that `CMD` only completes while the counter is enabled, since `EN` read 0 at every moment of
    // the first experiment. Two boots wrote `CMD` with `EN` set and both took a precise bus error at `PCNT0 + 0x14`:
    //
    //     fault: cfsr=00008200 hfsr=40000000 bfar=59030014 mmfar=59030014 pc=08006576
    //
    // What the two boots establish is narrower than that, and it is worth writing narrowly: a `CMD` write inside the
    // enable/disable sequence takes a precise bus error at `PCNT0 + 0x14`. They do not establish *which* write did it --
    // the sequence writes with `EN` low, raises `EN`, times out, lowers `EN`, delays 4 ms and writes again, and either
    // of two writes could be the one that faulted. The experiment is therefore recorded and not interpreted, and is not
    // repeated here: a fault destroys the record that would explain it. The module already says configuration is
    // refused with `EN` set; whether `CMD` shares that rule is still open.

    // The clock the block runs from, read from CMU: a block with no clock reads like a handshake that never finishes,
    // and those two must not be confused.
    // SAFETY: CMU is at a fixed address for the life of the program and these are reads of words this crate does not
    // own through `Pcnt`; nothing is written, so the driver's view of CMU cannot drift.
    let (clk, clken0, clken1, em23) = unsafe {
        let cmu = pcnt.raw_cmu();
        (
            cmu.pcnt0clkctrl().read().bits(),
            cmu.clken0().read().bits(),
            cmu.clken1().read().bits(),
            cmu.em23grpaclkctrl().read().bits(),
        )
    };

    mailbox::info(b"pcnt-req: record");
    let mut l = Record::<192>::new();
    l.s(b"pcnt-req: en0=");
    l.h32(en0);
    l.s(b" sync0=");
    l.h32(sync0);
    l.s(b" en1=");
    l.h32(en1);
    l.s(b" sync1=");
    l.h32(sync1);
    l.s(b" clk=");
    l.h32(clk);
    l.s(b" clken0=");
    l.h32(clken0);
    l.s(b" clken1=");
    l.h32(clken1);
    l.s(b" em23=");
    l.h32(em23);
    l.s(b" sync_ulfrco=");
    l.h32(sync3);

    // `FAIL` is right whatever the numbers say: this is an instrument that reports, not a claim that the block is well.
    mailbox::fail(l.bytes());
    halt()
}

fn fail(message: &[u8]) -> ! {
    mailbox::fail(message);
    halt()
}

fn halt() -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}
