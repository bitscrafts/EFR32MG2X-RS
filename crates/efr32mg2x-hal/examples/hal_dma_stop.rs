//! Does `stop()` really disable the DMA channel?
//!
//! `stop()` wrote zeros to `CHEN`, and the reference manual says writing zeros has no effect on that register
//! (§25.7.10) -- the disable register is `CHDIS`. So a `Transfer` could end the borrow of a buffer the DMA was still
//! writing into, from safe code, and nothing said so. The host tests could not catch it: their fake channel records
//! the call, not the register.
//!
//! ## Why the first version of this instrument could not answer
//!
//! It read `CHBUSY` while the channel was armed, and got `busy_armed=0`. That is not the defect showing: `CHBUSY`
//! (RM 25.7.13) reads 1 only while the channel is doing a transfer, and a 64-byte block finishes between two
//! statements, so the bit reads idle whether or not the channel was disabled. The register that answers "is this
//! channel still armed" is `CHSTATUS` (RM 25.7.12): one bit per channel, 1 from the moment `CHEN` is set until
//! `CHDIS` takes effect, and readable at any time.
//!
//! ## What this measures
//!
//!   1. `CHSTATUS` before arming: must be clear, or the bit is not this channel's.
//!   2. A peripheral-to-memory channel armed with `PeripheralRequest::None` -- deterministic, nothing requests it --
//!      then given one software request, which puts the circular descriptor to work.
//!   3. `CHSTATUS` while the guard is alive: must be set, or the instrument is not looking at the armed state.
//!   4. The guard is dropped, which is what a caller does, and it runs `end_transfer()` -> `stop()`.
//!   5. `CHSTATUS` again: must be clear. If it is still set, the channel is armed forever and the buffer is still
//!      the DMA's.
//!   6. A plain blocking `transfer()` must leave the channel **disabled** as well: it returned with `CHSTATUS` set
//!      before this session, i.e. armed after the borrow had ended, reachable by a later request. A second `stop()`
//!      must be idempotent.
//!
//! Step 3 is the control for step 5: the same bit, read through the same function, has to be able to read 1 in this
//! boot, or the zero afterwards would prove nothing.
//!
//! Every reading goes into the record, so a reader can tell "the check answered" from "the check agreed".

#![no_std]
#![no_main]

use defmt_rtt as _;
use efr32mg2x_hal::dma::{channel_busy, channel_enabled, trigger, Dma, PeripheralRequest};
use efr32mg2x_hal::entry;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Progress, written at every step. Without it, "no record" says something died and not where.
///
///   1 took the peripherals | 2 the controller came up | 3 read the state before arming
///   4 armed | 5 sampled CHBUSY | 6 sampled CHSTATUS | 7 dropped the guard | 8 sampled after the stop
///   9 ran the control transfer
#[used]
#[no_mangle]
static ETAPAS: [core::sync::atomic::AtomicU32; 10] = [
    core::sync::atomic::AtomicU32::new(0),
    core::sync::atomic::AtomicU32::new(0),
    core::sync::atomic::AtomicU32::new(0),
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

fn bit(line: &mut [u8], n: &mut usize, label: &[u8], v: bool) {
    push(line, n, label);
    push(line, n, if v { b"1" } else { b"0" });
}

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
    let mut buffer = [0u8; 64];

    let stat_before = channel_enabled(0);
    etapa(3, stat_before as u32 + 10);

    // Arm and KEEP the guard: dropping it is the operation under test. `None` means software-triggered only, so
    // nothing requests a byte until the `trigger` below.
    let guard = ch0.start_peripheral_to_memory(0x2000_0000, &mut buffer, PeripheralRequest::None);
    let armed = guard.is_ok();
    etapa(4, armed as u32 + 20);

    let mut busy_armed = false;
    let mut stat_armed = false;
    if armed {
        trigger(0);
        // Give the controller time to pick the channel up; without this the read is of the state before the request.
        for _ in 0..1000 {
            cortex_m::asm::nop();
        }
        busy_armed = channel_busy(0);
        stat_armed = channel_enabled(0);
    }
    etapa(5, busy_armed as u32 + 30);
    etapa(6, stat_armed as u32 + 40);

    // What a caller does when the buffer must come back.
    drop(guard);
    etapa(7, 1);

    let stat_stopped = channel_enabled(0);
    etapa(8, stat_stopped as u32 + 50);

    // A blocking memory-to-memory transfer: it must copy, and it must not leave the channel armed -- an armed
    // channel still owns its spent descriptor and a later request can run it over a buffer the caller now owns.
    let src = [1u32, 2, 3, 4, 5, 6, 7, 8];
    let mut dst = [0u32; 8];
    let control_ok = ch0.transfer(&src, &mut dst).is_ok() && dst == src;
    let stat_after_transfer = channel_enabled(0);
    // And a second stop is idempotent.
    ch0.stop();
    let stat_stopped_again = channel_enabled(0);
    etapa(9, control_ok as u32 + 60);

    let mut line = [0u8; 96];
    let mut n = 0usize;
    push(&mut line, &mut n, b"dma-stop: st0=");
    push(&mut line, &mut n, if stat_before { b"1" } else { b"0" });
    bit(&mut line, &mut n, b" armed=", armed);
    bit(&mut line, &mut n, b" stA=", stat_armed);
    bit(&mut line, &mut n, b" busyA=", busy_armed);
    bit(&mut line, &mut n, b" stS=", stat_stopped);
    bit(&mut line, &mut n, b" ctl=", control_ok);
    bit(&mut line, &mut n, b" stRe=", stat_after_transfer);
    bit(&mut line, &mut n, b" stS2=", stat_stopped_again);
    push(&mut line, &mut n, b"\n");

    // A pass needs the instrument to answer (armed and enabled while it was, cleared by the stop), the blocking
    // transfer to leave the channel disabled, and a second stop to be idempotent. `busyA` is recorded and not
    // required: it is the wrong instrument, and it read 0 while the channel was armed.
    if !stat_before
        && armed
        && stat_armed
        && !stat_stopped
        && control_ok
        && !stat_after_transfer
        && !stat_stopped_again
    {
        mailbox::pass(&line[..n]);
    } else {
        mailbox::fail(&line[..n]);
    }

    loop {
        cortex_m::asm::wfi();
    }
}
