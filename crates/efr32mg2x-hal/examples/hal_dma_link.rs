//! Do the LDMA's linked descriptors actually chain, and does the second leg move only because of the link?
//!
//! The `CLAUDE.md` limitations list said "linked descriptors (Stage 3) deferred" while the driver's own doc comment
//! claimed circular operation was implemented by pointing `LINK` at `CH0_CTRL`. Neither sentence came with a
//! measurement, and the second one was a reading of the reference manual rather than a record off the part.
//!
//! ## What is being measured
//!
//! The controller holds one descriptor per channel in that channel's registers: `CTRL`, `SRC`, `DST`, `LINK`, four
//! contiguous words. A descriptor's `LINK` word names the address of the next descriptor's `CTRL` word; with the
//! `LINK` bit set, the controller loads that descriptor by itself when the current one's count runs out. So a second
//! leg does not need a second worker: channel 1's register set, disabled, is storage for the four words.
//!
//!   1. Two legs armed on channel 0, the head's link word naming channel 1's `CTRL`. If the chain works, both
//!      destinations equal their sources, and channel 0's `SRC` register ends inside the *second* leg's range --
//!      which is what says the controller loaded the second descriptor rather than copying anything twice.
//!   2. The control: the same two descriptors, programmed identically, with the head's `LINK` bit cleared. The first
//!      destination must still arrive and the second must not move at all, and the call must report the failure
//!      (`Timeout`) instead of claiming a chain it did not run.
//!
//! The control is the whole point of the item: "the second leg moved" is worth nothing without the near miss where
//! the descriptor is present, complete, and never fetched.
//!
//! ## What would make this instrument lie
//!
//! Reading `DONE` as "the chain finished". It is one bit per channel (`IF`, RM 25.7.11) and the first leg sets it,
//! so the driver waits on it and then compares the buffers -- a leg that never ran leaves a destination that does not
//! match, which fails closed. The second destination is filled with a sentinel before arming, so "untouched" is a
//! value read back, not the absence of a value.
//!
//! VERIFIED ON HARDWARE (2026-11-27): the record is in this file's header and in `docs/04-reference/DMA-LINKED-2026-11-27.md`.

#![no_std]
#![no_main]

use defmt_rtt as _;
use efr32mg2x_hal::dma::{channel_state, ChainMode, Dma, Segment};
use efr32mg2x_hal::entry;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Progress, written at every step. Without it, "no record" says something died and not where.
///
///   1 took the peripherals | 2 the controller came up | 3 armed the chain | 4 read the chain back
///   5 armed the control | 6 read the control back
#[used]
#[no_mangle]
static ETAPAS: [core::sync::atomic::AtomicU32; 7] = [
    core::sync::atomic::AtomicU32::new(0),
    core::sync::atomic::AtomicU32::new(0),
    core::sync::atomic::AtomicU32::new(0),
    core::sync::atomic::AtomicU32::new(0),
    core::sync::atomic::AtomicU32::new(0),
    core::sync::atomic::AtomicU32::new(0),
    core::sync::atomic::AtomicU32::new(0),
];

fn etapa(i: usize, v: u32) {
    ETAPAS[i].store(v, core::sync::atomic::Ordering::Relaxed);
}

fn push(line: &mut [u8], n: &mut usize, bytes: &[u8]) {
    for &b in bytes {
        if *n < line.len() {
            line[*n] = b;
            *n += 1;
        }
    }
}

