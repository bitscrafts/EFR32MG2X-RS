//! Interrupt-driven LDMA completion -- [C2-DMA.4].
//!
//! [`Channel::transfer`] and [`Channel::transfer_chain`] finish by reading `IF` in a loop: the core spins while the
//! controller works. This module adds the other way to end a transfer -- the controller requests `LDMA_IRQ` when a
//! descriptor's count runs out, the handler acknowledges the flag and records the channel, and the caller sleeps in
//! `WFI` until the record appears. Between arming and completion the core executes no loop at all.
//!
//! # Three bits, and which one this item measures
//!
//! A done interrupt reaches the core only if three things line up:
//!
//! | where | bit | who sets it here |
//! |---|---|---|
//! | descriptor `CTRL` | `DONEIEN`, bit 20 | `start` always sets it |
//! | controller `IEN` | `CHDONE[channel]` | [`Gate`]: the one thing the two configurations differ in |
//! | core `NVIC` | the `LDMA_IRQ` vector | `start` unmasks it in both configurations |
//!
//! The control for this item is therefore one bit wide: same descriptor, same vector, same data, `IEN.CHDONE`
//! cleared. If the flag still latches and nothing wakes, the wake came from the gate and not from the transfer
//! finishing. If instead the deadline is what ends the wait, the wait cannot be completed by polling either --
//! it has no poll path.
//!
//! # Who installs the handler
//!
//! This HAL does not define `LDMA_IRQHandler`, for the same reason it does not define the two GPIO vectors
//! ([`crate::extint`]): a library that takes a vector from every application is a library the application cannot
//! configure. The handler is one call:
//!
//! ```ignore
//! #[no_mangle]
//! pub extern "C" fn LDMA_IRQHandler() {
//!     efr32mg2x_hal::dma::irq::service();
//! }
//! ```
//!
//! A handler that does not call [`service`] leaves the channel's flag set and the vector re-enters for ever: `IF`
//! is a level, not a pulse, and this part's `IF` is cleared through the `+0x2000` alias, not by writing 1 to the
//! plain register. [`service`] clears every bit it reads, which is the whole reason the acknowledgement lives here
//! and not in a caller's macro.

use core::sync::atomic::{AtomicU32, Ordering};

use super::{Channel, DmaError, TransferElement};
use crate::pac;

/// Channels whose `DONE` the handler has seen since the last reset, bit `n` = channel `n`.
static DONE: AtomicU32 = AtomicU32::new(0);

/// Entries the vector has made and found a flag for.
///
/// An entry with nothing pending returns before this moves, so the number says how many completions were
/// delivered rather than how often the line was pended.
static ENTRIES: AtomicU32 = AtomicU32::new(0);

/// Which gate the channel's done request passes through.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gate {
    /// `IEN.CHDONE` set: when the descriptor's count runs out the controller requests the vector.
    Enabled,
    /// `IEN.CHDONE` cleared, everything else identical -- the same descriptor, the same unmasked vector, the same
    /// data. The `DONE` flag still latches in `IF` (measured); nobody is told about it.
    Closed,
}

/// Serve the vector: acknowledge every flag it reports and record the channels that finished.
///
/// Returns the `IF` bits acknowledged. Must be called from `LDMA_IRQHandler`.
///
/// Every bit read is cleared, not only the ones a caller is waiting for: the vector's line is asserted while any
/// bit of `IF` is set, so a handler that leaves one behind is re-entered immediately, for ever.
pub fn service() -> u32 {
    // SAFETY: LDMA's register block is at a fixed address for the life of the program and the pointer comes from
    // the PAC; the accesses below are one status read and one write through the clear alias.
    let ldma = unsafe { &(*pac::LdmaS::ptr()) };
    let flags = ldma.if_().read().bits();
    if flags == 0 {
        return 0;
    }

    // Acknowledge first: the flag is a level that survives the handler, and counting before clearing would let the
    // same flag be counted twice if the handler were re-entered.
    // SAFETY: the alias window maps the same `IF` word with write-one-to-clear semantics, and `flags` is what this
    // handler just read -- no other channel's flag is touched.
    unsafe {
        crate::alias::clear_window(pac::LdmaS::ptr())
            .if_()
            .write(|w| w.bits(flags));
    }

    DONE.fetch_or(flags, Ordering::Relaxed);
    ENTRIES.fetch_add(1, Ordering::Relaxed);
    flags
}

