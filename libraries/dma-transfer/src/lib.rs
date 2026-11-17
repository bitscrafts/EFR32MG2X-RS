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
