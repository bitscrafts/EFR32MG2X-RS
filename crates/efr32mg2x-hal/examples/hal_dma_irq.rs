//! Does the LDMA end a transfer with an interrupt, with the core asleep, and does `IEN` gate it?
//!
//! The driver's own doc header listed "interrupt-driven transfers with callbacks" as a future extension, and the
//! `CLAUDE.md` limitations list said "DMA Stage 4 deferred". Neither sentence came with a measurement. This example
//! measures the two halves of it that can be read back off the part.
//!
//! ## What is being measured
//!
//! One 2047-word memory-to-memory transfer, armed twice with the same descriptor, the same unmasked `LDMA_IRQ`
//! vector and the same data. The only bit that differs between the two runs is `IEN.CHDONE` for channel 0:
//!
//! 1. **Gate open.** `Gate::Enabled` sets `IEN.CHDONE`, the handler `service()` acknowledges the flag, and the
//!    caller sleeps in `WFI`. The record has to show: both buffers equal, one handler entry, the `DONE` flag clear
//!    at return (the handler cleared it), and one single `WFI` -- one sleep, no poll, and the deadline clock still
//!    at zero, so the wake was the LDMA vector and not `SysTick`.
//! 2. **Control, gate closed.** `Gate::Closed` clears the same bit and changes nothing else. The data still has to
//!    arrive, the flag still has to latch (`cf=1`), the handler must not run (`cirq=0`) -- and the wait has to end
//!    on the deadline (`cd=0`, `csp>1`). That is what says the wake in run 1 came from the gate: without it the
//!    transfer finishes and the core is never told.
//!
//! ## What would make this instrument lie
//!
//! Counting `WFI` iterations as sleep. `WFI` returns for any pending interrupt, so `sp=1` with `tk=0` only means
//! nothing woke the core before the LDMA vector did; a busy loop would have run thousands of times instead and the
//! counter would show it. The other end is the deadline: `SysTick` is armed here so the control can end at all --
//! without a wake source the control would sleep for ever, and "it never woke" would be indistinguishable from "the
//! example hung".
//!
//! VERIFIED ON HARDWARE (2026-11-27): the record is in `docs/04-reference/DMA-IRQ-2026-11-27.md`.

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};

use defmt_rtt as _;
use efr32mg2x_hal::dma::irq::{self, Gate};
use efr32mg2x_hal::dma::Dma;
use efr32mg2x_hal::entry;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// `SysTick` has woken the core this many times. The deadline is the only thing that can wake a wait whose
/// channel never raises its interrupt, so this is also the number that proves the deadline really ran.
static TICKS: AtomicU32 = AtomicU32::new(0);

/// Ticks the control is allowed to spend before its wait is declared expired. At the default clock one tick is a
/// few milliseconds, so the control is over in tens of milliseconds; the count matters, not the wall time.
const DEADLINE: u32 = 5;

/// Words per transfer. Large enough that the controller is still working when the core reaches its first `WFI`:
/// a transfer that finished before the core slept would report `sp=0` and no longer prove the sleep.
const LEN: usize = 2047;

/// The LDMA vector's handler. The HAL does not define it (see `dma::irq`), the application does, and it is one
/// call: acknowledge every flag and record the channels that finished.
#[no_mangle]
pub extern "C" fn LDMA_IRQHandler() {
    irq::service();
}

/// The deadline clock. `SysTick` is the application's here, not the HAL's: the wait needs a wake source to have a
/// deadline at all, and choosing it is the product's decision.
#[no_mangle]
pub extern "C" fn SysTick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
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

/// Progress, written at every step: "no record" then says something died, not where.
///
///   1 took the peripherals | 2 the controller came up | 3 armed the gate-open run | 4 read it back
///   5 armed the control | 6 read it back
#[used]
#[no_mangle]
static ETAPAS: [AtomicU32; 7] = [const { AtomicU32::new(0) }; 7];

