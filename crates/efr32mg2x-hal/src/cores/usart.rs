//! The USART baud-rate divider, as arithmetic rather than as register writes.
//!
//! Why it is a crate of its own: the HAL computes this divider and cannot be compiled for the host (it contains ARM
//! `asm!`), so a defect here could only be read, never measured. The formula is plain integer arithmetic, so it lives
//! where a test can run it in microseconds.
//!
//! The defect it was written to close, from the 2026-11-17 review (H5): the HAL computed
//! `clkdiv = (256 * pclk / (oversample * baud)) - 256` in `u32`. At the 19 MHz the board actually runs, `256 * 19 000 000`
//! is 4 864 000 000, which does not fit in a `u32` (4 294 967 295): the multiplication wraps and the divider written to the
//! hardware is a number derived from a truncated product. Nothing reports it, and a wrong divider is a wrong baud rate.
//!
//! The formula itself is the vendor's, from the reference manual and `em_usart.c`: `CLKDIV = 256 * (fPCLK / (oversample *
//! baud) - 1)`, with `DIV` occupying the 20 bits at position 3 of the register -- which is why `MAX_DIV` here is 20 bits
//! wide, not 16.

/// `DIV` is 20 bits wide (`_USART_CLKDIV_DIV_MASK 0x7FFFF8`: bits 3..22 of the register).
pub const MAX_DIV: u32 = (1 << 20) - 1;

/// Why a divider could not be produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DividerError {
    /// A zero baud rate or oversample factor.
    Zero,
    /// The baud rate is higher than the clock can produce: the formula would need a negative divider.
    TooFast,
    /// The divider does not fit the 20-bit field.
    TooSlow,
}

/// The `CLKDIV` value for `pclk_hz` and `baud`, with the multiplication done in 64 bits.
///
/// Returns the value to write into the register's `DIV` field, or why it cannot be produced. `oversample` is 16 for
/// asynchronous mode on this family.
pub fn clkdiv(pclk_hz: u32, baud: u32, oversample: u32) -> Result<u32, DividerError> {
    if baud == 0 || oversample == 0 || pclk_hz == 0 {
        return Err(DividerError::Zero);
    }
    // 64-bit on purpose: 256 * 19 000 000 does not fit in a u32, and that is the defect this replaces.
    let numerator = 256u64 * pclk_hz as u64;
    let denominator = oversample as u64 * baud as u64;
    // `CLKDIV = 256 * (fPCLK / (oversample * baud) - 1)`: the 256 is subtracted, not the 1 inside it.
    let quotient = numerator / denominator;
    if quotient < 256 {
        return Err(DividerError::TooFast);
    }
    let div = quotient - 256;
    if div > MAX_DIV as u64 {
        return Err(DividerError::TooSlow);
    }
    Ok(div as u32)
}

/// The baud rate the hardware will actually produce for a divider, which is the number to report next to the request.
///
/// `baud = pclk / (oversample * (1 + div/256))`. The caller asked for a rate; this is the rate it got, and they are not
/// always the same number.
pub fn actual_baud(pclk_hz: u32, div: u32, oversample: u32) -> u32 {
    if oversample == 0 {
        return 0;
    }
    let denominator = 256u64 * oversample as u64 + oversample as u64 * div as u64;
    if denominator == 0 {
        return 0;
    }
    ((pclk_hz as u64 * 256) / denominator) as u32
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    /// The five clock rates this project has measured on the board, at the rate a debug console uses.
    #[test]
    fn dividers_at_every_measured_clock() {
        assert_eq!(clkdiv(19_000_000, 115_200, 16), Ok(2382));
        assert_eq!(clkdiv(38_000_000, 115_200, 16), Ok(5021));
        assert_eq!(clkdiv(56_000_000, 115_200, 16), Ok(7521));
        assert_eq!(clkdiv(80_000_000, 115_200, 16), Ok(10855));
        assert_eq!(clkdiv(19_000_000, 9_600, 16), Ok(31410));
    }

    /// The defect, reproduced: the multiplication that used to wrap.
    #[test]
    fn the_multiplication_does_not_wrap_at_19_mhz() {
        assert!(
            256u32.checked_mul(19_000_000).is_none(),
            "the old expression must overflow a u32, or this test is not measuring the defect"
        );
        let div = clkdiv(19_000_000, 115_200, 16).expect("a divider");
        // What the wrapping arithmetic produced, for the record: the same product modulo 2^32.
        let wrapped = (256u32.wrapping_mul(19_000_000) / (16 * 115_200)).wrapping_sub(256);
        assert_ne!(
            div, wrapped,
            "the 64-bit result must differ from the wrapped one, otherwise nothing was fixed"
        );
        std::println!("64-bit {div}, wrapped {wrapped}");
    }

    /// A rate the clock cannot produce is refused instead of wrapping into a plausible divider.
    #[test]
    fn too_fast_is_refused() {
        assert_eq!(
            clkdiv(19_000_000, 16_000_000, 16),
            Err(DividerError::TooFast)
        );
    }

    /// Zero inputs are refused rather than divided by.
    #[test]
    fn zero_inputs_are_refused() {
        assert_eq!(clkdiv(19_000_000, 0, 16), Err(DividerError::Zero));
        assert_eq!(clkdiv(19_000_000, 115_200, 0), Err(DividerError::Zero));
        assert_eq!(clkdiv(0, 115_200, 16), Err(DividerError::Zero));
    }

    /// The field is 20 bits: a value that does not fit is refused, not truncated.
    #[test]
    fn a_divider_beyond_the_field_is_refused() {
        // 19 MHz with 1 baud: 256 * 19e6 / 16 - 1, far above 20 bits.
        assert_eq!(clkdiv(19_000_000, 1, 16), Err(DividerError::TooSlow));
        assert!(MAX_DIV == 0xF_FFFF, "the field is 20 bits wide");
    }

    /// The rate the hardware produces is within the tolerance a UART needs, and it is the number to report.
    #[test]
    fn the_actual_rate_is_within_a_fraction_of_a_percent() {
        for pclk in [19_000_000u32, 38_000_000, 56_000_000, 80_000_000] {
            let div = clkdiv(pclk, 115_200, 16).unwrap();
            let actual = actual_baud(pclk, div, 16);
            let error_ppm = ((actual as i64 - 115_200).unsigned_abs() * 1_000_000) / 115_200;
            assert!(
                error_ppm < 10_000,
                "pclk {pclk}: {actual} baud is {error_ppm} ppm off, more than the 1 % a UART tolerates"
            );
        }
    }
}
