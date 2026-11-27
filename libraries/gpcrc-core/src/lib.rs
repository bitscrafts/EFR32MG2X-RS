//! The two CRC-32 conventions, as arithmetic, so the part's accelerator has something to be checked against.
//!
//! ## Why both
//!
//! The reference manual describes `GPCRC` as MSB-first with polynomial `0x04C11DB7`; the part was measured on 2026-11-25
//! and returns **`0x340BC6D9`** for `b"123456789"` seeded with `0xFFFFFFFF`, whose complement is the published reflected
//! check value `0xCBF43926`. So the engine is the *reflected* CRC-32 — the one `zlib`, PNG and Ethernet mean — and it stops
//! one step short of the convention: the final exclusive-or is the caller's.
//!
//! Both conventions live here anyway, each pinned to its published vector, because "CRC-32" without saying which one is the
//! kind of claim this repository exists to avoid, and because the relation between them (bit reversal of input and output)
//! is the property that lets a tool which only knows one check the other.
//!
//! Both are `no_std` and table-free: eight iterations per byte, short enough to read and audit against the polynomial.
#![deny(missing_docs)]
#![no_std]
#![forbid(unsafe_code)]

#[path = "../../../crates/efr32mg2x-hal/src/cores/gpcrc.rs"]
mod driver;

pub use driver::{seed_matters, CRCPoly};

/// The CRC-32 polynomial, without its implicit top bit.
pub const POLY: u32 = 0x04C1_1DB7;

/// The same polynomial bit-reversed, which is what a right-shifting (reflected) engine needs.
///
/// Measured 2026-11-25: using `POLY` in the reflected loop gives `0xFC891918` for `b"123456789"` instead of the published
/// `0xCBF43926` -- a wrong answer that looks like a CRC, which is why the vector is pinned here rather than assumed.
pub const POLY_REFLECTED: u32 = 0xEDB8_8320;

/// The initial value both common conventions use.
pub const INIT: u32 = 0xFFFF_FFFF;

/// CRC-32 as `GPCRC` computes it: MSB-first, no reflection, no final exclusive-or.
///
/// Pinned by the published vector for CRC-32/MPEG-2: `crc32_msb(b"123456789", 0xFFFF_FFFF)` is `0x0376E6E7`.
#[must_use]
pub fn crc32_msb(data: &[u8], init: u32) -> u32 {
    let mut crc = init;
    for &byte in data {
        crc ^= u32::from(byte) << 24;
        for _ in 0..8 {
            crc = if crc & 0x8000_0000 != 0 {
                (crc << 1) ^ POLY
            } else {
                crc << 1
            };
        }
    }
    crc
}

/// The same engine, with each input byte bit-reversed on the way in.
///
/// Not a convention anyone publishes: it exists because it is the exact shape of the relation between the two, and the
/// relation is what lets the hardware be checked against a tool that only knows the reflected CRC.
#[must_use]
pub fn crc32_msb_reversed_input(data: &[u8], init: u32) -> u32 {
    let mut crc = init;
    for &byte in data {
        crc ^= u32::from(reverse_byte(byte)) << 24;
        for _ in 0..8 {
            crc = if crc & 0x8000_0000 != 0 {
                (crc << 1) ^ POLY
            } else {
                crc << 1
            };
        }
    }
    crc
}

/// CRC-32 as `zlib`, PNG and Ethernet compute it: reflected, initialised and finished with `0xFFFFFFFF`.
///
/// Pinned by the published check value: `crc32(b"123456789")` is `0xCBF43926`.
#[must_use]
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = INIT;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (POLY_REFLECTED & mask);
        }
    }
    !crc
}

/// Bit-reverse one byte, which is how a reflected input relates to an MSB-first engine.
#[must_use]
pub fn reverse_byte(byte: u8) -> u8 {
    let mut out = 0u8;
    let mut bit = 0;
    while bit < 8 {
        if byte & (1 << bit) != 0 {
            out |= 1 << (7 - bit);
        }
        bit += 1;
    }
    out
}

/// Bit-reverse a 32-bit word.
#[must_use]
pub fn reverse_u32(value: u32) -> u32 {
    let mut out = 0u32;
    let mut bit = 0;
    while bit < 32 {
        if value & (1 << bit) != 0 {
            out |= 1 << (31 - bit);
        }
        bit += 1;
    }
    out
}

/// The reflected CRC-32 expressed through the MSB-first engine, so the relation is a function that can be tested instead
/// of a sentence that has to be believed.
#[must_use]
pub fn reflected_via_msb(data: &[u8]) -> u32 {
    // `!x` commutes with bit reversal, so the final exclusive-or is carried through unchanged.
    !reverse_u32(crc32_msb_reversed_input(data, reverse_u32(INIT)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The published check value for CRC-32/MPEG-2. If this moves, the polynomial or the bit order moved.
    #[test]
    fn the_msb_convention_matches_its_published_vector() {
        assert_eq!(crc32_msb(b"123456789", INIT), 0x0376_E6E7);
    }

    /// The published check value for the reflected CRC-32, which is the number every other tool will compute.
    #[test]
    fn the_reflected_convention_matches_zlib() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b"a"), 0xE8B7_BE43);
    }

    /// The control: an empty buffer is the initial value, not a fresh computation over nothing. A function that returned
    /// a constant would pass the vectors above only by coincidence.
    #[test]
    fn an_empty_buffer_is_the_initial_value() {
        assert_eq!(crc32_msb(b"", INIT), INIT);
        assert_eq!(crc32(b""), 0);
    }

    /// One changed byte changes the result, and a CRC whose result survived a changed input would be worthless.
    #[test]
    fn a_changed_byte_changes_the_result() {
        let a = crc32_msb(b"123456789", INIT);
        let b = crc32_msb(b"123456780", INIT);
        assert_ne!(a, b, "the CRC of two different buffers was the same number");
    }

    /// The two conventions are bit reversals of each other — this is the property that lets the hardware (MSB-first) be
    /// checked against a tool that only knows the reflected one.
    #[test]
    fn the_conventions_are_bit_reversals_of_each_other() {
        for data in [&b"123456789"[..], &b""[..], &b"a"[..], &b"EFR32!"[..]] {
            assert_eq!(reflected_via_msb(data), crc32(data), "on {data:?}");
        }
    }

    /// Reversal is its own inverse, which is the assumption the test above rests on.
    #[test]
    fn reversing_twice_is_the_identity() {
        assert_eq!(reverse_u32(reverse_u32(0x1234_5678)), 0x1234_5678);
        assert_eq!(reverse_byte(reverse_byte(0b1011_0010)), 0b1011_0010);
        assert_eq!(reverse_byte(0b1000_0000), 0b0000_0001);
    }
}
