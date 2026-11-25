//! Host-side copy of the record formatter, so its arithmetic is tested where it can run.
//!
//! The source lives in the HAL (`crates/efr32mg2x-hal/src/record.rs`) and is compiled here through `#[path]`, the same
//! arrangement the other `libraries/*-core` crates use: one copy, and the tests run on the host.

#![no_std]
#![forbid(unsafe_code)]

#[path = "../../../crates/efr32mg2x-hal/src/record.rs"]
mod record;

pub use record::Record;

#[cfg(test)]
mod tests {
    use super::Record;

    fn texto<const N: usize>(r: &Record<N>) -> &str {
        core::str::from_utf8(r.bytes()).expect("a record only ever holds ASCII")
    }

    #[test]
    fn zero_is_one_digit_not_an_empty_field() {
        let mut r = Record::<16>::new();
        r.u(0u32);
        assert_eq!(texto(&r), "0");
    }

    /// `u32::MAX` printed through a `u32` used to be the bug class this module exists to remove: a digit count that
    /// was hardcoded for ten digits and a value that needed all of them.
    #[test]
    fn the_largest_values_print_every_digit() {
        let mut r = Record::<32>::new();
        r.u(u32::MAX).s(b" ").u(u64::MAX);
        assert_eq!(texto(&r), "4294967295 18446744073709551615");
    }

    /// `i64::MIN` has no positive counterpart in `i64`, so a `-v` implementation panics on it. This one does not.
    #[test]
    fn the_most_negative_number_does_not_overflow() {
        let mut r = Record::<32>::new();
        r.i(i64::MIN);
        assert_eq!(texto(&r), "-9223372036854775808");
    }

    #[test]
    fn a_negative_number_keeps_its_sign() {
        let mut r = Record::<16>::new();
        r.i(-206i32);
        assert_eq!(texto(&r), "-206");
    }

    /// Two digits always: a record mixing `5` and `12` cannot be read back without knowing which byte is which.
    #[test]
    fn hex_is_zero_padded() {
        let mut r = Record::<16>::new();
        r.h(0x05).s(b" ").h(0xa5).s(b" ").h(0x00).s(b" ").h(0xff);
        assert_eq!(texto(&r), "05 a5 00 ff");
    }

    #[test]
    fn a_bit_is_one_or_zero() {
        let mut r = Record::<8>::new();
        r.bit(true).s(b",").bit(false);
        assert_eq!(texto(&r), "1,0");
    }

    /// The failure mode that matters: an over-long record must not panic, must not write past the buffer, and must say
    /// it was cut -- a truncated field reads as a field that was zero.
    #[test]
    fn a_full_buffer_truncates_and_says_so() {
        let mut r = Record::<4>::new();
        r.s(b"abcdefgh");
        assert_eq!(texto(&r), "abcd");
        assert!(r.truncated());
        assert_eq!(r.len(), 4);
    }

    /// The near-miss that keeps the test above honest: a record that fits is *not* marked truncated, and the counter
    /// stops exactly at the capacity.
    #[test]
    fn a_record_that_fits_is_not_marked_truncated() {
        let mut r = Record::<4>::new();
        r.s(b"abcd");
        assert_eq!(texto(&r), "abcd");
        assert!(!r.truncated());
    }

    /// A number cut in half would read as a smaller number, which is the reason `truncated` exists rather than a silent
    /// stop: `4294` out of `4294967295` is not evidence of anything.
    #[test]
    fn a_number_cut_by_the_buffer_is_flagged() {
        let mut r = Record::<6>::new();
        r.u(u32::MAX);
        assert_eq!(texto(&r), "429496");
        assert!(r.truncated());
    }

    #[test]
    fn clear_starts_again_including_the_truncation_flag() {
        let mut r = Record::<4>::new();
        r.s(b"abcdefgh");
        assert!(r.truncated());
        r.clear().s(b"ok");
        assert_eq!(texto(&r), "ok");
        assert!(!r.truncated());
        assert_eq!(r.len(), 2);
    }

    /// An empty record is empty, not a buffer of zero bytes with a length nobody checked.
    #[test]
    fn a_new_record_is_empty() {
        let r = Record::<8>::new();
        assert!(r.is_empty());
        assert_eq!(r.bytes(), b"");
    }
}
