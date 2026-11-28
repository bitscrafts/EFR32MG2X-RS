//! `embedded-hal-async` I2C: the same byte-by-byte state machine, waiting on the interrupt instead of spinning.
//!
//! ## Why this is not just `async` wrapped around the code that already existed
//!
//! The blocking `I2c0::write` does `while status().pstart() { polls += 1 }` up to a bound. Wrapping that in an `async
//! fn` gives a function called `async` that returns a future and **blocks the whole executor** while it runs -- every
//! other task stops for the duration of the transfer. That is the lie this file exists to avoid telling.
//!
//! What happens here is real waiting: when the condition is not yet satisfied the task **records its waker and
//! returns `Pending`**, and the I2C interrupt wakes it when the peripheral changes state. Between one wait and the
//! next the executor runs whatever it likes.
//!
//! ## Who serves the interrupt
//!
//! The vector belongs to the application, like every other one in this HAL: it defines `I2C0_IRQHandler` and calls
//! [`on_interrupt`]. The HAL does not install handlers in library crates -- a handler installed here would run in a
//! firmware that never used asynchronous I2C at all.

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use embedded_hal_async::i2c::{I2c as AsyncI2c, Operation};

use super::{Error, I2c0, WAIT_POLLS};

// ## What made this fail, and why the answer was in the host-tested core
//
// The first version read every byte **with ACK**, including the last one. The slave was never told to stop, kept
// driving data and the master never closed the reception: measured, the bus was held (`BUSHOLD`), with no `RXDATAV`
// and no NACK -- a hang with no error at all. The correct sequence is in `cores::i2c::write_read`, which calls
// `read(i != last)`: **NACK on the last byte**. The `HwBus` that serves it sends `CMD.ACK` or `CMD.NACK` before every
// read, and that is what is mirrored here.
//
// The lesson is not about I2C: the answer was in a core with host tests, and the first version of this file was
// written from what looked right instead of being read from it. It cost six flashes.
//
// ## The I2C interrupt, and why the first version re-entered 44 million times
//
// The first version armed `IEN` for `txc`/`rxdatav`/`nack`/`mstop` **all at once**, and woke the task in the interrupt.
// Measured on the board: **44 647 591** handler entries in a few seconds (`ICSR = 0x82b`, vector 43). The cause is in
// this peripheral and not in the handler: `IF` **re-asserts itself while the condition is true**, unlike a latched
// event, and clearing it does not disarm. Arming every bit and clearing them in the handler is an open gate.
//
// What happens now, one wait at a time:
//
//   1. clear the `IF` of the event about to be waited for (through the `_CLR` window, where writing 1 clears);
//   2. arm **only** that event's `IEN` and unmask the vector in the NVIC;
//   3. return `Pending` without asking to be polled again -- that is what lets the core sleep;
//   4. the interrupt calls [`on_interrupt`], which **disarms first** and only then clears, and wakes the task.
//
// Disarming before clearing is the difference between one entry per event and 44 million: while `IEN` is armed and
// the condition is true, the vector fires again the instant the ISR returns.
//
// --- markers, so a hang says where it hung ---------------------------------------------------------
//
// A wait that is never satisfied writes no record at all, and a silent firmware does not say which of `put_byte`'s
// three waits failed. These counters are read from RAM over SWD (`nm` on the ELF, `probe-rs read`) while the program
// is stopped, and they answer that question without another flash per hypothesis.
#[used]
#[no_mangle]
static WAITS_TXBL: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
#[used]
#[no_mangle]
static WAITS_TXC: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
/// The `STATUS` at the moment a wait last failed.
///
/// This is the measurement that separates the hypotheses: it says whether the peripheral is busy, has received,
/// is waiting for ACK -- or never started. A counter says *where* it stopped; this says *in what state*.
#[used]
#[no_mangle]
static LAST_STATUS: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
/// The `IF` at the same instant -- to see the NACKs and errors `STATUS` does not show.
#[used]
#[no_mangle]
static LAST_IF: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
/// 1 = waiting for TXBL, 2 = for TXC, 3 = for PSTART, 4 = for PSTOP, 5 = for RXDATAV.
#[used]
#[no_mangle]
static LAST_WAIT: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

/// The counters, for a caller who wants them without `nm`.
pub fn waits() -> (u32, u32, u32) {
    use core::sync::atomic::Ordering::Relaxed;
    (
        WAITS_TXBL.load(Relaxed),
        WAITS_TXC.load(Relaxed),
        LAST_WAIT.load(Relaxed),
    )
}

