//! The CRC accelerator's decisions, as arithmetic rather than as register writes.
//!
//! `GPCRC` itself is registers and a state machine, and the part of it that can be wrong in a way a host test catches is
//! small: which polynomial, how wide the result is, and whether the seed a caller passes actually reaches the engine. That
//! last one is the question this crate has been bitten by twice — a watchdog feed and an I2C transfer that both wrote a
//! register and did not get the effect the register implied — so it is a function here, and the driver exposes it as
//! `Gpcrc::seed_changes_the_result` for the instrument on the part to answer.
//!
//! The CRC arithmetic itself is in `libraries/gpcrc-core`, pinned to the published vectors of both conventions.

/// The polynomial a `GPCRC` is configured for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CRCPoly {
    /// `0x04C11DB7`, and on this part the engine is reflected: `!result` is the published `zlib` value (measured
    /// 2026-11-25, `raw=340bc6d9` for the standard check string).
    Crc32,
    /// The programmable 16-bit polynomial.
    Crc16,
}

impl CRCPoly {
    /// The polynomial, as a number a record can print.
    #[must_use]
    pub fn value(self) -> u32 {
        match self {
            CRCPoly::Crc32 => 0x04C1_1DB7,
            CRCPoly::Crc16 => 0x0000_1021,
        }
    }

    /// How many bits of the result are meaningful, which is what a caller needs in order to mask it.
    #[must_use]
    pub fn width(self) -> u32 {
        match self {
            CRCPoly::Crc32 => 32,
            CRCPoly::Crc16 => 16,
        }
    }

    /// The name a record prints.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            CRCPoly::Crc32 => "crc32",
            CRCPoly::Crc16 => "crc16",
        }
    }
}

/// Ask a CRC function whether its seed matters, by computing the same message with two seeds.
///
/// A function that ignores the seed returns the same number twice; that is the failure mode of a driver that writes `INIT`
/// without the `CMD.INIT` that loads it, and it produces a plausible CRC over the wrong message rather than an error.
/// Written as a generic so the host tests can hand it a constant and watch it say `false`.
pub fn seed_matters<F>(mut crc: F) -> bool
where
    F: FnMut(u32, &[u8]) -> u32,
{
    let a = crc(0xFFFF_FFFF, b"123456789");
    let b = crc(0x0000_0000, b"123456789");
    a != b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_polynomials_are_the_ones_the_manual_names() {
        assert_eq!(CRCPoly::Crc32.value(), 0x04C1_1DB7);
        assert_eq!(CRCPoly::Crc16.value(), 0x1021);
        assert_eq!(CRCPoly::Crc32.width(), 32);
        assert_eq!(CRCPoly::Crc16.width(), 16);
    }

    /// The control for `seed_matters`: a function that ignores its seed must be reported as such. Without this, the
    /// helper could return `true` unconditionally and every caller would look correct.
    #[test]
    fn a_function_that_ignores_the_seed_is_reported() {
        assert!(!seed_matters(|_seed, _data| 0x1234));
    }

    /// And a function whose seed changes the answer is reported as using it -- the near-miss that keeps the test above
    /// from being satisfied by a function that always says no.
    #[test]
    fn a_function_that_uses_the_seed_is_reported() {
        assert!(seed_matters(|seed, data| seed ^ u32::from(data[0])));
    }

    /// The name a record prints has to be the name of the polynomial configured, or a reader chases the wrong one.
    #[test]
    fn the_record_names_match_the_polynomials() {
        assert_eq!(CRCPoly::Crc32.as_str(), "crc32");
        assert_eq!(CRCPoly::Crc16.as_str(), "crc16");
    }
}