fn etapa(i: usize, v: u32) {
    ETAPAS[i].store(v, Ordering::Relaxed);
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

    // The deadline clock. `SysTick` counting core clocks; with a reload of 78 000 one tick is one millisecond if
    // the core runs at its maximum and a few if it does not. Only the number of ticks is read, never a duration.
    let mut cp = match cortex_m::Peripherals::take() {
        Some(p) => p,
        None => {
            etapa(2, 9);
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    cp.SYST
        .set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    cp.SYST.set_reload(78_000);
    cp.SYST.clear_current();
    cp.SYST.enable_interrupt();
    cp.SYST.enable_counter();
    etapa(2, 1);

    let mut ch0 = dma.channel0();

    let mut src = [0u32; LEN];
    for (i, v) in src.iter_mut().enumerate() {
        *v = 0x5A5A_0000 | i as u32;
    }

    // ---- run 1: the gate open -------------------------------------------------------------------------------
    let mut dst_a = [0u32; LEN];
    TICKS.store(0, Ordering::Relaxed);
    irq::reset(0);
    let mut a = match ch0.start_interrupt(&src, &mut dst_a, Gate::Enabled) {
        Ok(p) => {
            etapa(3, 1);
            p
        }
        Err(_) => {
            etapa(3, 2);
            mailbox::fail(b"dmairq: arm failed");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    let wa = a.wait(|| TICKS.load(Ordering::Relaxed) > DEADLINE);
    let gate_a = irq::gate(0);
    let a_len = a.buffer().len();
    drop(a);
    let a_moved = dst_a.iter().zip(src.iter()).filter(|(d, s)| d == s).count();
    let ticks_a = TICKS.load(Ordering::Relaxed);
    let irq_a = irq::entries();
    etapa(4, 1 + a_moved as u32 + 10 * wa.sleeps + 100 * ticks_a);

    // ---- run 2: the control, gate closed --------------------------------------------------------------------
    let mut dst_b = [0u32; LEN];
    TICKS.store(0, Ordering::Relaxed);
    let mut b = match ch0.start_interrupt(&src, &mut dst_b, Gate::Closed) {
        Ok(p) => {
            etapa(5, 1);
            p
        }
        Err(_) => {
            etapa(5, 2);
            mailbox::fail(b"dmairq: control arm failed");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    let wb = b.wait(|| TICKS.load(Ordering::Relaxed) > DEADLINE);
    let gate_b = irq::gate(0);
    drop(b);
    let b_moved = dst_b.iter().zip(src.iter()).filter(|(d, s)| d == s).count();
    let ticks_b = TICKS.load(Ordering::Relaxed);
    let irq_b = irq::entries() - irq_a;
    etapa(6, 1 + b_moved as u32 + 10 * wb.sleeps + 100 * ticks_b);

    // A pass needs both runs to have moved every word -- the control is a control on the wake, not on the copy --
    // and then: one handler entry and one sleep with the clock still at zero in run 1; no handler entry, the flag
    // latched, and the deadline spent in run 2.
    let ok = a_moved == LEN
        && a_len == LEN
        && wa.done
        && wa.sleeps >= 1
        && !wa.flag
        && gate_a
        && ticks_a == 0
        && irq_a == 1
        && b_moved == LEN
        && !wb.done
        && wb.sleeps >= 1
        && wb.flag
        && !gate_b
        && ticks_b > DEADLINE
        && irq_b == 0;

    let mut line = [0u8; 200];
    let mut n = 0usize;
    push(&mut line, &mut n, b"dmairq: ok=");
    dec(&mut line, &mut n, ok as usize);
    push(&mut line, &mut n, b" n=");
    dec(&mut line, &mut n, LEN);
    push(&mut line, &mut n, b" am=");
    dec(&mut line, &mut n, a_moved);
    push(&mut line, &mut n, b" irq=");
    dec(&mut line, &mut n, irq_a as usize);
    push(&mut line, &mut n, b" sp=");
    dec(&mut line, &mut n, wa.sleeps as usize);
    push(&mut line, &mut n, b" tk=");
    dec(&mut line, &mut n, ticks_a as usize);
    push(&mut line, &mut n, b" g=");
    dec(&mut line, &mut n, gate_a as usize);
    push(&mut line, &mut n, b" ad=");
    dec(&mut line, &mut n, wa.done as usize);
    push(&mut line, &mut n, b" af=");
    dec(&mut line, &mut n, wa.flag as usize);
    push(&mut line, &mut n, b" cm=");
    dec(&mut line, &mut n, b_moved);
    push(&mut line, &mut n, b" cirq=");
    dec(&mut line, &mut n, irq_b as usize);
    push(&mut line, &mut n, b" csp=");
    dec(&mut line, &mut n, wb.sleeps as usize);
    push(&mut line, &mut n, b" ctk=");
    dec(&mut line, &mut n, ticks_b as usize);
    push(&mut line, &mut n, b" cg=");
    dec(&mut line, &mut n, gate_b as usize);
    push(&mut line, &mut n, b" cd=");
    dec(&mut line, &mut n, wb.done as usize);
    push(&mut line, &mut n, b" cf=");
    dec(&mut line, &mut n, wb.flag as usize);
    push(&mut line, &mut n, b"\n");

    if ok {
        mailbox::pass(&line[..n]);
    } else {
        mailbox::fail(&line[..n]);
    }

    loop {
        cortex_m::asm::wfi();
    }
}
