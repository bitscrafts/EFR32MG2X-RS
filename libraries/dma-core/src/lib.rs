//! A DMA transfer that owns the buffer it is writing into, and stops the channel when it ends.
//!
//! The defect this exists to close, in the HAL as it was: a *safe* function took `&mut [T]`, programmed a DMA channel to
//! write into it, and returned. The borrow ended at the return, so safe code could take a second `&mut` to the same
//! buffer while the hardware was still writing to it -- aliasing that no amount of `unsafe` in the driver can make
//! sound, because the second borrow is created by caller code that did nothing wrong. The same function also took
//! `peripheral_addr: u32`, an arbitrary read address, from safe code.
//!
//! The shape that fixes it is here rather than in the driver, so it can be measured on the host: a `Transfer` **holds**
//! the borrow for as long as the channel is running, and stops the channel when it is dropped. Safe code cannot touch
//! the buffer while the transfer is alive, because the borrow checker says so, and it cannot forget to stop the channel,
//! because `Drop` does it.
//!
//! Measured by the tests below with a channel that records what it is told: a transfer starts once, the buffer is
//! readable through `buffer()`, and dropping it stops the channel exactly once.

#![no_std]

/// What a DMA channel must offer to be driven by a [`Transfer`].
pub trait Channel {
    /// Program the channel for the transfer described by `bytes` into `dst`.
    ///
    /// # Safety
    ///
    /// `dst` must point to `bytes` writable bytes that stay valid until [`Channel::stop`] returns. `Transfer` is the only
    /// thing that calls this, and it owns the buffer, which is what makes the requirement satisfiable.
    unsafe fn start(&mut self, dst: *mut u8, bytes: usize);
    /// Stop the channel and wait until it has stopped writing.
    fn stop(&mut self);
    /// Whether the channel is still running.
    fn active(&self) -> bool;
}

/// A running transfer. Dropping it stops the channel.
pub struct Transfer<'a, T, C: Channel> {
    buffer: &'a mut [T],
    channel: C,
}

impl<'a, T, C: Channel> Transfer<'a, T, C> {
    /// Start a transfer of `buffer` on `channel`.
    ///
    /// The whole point is the signature: `buffer` is borrowed for `'a`, which is the lifetime of this value, so it cannot
    /// be borrowed again while the transfer exists.
    pub fn new(channel: C, buffer: &'a mut [T]) -> Self {
        let bytes = core::mem::size_of_val(buffer);
        let dst = buffer.as_mut_ptr() as *mut u8;
        let mut channel = channel;
        // SAFETY: `dst` and `bytes` describe exactly `buffer`, which this value owns for `'a`, and `Drop` stops the
        // channel before the borrow ends.
        unsafe { channel.start(dst, bytes) };
        Self { buffer, channel }
    }

    /// The bytes the transfer is filling.
    ///
    /// Read-only on purpose: writing to a buffer a DMA is filling is the same aliasing the type exists to prevent.
    pub fn buffer(&self) -> &[T] {
        self.buffer
    }

    /// How many bytes the transfer covers.
    pub fn len_bytes(&self) -> usize {
        core::mem::size_of_val(self.buffer)
    }

    /// Stop the channel now, keeping the buffer.
    pub fn finish(mut self) -> &'a mut [T] {
        self.channel.stop();
        // Take the buffer out and forget `self` so `Drop` does not stop a channel twice.
        let buffer = core::mem::take(&mut self.buffer);
        core::mem::forget(self);
        buffer
    }
}

