//! Does `stop()` really disable the DMA channel?
//!
//! `stop()` wrote zeros to `CHEN`, and the reference manual says writing zeros has no effect on that register
//! (§25.7.10) -- the disable register is `CHDIS`. So a `Transfer` could end the borrow of a buffer the DMA was still
//! writing into, from safe code, and nothing said so. The host tests could not catch it: their fake channel records
//! the call, not the register.
//!
//! This measures it where it can be measured. A peripheral-to-memory transfer is armed with
//! `PeripheralRequest::None`, so the channel is enabled and waiting for a request that never comes -- deterministic,
//! no timing race, and exactly the state `stop()` exists to leave. Then:
//!
//!   1. `CHBUSY` is read while the guard is alive: it must say busy, or the instrument is not looking at the right
//!      bit and nothing below means anything.
//!   2. The guard is dropped, which is what a caller does, and it runs `end_transfer()` -> `stop()`.
//!   3. `CHBUSY` is read again: it must say idle. If it still says busy, the channel is armed forever and the buffer
//!      is still the DMA's.
//!
//! Both readings go into the record, so a reader can tell "the check answered" from "the check agreed".

#![no_std]
#![no_main]

use defmt_rtt as _;
use efr32mg2x_hal::dma::{channel_busy, Dma, PeripheralRequest};
use efr32mg2x_hal::entry;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Progresso, escrito a cada passo. Sem isto, "não houve registo" diz que algo morreu e não onde -- e a diferença
/// entre os dois é a única coisa que interessa quando a placa não responde.
///
///   0 nada | 1 tirou periféricos | 2 o controlador ligou | 3 armou | 4 soltou o guard | 5 escreveu o registo
///   6 idle no inicio (0/1) | 7 chbusy armado (0/1) | 8 chbusy depois do stop (0/1)
#[used]
#[no_mangle]
static ETAPAS: [core::sync::atomic::AtomicU32; 9] = [
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

/// `CHBUSY`, asked without the handle: the guard borrows the channel while the transfer is armed.
fn busy_bit() -> bool {
    channel_busy(0)
}

#[entry]
fn main() -> ! {
    mailbox::invalidate();

    let dp = match Dma::take() {
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
    let mut dma = dp;
    etapa(2, 1);
    let mut ch0 = dma.channel0();
    let mut buffer = [0u8; 64];

    let idle_at_start = !ch0.is_busy();
    etapa(6, idle_at_start as u32 + 10);

    // Arm the channel and KEEP the guard: dropping it is the operation under test. `None` means software-triggered
    // only, so nothing will request a byte and the channel stays enabled -- the state `stop()` must be able to undo.
    let guard = ch0.start_peripheral_to_memory(0x2000_0000, &mut buffer, PeripheralRequest::None);
    let armed = guard.is_ok();
    etapa(3, armed as u32 + 2);
    let busy_while_armed = busy_bit();
    etapa(7, busy_while_armed as u32 + 20);

    // What a caller does when the transfer is over, or the buffer must come back.
    drop(guard);
    etapa(4, 1);

    let busy_after_stop = ch0.is_busy();
    etapa(8, busy_after_stop as u32 + 30);

    let mut line = [0u8; 72];
    let mut n = 0usize;
    for chunk in [
        &b"dma-stop: idle0="[..],
        if idle_at_start { &b"1"[..] } else { &b"0"[..] },
        &b" armed="[..],
        if armed { &b"1"[..] } else { &b"0"[..] },
        &b" busy_armed="[..],
        if busy_while_armed {
            &b"1"[..]
        } else {
            &b"0"[..]
        },
        &b" busy_after_stop="[..],
        if busy_after_stop {
            &b"1"[..]
        } else {
            &b"0"[..]
        },
        &b"\n"[..],
    ] {
        for &b in chunk {
            if n < line.len() {
                line[n] = b;
                n += 1;
            }
        }
    }

    // A pass needs all three: the instrument answered (armed and busy), and the stop undid it.
    etapa(5, 1);
    if armed && busy_while_armed && !busy_after_stop {
        mailbox::pass(&line[..n]);
    } else {
        mailbox::fail(&line[..n]);
    }

    loop {
        cortex_m::asm::wfi();
    }
}