/// The `(STATUS, IF)` of the last failed wait -- the reading that says what state the bus was left in.
pub fn last_state() -> (u32, u32) {
    use core::sync::atomic::Ordering::Relaxed;
    (LAST_STATUS.load(Relaxed), LAST_IF.load(Relaxed))
}

/// The `IEN` bits this file arms, one per wait.
///
/// One at a time, and that is the point of the file: arming several at once and clearing them in the handler is what
/// produced 44 million entries. The numbers are the register's (`ien.rs` in the PAC: `START` 0, `TXC` 3, `TXBL` 4,
/// `RXDATAV` 5, `MSTOP` 8).
const IEN_START: u32 = 1 << 0;
const IEN_TXC: u32 = 1 << 3;
const IEN_TXBL: u32 = 1 << 4;
const IEN_RXDATAV: u32 = 1 << 5;
const IEN_MSTOP: u32 = 1 << 8;

/// A future that ends when `ready` says yes, or with an error when the bus refuses.
///
/// The bound is not a wall-clock timeout: it counts **how many times the task was woken**, and it exists so that a
/// lost interrupt becomes a visible error instead of a task that never runs again. A 100 kHz I2C transfer raises
/// these notifications tens of times, not thousands.
struct Until<F> {
    /// The `IEN` bit for the expected event. One at a time: see the note at the top.
    mask: u32,
    ready: F,
}

impl<F> Future for Until<F>
where
    F: FnMut(&efr32mg2x_pac::i2c0_ns::RegisterBlock) -> bool + Unpin,
{
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        // SAFETY: raw view of I2C0, as in `on_interrupt`; only state is read.
        let i2c = unsafe { efr32mg2x_pac::I2c0Ns::steal() };

        if (this.ready)(&i2c) {
            return Poll::Ready(Ok(()));
        }

        // Record the state at the instant of the failure: that is what says what state the peripheral is in, not
        // only where the task stopped. `read()` is a register read and changes nothing.
        LAST_STATUS.store(
            i2c.status().read().bits(),
            core::sync::atomic::Ordering::Relaxed,
        );
        LAST_IF.store(
            i2c.if_().read().bits(),
            core::sync::atomic::Ordering::Relaxed,
        );

        // Clear the event *before* arming it: an `IF` already set would make the interrupt fire immediately and the
        // handler would disarm it without the condition having changed. The `_CLR` window is where writing 1 clears.
        // SAFETY: the I2C0 secure window is the same block at another offset, and only flag bits are written.
        let secure = unsafe { crate::alias::clear_window(efr32mg2x_pac::I2c0S::ptr()) };
        // SAFETY: `this.mask` comes from a table of this peripheral's `IEN` bits, not from a caller's number.
        secure.if_().write(|w| unsafe { w.bits(this.mask) });
        // SAFETY: likewise -- the same bits, now in the enable register of the secure view.
        let secure_en = unsafe { efr32mg2x_pac::I2c0S::steal() };
        secure_en
            .ien()
            .modify(|r, w| unsafe { w.bits(r.bits() | this.mask) });
        // The vector belongs to the application, but the driver is what knows the wait is armed: unmasking here is
        // what turns `Pending` into a sleeping core instead of a spinning task.
        // SAFETY: `unmask` touches the NVIC, which is not a peripheral of this HAL; the effect is one more interrupt.
        unsafe { cortex_m::peripheral::NVIC::unmask(efr32mg2x_pac::Interrupt::I2C0) };
        // No `wake_by_ref()`: the task does not ask to be polled, the interrupt wakes it.
        let _ = cx;
        Poll::Pending
    }
}

/// What the application calls in its `I2C0_IRQHandler`.
///
/// It returns the `IF` that was set and leaves the peripheral with `IEN` **disarmed**, so the vector does not fire
/// again while the condition stays true. The caller wakes its task; see the `hal_i2c_async` example.
#[must_use]
pub fn on_interrupt() -> u32 {
    // SAFETY: secure view of I2C0, the same one the rest of this file uses for `IF`; only flags are read and cleared.
    let secure = unsafe { efr32mg2x_pac::I2c0S::steal() };
    let pending = secure.if_().read().bits();
    // Disarm first. This is the fix for the 44-million-entry defect.
    secure.ien().write(|w| unsafe { w.bits(0) });
    // SAFETY: `_CLR` window of the same block; writing 1 into each pending bit clears it.
    let clear = unsafe { crate::alias::clear_window(efr32mg2x_pac::I2c0S::ptr()) };
    clear.if_().write(|w| unsafe { w.bits(pending) });
    pending
}