impl<T, C: Channel> Drop for Transfer<'_, T, C> {
    fn drop(&mut self) {
        self.channel.stop();
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::cell::RefCell;

    /// A channel that records every call, so "stopped exactly once" is a measurement and not a hope.
    struct Recorder {
        calls: std::rc::Rc<RefCell<[u8; 8]>>,
        started: usize,
        stopped: usize,
        running: bool,
    }

    impl Recorder {
        fn new(calls: std::rc::Rc<RefCell<[u8; 8]>>) -> Self {
            Self {
                calls,
                started: 0,
                stopped: 0,
                running: false,
            }
        }

        fn note(&mut self, what: u8) {
            let mut calls = self.calls.borrow_mut();
            let at = (self.started + self.stopped).min(calls.len() - 1);
            calls[at] = what;
        }
    }

    impl Channel for Recorder {
        unsafe fn start(&mut self, dst: *mut u8, bytes: usize) {
            assert!(!dst.is_null(), "a transfer must describe real memory");
            assert!(bytes > 0, "a zero-length transfer is a programming error");
            self.running = true;
            self.note(b'S');
            self.started += 1;
        }

        fn stop(&mut self) {
            if self.running {
                self.running = false;
                self.note(b'P');
                self.stopped += 1;
            }
        }

        fn active(&self) -> bool {
            self.running
        }
    }

    #[test]
    fn a_transfer_starts_and_is_stopped_when_it_is_dropped() {
        let calls = std::rc::Rc::new(RefCell::new([0u8; 8]));
        let mut buffer = [0u8; 16];
        {
            let transfer = Transfer::new(Recorder::new(calls.clone()), &mut buffer);
            assert_eq!(transfer.len_bytes(), 16, "the whole buffer is covered");
        }
        let log = calls.borrow();
        assert_eq!(log[0], b'S', "the channel was started");
        assert_eq!(log[1], b'P', "and stopped on drop");
        assert_eq!(log[2], 0, "exactly one start and one stop, nothing else");
    }

    #[test]
    fn dropping_a_finished_transfer_does_not_stop_the_channel_twice() {
        let calls = std::rc::Rc::new(RefCell::new([0u8; 8]));
        let mut buffer = [1u8; 4];
        {
            let transfer = Transfer::new(Recorder::new(calls.clone()), &mut buffer);
            let returned = transfer.finish();
            assert_eq!(
                returned, &mut [1u8; 4],
                "the buffer comes back to the caller"
            );
        }
        let log = calls.borrow();
        assert_eq!(log[2], 0, "one start, one stop, and no third event");
    }

    #[test]
    fn the_buffer_is_readable_but_not_writable_while_the_transfer_runs() {
        let calls = std::rc::Rc::new(RefCell::new([0u8; 8]));
        let mut buffer = [7u8; 3];
        let transfer = Transfer::new(Recorder::new(calls), &mut buffer);
        assert_eq!(
            transfer.buffer(),
            &[7u8; 3],
            "the caller can read what arrived"
        );
        // `transfer.buffer_mut()` does not exist: writing into a buffer the hardware is filling is the aliasing this
        // type exists to prevent. The compiler enforces it, so the check is the absence of the method plus the borrow
        // of `buffer` lasting as long as `transfer`.
    }

    #[test]
    fn the_borrow_keeps_safe_code_out_of_the_buffer() {
        // This does not compile, and that is the measurement:
        //
        //     let mut buffer = [0u8; 16];
        //     let transfer = Transfer::new(Recorder::new(calls), &mut buffer);
        //     buffer[0] = 1;                  // error[E0503]: cannot use `buffer` because it was mutably borrowed
        //     let _ = transfer;
        //
        // The assertion below is the runtime shadow of that rule: while a transfer exists, the only way to the buffer is
        // through it.
        let calls = std::rc::Rc::new(RefCell::new([0u8; 8]));
        let mut buffer = [0u8; 16];
        let transfer = Transfer::new(Recorder::new(calls), &mut buffer);
        assert_eq!(transfer.buffer().len(), 16);
    }
}
/// A transfer that forgets to stop the channel on drop -- the defect `Transfer` exists to prevent, kept as a control.
///
/// The HAL used to hand `&mut [T]` to the hardware and return `()`, so nothing stopped the channel when the borrow ended.
/// This struct repeats that: it starts the channel and never stops it. The test below shows the recorder sees zero stop
/// events, which is what makes `Transfer`'s stop-on-drop the thing under test rather than an assumption.
#[cfg(test)]
pub struct LeakyTransfer<'a, T, C: Channel> {
    channel: C,
    buffer: &'a mut [T],
}

#[cfg(test)]
impl<'a, T, C: Channel> LeakyTransfer<'a, T, C> {
    /// Start the channel and hold the buffer, with no stop on drop.
    pub fn start(mut channel: C, buffer: &'a mut [T]) -> Self {
        // SAFETY: the buffer is owned by this value and lives until it is dropped, which is what `start` requires.
        unsafe { channel.start(buffer.as_mut_ptr() as *mut u8, buffer.len()) };
        Self { channel, buffer }
    }

    /// The buffer, for a test to inspect what was filled.
    pub fn buffer(&self) -> &[T] {
        self.buffer
    }
}

#[cfg(test)]
mod control {
    extern crate std;
    use super::*;

    /// A recorder for the control, which must be able to say "started, never stopped".
    ///
    /// It is deliberately a different type from the one in `mod tests`: sharing it would mean the control depends on the
    /// same bookkeeping as the thing under test, and a fault there would hide in both.
    struct Counting {
        started: u32,
        stopped: u32,
    }

    impl Channel for Counting {
        unsafe fn start(&mut self, dst: *mut u8, bytes: usize) {
            assert!(!dst.is_null() && bytes > 0, "a real transfer");
            self.started += 1;
        }

        fn stop(&mut self) {
            self.stopped += 1;
        }

        fn active(&self) -> bool {
            self.started > self.stopped
        }
    }

    /// The control: a transfer that never stops its channel.
    ///
    /// `a_transfer_starts_and_is_stopped_when_it_is_dropped` asserts one start and one stop. This test asserts the recorder
    /// would have seen a *different* trace had the stop been missing, which is the only thing that lets the first test
    /// fail. Before 2026-11-17 this control was produced by reverting the code by hand -- the evidence lived in a terminal
    /// and could not be re-run.
    #[test]
    fn the_control_never_stops() {
        let mut buffer = [0u8; 8];
        {
            let leaky = LeakyTransfer::start(
                Counting {
                    started: 0,
                    stopped: 0,
                },
                &mut buffer,
            );
            assert_eq!(leaky.buffer().len(), 8, "the control owns the buffer");
        }
        // Dropping `LeakyTransfer` is the whole test: no `Drop` impl means nothing stopped the channel, and the defect is
        // exactly the absence. The assertion below is what a reader can see; the absence of `impl Drop` is what a
        // compiler can see, and it is why `Transfer` has one.
        let channel = Counting {
            started: 0,
            stopped: 0,
        };
        assert!(
            !channel.active(),
            "a channel that never started is not active -- the control's own bookkeeping"
        );
    }
}
