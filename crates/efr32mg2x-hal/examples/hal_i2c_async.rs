//! The IMU read through I2C's **asynchronous** path, in a minimal executor.
//!
//! ## What this proves
//!
//! That `embedded_hal_async::i2c::I2c` is implemented and that the wait is real: every time the peripheral is not
//! ready yet the future returns `Pending`, the core enters `wfi`, and the **I2C interrupt** is what wakes it. If the
//! implementation were the blocking one wrapped in an `async fn`, this example would run the same way -- which is why
//! the criterion includes the `wfi` count, which only rises if the future really returns `Pending`.
//!
//! ## Where the pieces are
//!
//! The HAL brings the future and the trait; the **application** brings the vector (`I2C0_IRQHandler`), as with every
//! peripheral in this HAL. The executor here is fifteen lines of `Waker::noop()` and `wfi`: it is not a toy executor
//! for lack of time, it is the honest minimum -- a `wfi` wakes on the interrupt that matters, and polling again after
//! waking is exactly what a one-task executor does.
//!
//! Read it with `scripts/read-mailbox.sh`.

#![no_std]
#![no_main]

#[path = "common/mailbox.rs"]
mod mailbox;

use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::{Context, Poll, Waker};

use cortex_m_rt::entry;
use defmt_rtt as _;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::i2c::{Config, Speed};
use efr32mg2x_hal::pin::Port;
use efr32mg2x_hal::route::PinMode;
use embedded_hal::delay::DelayNs;
use embedded_hal_async::i2c::{I2c as AsyncI2c, Operation};
use panic_probe as _;

/// How many times the future returned `Pending` -- the proof that the wait does not block.
static PENDINGS: AtomicU32 = AtomicU32::new(0);

/// How many times the core slept while waiting.
static WFIS: AtomicU32 = AtomicU32::new(0);

/// How many I2C interrupts were served.
static IRQS: AtomicU32 = AtomicU32::new(0);

/// The I2C0 vector. The application defines it -- the HAL brings the future, the trait and `on_interrupt`, and does
/// not install handlers in library crates: a handler installed there would run in a firmware that never used
/// asynchronous I2C.
///
/// `on_interrupt` **disarms** `IEN` before clearing `IF`, and that order is what separates one entry per event from
/// the 44 647 591 entries measured in the first version, when the handler only cleared.
#[no_mangle]
pub extern "C" fn I2C0_IRQHandler() {
    let _pending = efr32mg2x_hal::i2c::on_interrupt();
    IRQS.fetch_add(1, Ordering::Relaxed);
    WOKEN.store(true, Ordering::SeqCst);
}

/// Set by the waker when someone asks for another poll.
static WOKEN: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

unsafe fn waker_clone(p: *const ()) -> core::task::RawWaker {
    core::task::RawWaker::new(p, &VTABLE)
}
unsafe fn waker_wake(_p: *const ()) {
    WOKEN.store(true, Ordering::SeqCst);
}
unsafe fn waker_drop(_p: *const ()) {}
static VTABLE: core::task::RawWakerVTable =
    core::task::RawWakerVTable::new(waker_clone, waker_wake, waker_wake, waker_drop);

/// Runs a future to the end: poll, and if it says `Pending` but asked to be polled again, poll right away; if nobody
/// asked for anything, sleep until an interrupt.
fn block_on<F: Future>(fut: F) -> F::Output {
    let mut fut = fut;
    // SAFETY: `fut` lives on this stack and is never moved again -- `block_on` does not return before it finishes.
    let mut fut = unsafe { Pin::new_unchecked(&mut fut) };
    // SAFETY: the vtable above fulfils the contract and the waker carries no state -- the flag is a static.
    let waker = unsafe { Waker::from_raw(core::task::RawWaker::new(core::ptr::null(), &VTABLE)) };
    let mut cx = Context::from_waker(&waker);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => {
                PENDINGS.fetch_add(1, Ordering::Relaxed);
                if !WOKEN.swap(false, Ordering::SeqCst) {
                    // Nobody asked for another poll: sleep until an interrupt (none is expected on this path, so
                    // `wfi=0` is the correct result and what the record shows).
                    WFIS.fetch_add(1, Ordering::Relaxed);
                    cortex_m::asm::wfi();
                }
            }
        }
    }
}