/// How many times the vector has entered and found work.
#[must_use]
pub fn entries() -> u32 {
    ENTRIES.load(Ordering::Relaxed)
}

/// True once the handler has recorded a completion for `channel`.
#[must_use]
pub fn completed(channel: u8) -> bool {
    DONE.load(Ordering::Relaxed) & (1u32 << channel) != 0
}

/// True while the controller's `IEN.CHDONE` bit for `channel` is set.
#[must_use]
pub fn gate(channel: u8) -> bool {
    // SAFETY: fixed block address, pointer from the PAC; this is a status read.
    let ldma = unsafe { &(*pac::LdmaS::ptr()) };
    ldma.ien().read().chdone().bits() & (1u8 << channel) != 0
}

/// The per-channel `DONE` flag as the hardware holds it, whether or not anybody was told.
#[must_use]
pub fn flag(channel: u8) -> bool {
    // SAFETY: as above.
    let ldma = unsafe { &(*pac::LdmaS::ptr()) };
    ldma.if_().read().bits() & (1u32 << channel) != 0
}

/// Drop the record and the hardware flag for `channel`, leaving `IEN` as it is.
pub fn reset(channel: u8) {
    let bit = 1u32 << channel;
    DONE.fetch_and(!bit, Ordering::Relaxed);
    // SAFETY: one bit of `IF`, cleared through the alias window that gives it write-one-to-clear semantics.
    unsafe {
        crate::alias::clear_window(pac::LdmaS::ptr())
            .if_()
            .write(|w| w.bits(bit));
    }
}

/// What a wait ended with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Wait {
    /// The handler recorded the channel before the deadline expired.
    pub done: bool,
    /// How many times the core entered `WFI` in this wait. One means the loop body ran once: arm, sleep, wake.
    pub sleeps: u32,
    /// The channel's `DONE` flag at the moment the wait returned. `true` with `done == false` is the control:
    /// the transfer finished, the flag latched, and nothing told the core.
    pub flag: bool,
}

/// Sleep until the handler records `channel`, or until `expired` says the window is over.
///
/// `expired` is the application's clock, not this crate's: the only thing that can wake a sleeping core is an
/// armed interrupt, and choosing which one is the product's decision -- the same rule that keeps this crate from
/// defining the part's vectors. The example arms `SysTick` for it.
///
/// The order of the two checks is not incidental. `completed` is read first, so a completion that arrives at the
/// same moment the deadline expires counts as done; and `WFI` is entered only after both were found false, which
/// is what makes `sleeps` a measurement of sleeping rather than of spinning.
pub fn wait<D: Fn() -> bool>(channel: u8, expired: D) -> Wait {
    let mut sleeps = 0u32;
    loop {
        if completed(channel) {
            return Wait {
                done: true,
                sleeps,
                flag: flag(channel),
            };
        }
        if expired() {
            return Wait {
                done: false,
                sleeps,
                flag: flag(channel),
            };
        }
        cortex_m::asm::wfi();
        sleeps += 1;
    }
}

/// A transfer that is running, and whose destination buffer this value owns until it ends.
///
/// The borrow is the point: the controller writes into the buffer while the caller holds this guard, so safe code
/// cannot take a second `&mut` to memory the hardware is still filling. Dropping it disables the channel, which is
/// also what clears the flag a caller may have been waiting for.
pub struct Pending<'a, T, const N: u8> {
    channel: &'a mut Channel<N>,
    dst: &'a mut [T],
}

impl<'a, T, const N: u8> Pending<'a, T, N> {
    /// Sleep until this transfer's `DONE` is recorded, or until the deadline expires.
    pub fn wait<D: Fn() -> bool>(&mut self, expired: D) -> Wait {
        wait(N, expired)
    }

    /// The destination the controller is filling.
    ///
    /// Read-only: writing to a buffer a DMA is filling is the aliasing this type exists to prevent. Read through
    /// it only after [`wait`](Self::wait) has returned `done`.
    pub fn buffer(&self) -> &[T] {
        self.dst
    }
}

impl<T, const N: u8> Drop for Pending<'_, T, N> {
    fn drop(&mut self) {
        // Disabling the channel is also what makes the caller's borrow valid again: nothing is left writing into
        // the buffer after this returns.
        self.channel.stop();
    }
}