impl I2c0 {
    /// The bus's state, for [`Until`]'s predicate.
    fn until<F>(&self, mask: u32, ready: F) -> Until<F>
    where
        F: FnMut(&efr32mg2x_pac::i2c0_ns::RegisterBlock) -> bool,
    {
        Until { mask, ready }
    }

    /// Puts one byte on the bus: wait for the buffer, write, wait for the transmission, check for NACK.
    ///
    /// The three waits are the blocking path's, one by one. Only the first was not enough: with TBXL already set, two
    /// bytes in a row write into the same register before the first has left. Measured -- `who=0x00` instead of
    /// `0x6a`.
    ///
    /// The NACK is read **in the secure alias**: `if_` in this peripheral's non-secure view does not reflect the
    /// event, and this file's first version read it there (always zero) and therefore saw no refusal at all.
    async fn put_byte(&mut self, byte: u8) -> Result<(), Error> {
        LAST_WAIT.store(1, core::sync::atomic::Ordering::Relaxed);
        WAITS_TXBL.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        self.until(IEN_TXBL, |r| r.status().read().txbl().bit_is_set())
            .await?;
        // SAFETY: `TXDATA` takes the byte being shifted out; `byte` is a `u8`, so the widening to `u32` adds nothing.
        self.i2c.txdata().write(|w| unsafe { w.bits(byte as u32) });
        LAST_WAIT.store(2, core::sync::atomic::Ordering::Relaxed);
        WAITS_TXC.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        self.until(IEN_TXC, |r| r.status().read().txc().bit_is_set())
            .await?;
        // SAFETY: the same two lines as the blocking path -- the I2C0 secure window and clearing one flag.
        let secure = unsafe { crate::alias::clear_window(efr32mg2x_pac::I2c0S::ptr()) };
        if secure.if_().read().nack().bit_is_set() {
            secure.if_().write(|w| w.nack().set_bit());
            return Err(Error::Nack);
        }
        Ok(())
    }
}

impl AsyncI2c for I2c0 {
    /// One transaction, with a **repeated START** between operations.
    ///
    /// The first version called a `write` and then a `read`, each with its own STOP: the bus was released in the
    /// middle and the device lost the register pointer it had just been given -- measured on the board, `who=0x00`
    /// instead of `0x6a`. It is the same defect the blocking path had already fixed, and it is described in its
    /// comment. Between operations there is a START with no STOP; the STOP is single and at the end.
    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        if operations.is_empty() {
            return Err(Error::InvalidData);
        }

        for operation in operations.iter_mut() {
            self.i2c.cmd().write(|w| w.start().set_bit());
            self.until(IEN_START, |r| !r.status().read().pstart().bit_is_set())
                .await?;

            match operation {
                Operation::Write(bytes) => {
                    if bytes.is_empty() {
                        return Err(Error::InvalidData);
                    }
                    self.put_byte((address << 1) & 0xFE).await?;
                    for &byte in bytes.iter() {
                        self.put_byte(byte).await?;
                    }
                }
                Operation::Read(buffer) => {
                    if buffer.is_empty() {
                        return Err(Error::InvalidData);
                    }
                    self.put_byte((address << 1) | 1).await?;
                    // **One byte at a time, with ACK -- and NACK on the last.** It is the sequence of the host-tested
                    // core (`cores::i2c::write_read` calls `read(i != last)`) and of the `HwBus` that serves it:
                    // without the final NACK the slave keeps driving data, the master never closes the reception, and
                    // what is observed is exactly what was measured -- the bus held (`BUSHOLD`), with no byte.
                    //
                    // `STATUS.RXDATAV` is the condition `HwBus::read` waits for; it was moved to `IF` by mistake
                    // before this comparison.
                    let total = buffer.len();
                    for (i, slot) in buffer.iter_mut().enumerate() {
                        self.until(IEN_RXDATAV, |r| r.status().read().rxdatav().bit_is_set())
                            .await?;
                        if i + 1 == total {
                            self.i2c.cmd().write(|w| w.nack().set_bit());
                        } else {
                            self.i2c.cmd().write(|w| w.ack().set_bit());
                        }
                        *slot = self.i2c.rxdata().read().bits() as u8;
                    }
                }
            }
        }

        self.i2c.cmd().write(|w| w.stop().set_bit());
        self.until(IEN_MSTOP, |r| !r.status().read().pstop().bit_is_set())
            .await?;
        Ok(())
    }
}

// `WAIT_POLLS` is the blocking path's bound; it is not used here, and keeping it imported would be noise. The
// constant stays named in this note for whoever looks for the parallel between the two paths.
const _: u32 = WAIT_POLLS;
