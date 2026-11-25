//! A record in RAM, written the same way by every instrument.
//!
//! ## Why this is a module and not twenty copies
//!
//! Most examples in this crate end by writing a line into a mailbox that a host reads back over the debugger, and every
//! one of them used to carry its own formatter: the same twenty lines of `s`/`u`/`h`/`buf`, copied, with small
//! differences — one truncated silently, one printed `u64` as `u32`, one had no hex at all. A record is evidence, and
//! evidence formatted differently in each instrument is evidence nobody can compare across instruments.
//!
//! ## What it guarantees
//!
//! - **It never panics and never writes out of bounds.** An over-long record is truncated, and the record says so: the
//!   buffer is never left with a half-written number that reads as a different number.
//! - **Numbers are exact or absent.** `u` prints every digit of a `u64`; `i` handles `i64::MIN` without overflow; `h`
//!   prints two digits, zero-padded, because a byte compares better as `0a` than as `a`.
//! - **No allocation, no formatting machinery.** Fixed buffer, `no_std`, and no `core::fmt` — the point is to be usable
//!   in a panic path and to cost the same everywhere.
//!
//! The arithmetic is host-tested through `libraries/record-core`, which compiles this file by path.

/// A line of evidence in a fixed buffer.
///
/// `N` is the caller's buffer size: an instrument that knows its record is 40 bytes can say so, and a builder that
/// cannot predict it can ask for 200. When the record does not fit, the text stops — and [`Record::truncated`] says it
/// did, so a reader is never shown a number that was cut in half.
pub struct Record<const N: usize> {
    buf: [u8; N],
    len: usize,
    truncated: bool,
}

impl<const N: usize> Default for Record<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Record<N> {
    /// An empty record.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buf: [0; N],
            len: 0,
            truncated: false,
        }
    }

    /// Append text.
    pub fn s(&mut self, text: &[u8]) -> &mut Self {
        for byte in text {
            self.push(*byte);
        }
        self
    }

    /// Append an unsigned number, every digit of it.
    pub fn u(&mut self, value: impl Into<u64>) -> &mut Self {
        let mut value = value.into();
        if value == 0 {
            self.push(b'0');
            return self;
        }
        let mut digits = [0u8; 20];
        let mut n = 0;
        while value > 0 {
            digits[n] = b'0' + (value % 10) as u8;
            value /= 10;
            n += 1;
        }
        while n > 0 {
            n -= 1;
            let digit = digits[n];
            self.push(digit);
        }
        self
    }

    /// Append a signed number. The magnitude is taken without `abs`, so `i64::MIN` does not overflow.
    pub fn i(&mut self, value: impl Into<i64>) -> &mut Self {
        let value = value.into();
        if value < 0 {
            self.push(b'-');
            // `unsigned_abs` is the whole reason this does not panic on `i64::MIN`.
            return self.u(value.unsigned_abs());
        }
        self.u(value.unsigned_abs())
    }

    /// Append a byte as two hexadecimal digits.
    ///
    /// Two digits always, zero-padded: a record that prints `5` for one byte and `12` for another cannot be read back by
    /// a person or a script without knowing which is which.
    pub fn h(&mut self, value: u8) -> &mut Self {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        self.push(HEX[usize::from(value >> 4)]);
        self.push(HEX[usize::from(value & 0xF)]);
        self
    }

    /// Append `1` or `0`, the shape a record uses for a verdict field.
    pub fn bit(&mut self, value: bool) -> &mut Self {
        self.push(if value { b'1' } else { b'0' });
        self
    }

    /// Append one byte, or record that the buffer is full.
    fn push(&mut self, byte: u8) {
        if let Some(slot) = self.buf.get_mut(self.len) {
            *slot = byte;
            self.len += 1;
        } else {
            self.truncated = true;
        }
    }

    /// The record as text.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    /// How many bytes the record holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether nothing has been written.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Whether anything was dropped because the buffer was full.
    ///
    /// An instrument that truncates must be able to say so: a record whose last field is missing reads as a record whose
    /// last field was zero.
    #[must_use]
    pub fn truncated(&self) -> bool {
        self.truncated
    }

    /// Empty the record and forget the truncation.
    pub fn clear(&mut self) -> &mut Self {
        self.len = 0;
        self.truncated = false;
        self
    }
}