fn dec(line: &mut [u8], n: &mut usize, v: usize) {
    let mut buf = [0u8; 20];
    let mut i = buf.len();
    let mut v = v;
    loop {
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    push(line, n, &buf[i..]);
}

fn hex(line: &mut [u8], n: &mut usize, v: u32) {
    push(line, n, b"0x");
    let mut started = false;
    for shift in (0..8).rev() {
        let digit = ((v >> (shift * 4)) & 0xF) as u8;
        if digit != 0 || started || shift == 0 {
            started = true;
            let c = if digit < 10 {
                b'0' + digit
            } else {
                b'a' + digit - 10
            };
            push(line, n, &[c]);
        }
    }
}

/// Sentinel written into every destination before arming: a leg that did not run leaves this value, and reading it
/// back is the difference between "the second destination was not written" and "nobody looked".
const SENTINEL: u32 = 0xDEAD_BEEF;

const LEN: usize = 8;

#[entry]
fn main() -> ! {
    mailbox::invalidate();

    let mut dma = match Dma::take() {
        Some(d) => {
            etapa(1, 1);
            d
        }
        None => {
            etapa(1, 9);
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    etapa(2, 1);
    let mut ch0 = dma.channel0();

    // ---- the chain ---------------------------------------------------------------------------------------------
    let src_a = [
        0xA0A0_0001u32,
        0xA0A0_0002,
        0xA0A0_0003,
        0xA0A0_0004,
        0xA0A0_0005,
        0xA0A0_0006,
        0xA0A0_0007,
        0xA0A0_0008,
    ];
    let src_b = [
        0xB0B0_0001u32,
        0xB0B0_0002,
        0xB0B0_0003,
        0xB0B0_0004,
        0xB0B0_0005,
        0xB0B0_0006,
        0xB0B0_0007,
        0xB0B0_0008,
    ];
    let mut dst_a = [0u32; LEN];
    let mut dst_b = [SENTINEL; LEN];

    let mut segs = [
        Segment::new(&src_a, &mut dst_a),
        Segment::new(&src_b, &mut dst_b),
    ];
    let chained = ch0.transfer_chain(&mut segs, ChainMode::Linked);
    etapa(3, if chained.is_ok() { 1 } else { 2 });

    let chain_ok = chained.is_ok();
    let a_moved = dst_a
        .iter()
        .zip(src_a.iter())
        .filter(|(d, s)| d == s)
        .count();
    let b_moved = dst_b
        .iter()
        .zip(src_b.iter())
        .filter(|(d, s)| d == s)
        .count();
    let after_chain = channel_state(0);
    etapa(4, 1 + a_moved as u32 + 100 * b_moved as u32);

    // ---- the control: the same descriptors, LINK cleared -------------------------------------------------------
    let src_c = [0xC0C0_0001u32; LEN];
    let src_d = [0xD0D0_0001u32; LEN];
    let mut dst_c = [0u32; LEN];
    let mut dst_d = [SENTINEL; LEN];

    let mut segs2 = [
        Segment::new(&src_c, &mut dst_c),
        Segment::new(&src_d, &mut dst_d),
    ];
    let first_only = ch0.transfer_chain(&mut segs2, ChainMode::FirstOnly);
    etapa(5, 1);

    let c_moved = dst_c
        .iter()
        .zip(src_c.iter())
        .filter(|(d, s)| d == s)
        .count();
    let d_moved = dst_d
        .iter()
        .zip(src_d.iter())
        .filter(|(d, s)| d == s)
        .count();
    let control_failed = matches!(first_only, Err(efr32mg2x_hal::dma::DmaError::Timeout));
    let after_control = channel_state(0);
    etapa(6, 1 + c_moved as u32 + 100 * d_moved as u32);

    // The mechanism, read off the channel: after the chain, CH0's source register has to be somewhere inside the
    // second leg (the controller loaded that descriptor and ran it); after the control it has to still be inside the
    // first. The registers advance as the transfer runs, so the test is a range, not an equality.
    let b_range = (
        src_b.as_ptr() as u32,
        src_b.as_ptr() as u32 + 4 * LEN as u32,
    );
    let c_range = (
        src_c.as_ptr() as u32,
        src_c.as_ptr() as u32 + 4 * LEN as u32,
    );
    let in_b = |v: u32| v >= b_range.0 && v <= b_range.1;
    let in_c = |v: u32| v >= c_range.0 && v <= c_range.1;
    let chain_src_in_b = after_chain.map(|s| in_b(s.src)).unwrap_or(false);
    let control_src_in_c = after_control.map(|s| in_c(s.src)).unwrap_or(false);
    let chain_link = after_chain.map(|s| s.link).unwrap_or(0xFFFF_FFFF);
    let control_link = after_control.map(|s| s.link).unwrap_or(0xFFFF_FFFF);

    let mut line = [0u8; 200];
    let mut n = 0usize;
    push(&mut line, &mut n, b"dmalink: ok=");
    dec(&mut line, &mut n, chain_ok as usize);
    push(&mut line, &mut n, b" d1=");
    dec(&mut line, &mut n, a_moved);
    push(&mut line, &mut n, b" d2=");
    dec(&mut line, &mut n, b_moved);
    push(&mut line, &mut n, b" s2=");
    dec(&mut line, &mut n, chain_src_in_b as usize);
    push(&mut line, &mut n, b" lk=");
    hex(&mut line, &mut n, chain_link);
    push(&mut line, &mut n, b" cto=");
    dec(&mut line, &mut n, control_failed as usize);
    push(&mut line, &mut n, b" c1=");
    dec(&mut line, &mut n, c_moved);
    push(&mut line, &mut n, b" c2=");
    dec(&mut line, &mut n, d_moved);
    push(&mut line, &mut n, b" c1r=");
    dec(&mut line, &mut n, control_src_in_c as usize);
    push(&mut line, &mut n, b" clk=");
    hex(&mut line, &mut n, control_link);
    push(&mut line, &mut n, b" keep=");
    hex(&mut line, &mut n, dst_d[0]);
    // The descriptor words themselves: what channel 0 ended up holding. A second leg that was fetched but never run
    // leaves the source register at the start of leg 2 and the count untouched; one that ran leaves it at the end.
    let ch0_end = channel_state(0).unwrap_or_default();
    push(&mut line, &mut n, b" ctl=");
    hex(&mut line, &mut n, ch0_end.ctrl);
    push(&mut line, &mut n, b" ssrc=");
    hex(&mut line, &mut n, ch0_end.src);
    push(&mut line, &mut n, b" sdst=");
    hex(&mut line, &mut n, ch0_end.dst);
    push(&mut line, &mut n, b"\n");

    // A pass needs: the chain to report success and to have moved both legs; the controller's own source register to
    // have ended in the second leg; the last descriptor to link nowhere; and the control to refuse the chain while
    // still moving the first leg and leaving the second at the sentinel.
    if chain_ok
        && a_moved == LEN
        && b_moved == LEN
        && chain_src_in_b
        && chain_link == 0
        && control_failed
        && c_moved == LEN
        && d_moved == 0
        && control_src_in_c
        && dst_d[0] == SENTINEL
    {
        mailbox::pass(&line[..n]);
    } else {
        mailbox::fail(&line[..n]);
    }

    loop {
        cortex_m::asm::wfi();
    }
}