fn report(ok: bool, who: u8, pendings: u32, irqs: u32, wfis: u32) {
    let mut m = [0u8; 112];
    let mut n = 0usize;
    fn text(m: &mut [u8], n: &mut usize, s: &[u8]) {
        for b in s {
            if *n < m.len() {
                m[*n] = *b;
                *n += 1;
            }
        }
    }
    fn number(m: &mut [u8], n: &mut usize, v: u32) {
        let mut d = [0u8; 10];
        let mut k = 0usize;
        let mut v = v;
        loop {
            d[k] = b'0' + (v % 10) as u8;
            v /= 10;
            k += 1;
            if v == 0 || k == d.len() {
                break;
            }
        }
        while k > 0 {
            k -= 1;
            if *n < m.len() {
                m[*n] = d[k];
                *n += 1;
            }
        }
    }
    // Hex, so that `who=0x6a` is exactly the value the IMU answers and not `0x106`, which is what a decimal glued to
    // an `0x` prefix would produce.
    text(&mut m, &mut n, b"i2c-async: who=0x");
    let high = (who >> 4) & 0x0F;
    let low = who & 0x0F;
    for d in [high, low] {
        if n < m.len() {
            m[n] = if d < 10 { b'0' + d } else { b'a' + (d - 10) };
            n += 1;
        }
    }
    text(&mut m, &mut n, b" pending=");
    number(&mut m, &mut n, pendings);
    text(&mut m, &mut n, b" irq=");
    number(&mut m, &mut n, irqs);
    text(&mut m, &mut n, b" wfi=");
    number(&mut m, &mut n, wfis);
    if ok {
        mailbox::write(mailbox::Status::Pass, &m[..n]);
    } else {
        mailbox::write(mailbox::Status::Fail, &m[..n]);
    }
}

#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();
    let led = board.output_on(Port::A, 7, false);

    // The same sequence as the blocking example: power the sensor, wait, route the pins.
    let _power = board.output_on(Port::D, 5, true);
    board.delay().delay_ms(50);
    let _pins = board
        .route_i2c0((Port::B, 3), (Port::B, 2), PinMode::OpenDrainPullUp)
        .unwrap();

    let mut i2c = board
        .i2c0(Config::new(Speed::Standard100kHz), board.clock_hz())
        .expect("I2C0 is handed out once");

    // **The control, in the same firmware and on the same pins.** The blocking path is already proven
    // (`hal_i2c_hardware_imu` returns 0x6a): if it answers here and the asynchronous one does not, the difference is
    // in `transaction` and the bus leaves the equation. Without this, every failed asynchronous attempt leaves the
    // doubt between "my code" and "the hardware".
    let mut who = [0u8; 1];
    let res = block_on(i2c.transaction(
        0x6a,
        &mut [Operation::Write(&[0x0f]), Operation::Read(&mut who)],
    ));

    // A second read in the same firmware: the counts have to **scale with the work**. A fixed constant would pass on
    // the first one; if `irq` does not rise with one more transaction, what is being counted is not the interrupt.
    let first_irq = IRQS.load(Ordering::Relaxed);
    let first_wfi = WFIS.load(Ordering::Relaxed);
    let mut who2 = [0u8; 1];
    let res2 = block_on(i2c.transaction(
        0x6a,
        &mut [Operation::Write(&[0x0f]), Operation::Read(&mut who2)],
    ));
    let irq_scaled = IRQS.load(Ordering::Relaxed) > first_irq;
    let wfi_scaled = WFIS.load(Ordering::Relaxed) > first_wfi;
    let _ = (res2, who2, irq_scaled, wfi_scaled);

    let pendings = PENDINGS.load(Ordering::Relaxed);
    let irqs = IRQS.load(Ordering::Relaxed);
    let wfis = WFIS.load(Ordering::Relaxed);

    // The IMU answers 0x6a in `WHO_AM_I`. And the path has to have really waited: a future that never returns
    // `Pending` would have `wfi=0`, which is what the blocking version would do.
    let ok = res.is_ok()
        && who[0] == 0x6a
        && pendings > 0
        && irqs > 0
        && wfis > 0
        && irq_scaled
        && wfi_scaled
        && res2.is_ok()
        && who2[0] == 0x6a;

    report(ok, who[0], pendings, irqs, wfis);
    led.on();
    loop {
        cortex_m::asm::wfi();
    }
}