impl<const N: u8> Channel<N> {
    /// Start a memory-to-memory transfer that ends with an interrupt, without waiting for it.
    ///
    /// Returns a guard that owns `dst` until it is dropped, so the buffer cannot be read or written by safe code
    /// while the controller is filling it. [`Pending::wait`] is how the caller learns that it finished.
    ///
    /// # Errors
    ///
    /// `InvalidLength` for slices of different lengths, empty, or longer than 2047 units; `Unsupported` for a
    /// channel this HAL does not program; `Busy` if the channel will not go idle first.
    pub fn start_interrupt<'a, T>(
        &'a mut self,
        src: &[T],
        dst: &'a mut [T],
        gate: Gate,
    ) -> Result<Pending<'a, T, N>, DmaError>
    where
        T: TransferElement,
    {
        if src.len() != dst.len() || src.is_empty() || src.len() > 2047 {
            return Err(DmaError::InvalidLength);
        }
        if N != 0 {
            return Err(DmaError::Unsupported);
        }

        let count = src.len() as u16;
        let size = T::SIZE;
        let src_ptr = src.as_ptr() as u32;
        let dst_ptr = dst.as_mut_ptr() as u32;
        let bit = 1u32 << N;

        critical_section::with(|_cs| {
            // SAFETY: the LDMA block lives at a fixed address for the life of the program, the pointer comes from
            // the PAC, and every write below is inside the critical section that owns this channel.
            let ldma = unsafe { &(*pac::LdmaS::ptr()) };

            if ldma.chbusy().read().bits() & bit != 0 {
                return Err(DmaError::Busy);
            }

            // Disarm and wait for it: the control registers may only be rewritten once the channel has stopped,
            // and the disable is not instantaneous. Bounded, so a wedged channel reports `Busy` instead of hanging.
            // SAFETY: CHDIS is write-1-to-disable and one bit touches only this channel.
            unsafe {
                ldma.chdis().write(|w| w.bits(bit));
            }
            let mut spins = 0u32;
            while ldma.chstatus().read().chstatus().bits() & (bit as u8) != 0 {
                spins += 1;
                if spins > 100_000 {
                    return Err(DmaError::Busy);
                }
            }

            // SAFETY: the channel is disabled and idle, and these are the fields `transfer` programs, with
            // `DONEIEN` set in both configurations so the gate stays the only difference between them.
            unsafe {
                ldma.ch0_ctrl().write(|w| {
                    w.structtype()
                        .transfer()
                        .xfercnt()
                        .bits(count - 1) // hardware uses count-1
                        .size()
                        .bits(size as u8)
                        .srcinc()
                        .one()
                        .dstinc()
                        .one()
                        .blocksize()
                        .all()
                        .doneien()
                        .set_bit()
                });
                ldma.ch0_src().write(|w| w.bits(src_ptr));
                ldma.ch0_dst().write(|w| w.bits(dst_ptr));
                // One descriptor, no link: this module's transfers are a single leg by construction.
                ldma.ch0_link().write(|w| w.bits(0));

                // The record and the hardware flag start empty, so a completion counted here is this transfer's.
                DONE.fetch_and(!bit, Ordering::Relaxed);
                crate::alias::clear_window(pac::LdmaS::ptr())
                    .if_()
                    .write(|w| w.bits(bit));

                // The gate. `IEN` is read-modify-written rather than field-written so the other channels' bits and
                // the error bit survive.
                match gate {
                    Gate::Enabled => {
                        ldma.ien().modify(|r, w| w.bits(r.bits() | bit));
                    }
                    Gate::Closed => {
                        ldma.ien().modify(|r, w| w.bits(r.bits() & !bit));
                    }
                }

                // The vector is unmasked in both configurations: the control must differ in the gate and in
                // nothing else, or "IEN is what delivers" would not be the thing being measured.
                cortex_m::peripheral::NVIC::unpend(pac::Interrupt::LDMA);
                cortex_m::peripheral::NVIC::unmask(pac::Interrupt::LDMA);

                ldma.chen().write(|w| w.bits(bit));
                ldma.swreq().write(|w| w.bits(bit));
            }

            Ok(())
        })?;

        Ok(Pending { channel: self, dst })
    }
}
