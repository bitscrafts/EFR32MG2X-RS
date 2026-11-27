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

/// The EUSART's fractional divider step: the field's low five bits are the fraction.
///
/// ## Why the EUSART needs its own arithmetic, measured
///
/// The USART's `CLKDIV` splits the field into eight fraction bits (`1 + DIV/256`). The EUSART's does not: feeding the
/// USART's divider into an EUSART0 async port produced frames at **15 650 baud** where 115 200 was asked for
/// (`hal_eusart_loopback`, 2026-11-27: 80 bit times in 167 SYSRTC ticks, with the same instrument calibrated at 3290
/// ticks per 100 ms). At 19 MHz the EUSART's rate is `f / (OVS * (1 + DIV/32))` — the same shape, five fraction bits
/// instead of eight — and with that divisor the same request lands on 115 152.
pub const EUSART_FRACTION: u32 = 32;

/// The `CLKDIV` value an **EUSART** needs for `baud`, with the multiplication done in 64 bits.
///
/// `DIV = 32 * (f / (OVS * baud) - 1)`, the EUSART's five-fraction-bit form of the USART formula. Same refusals as
/// [`clkdiv`]: a rate the clock cannot produce is an error, never a wrapped number.
///
/// ```
/// use efr32mg2x_usart_core::{eusart_actual_baud, eusart_clkdiv};
///
/// let div = eusart_clkdiv(19_000_000, 115_200, 16).expect("reachable");
/// assert_eq!(div, 297);
/// assert!((eusart_actual_baud(19_000_000, div, 16) as i64 - 115_200).abs() < 600);
/// ```
pub fn eusart_clkdiv(clock_hz: u32, baud: u32, oversample: u32) -> Result<u32, DividerError> {
    if baud == 0 || oversample == 0 || clock_hz == 0 {
        return Err(DividerError::Zero);
    }
    // 64-bit for the same reason as `clkdiv`: 32 * 19 000 000 is 608 000 000 and would already be uncomfortable in a
    // `u32` once the clock rises.
    let numerator = EUSART_FRACTION as u64 * clock_hz as u64;
    let denominator = oversample as u64 * baud as u64;
    let quotient = numerator / denominator;
    if quotient <= EUSART_FRACTION as u64 {
        return Err(DividerError::TooFast);
    }
    let div = quotient - EUSART_FRACTION as u64;
    if div > MAX_DIV as u64 {
        return Err(DividerError::TooSlow);
    }
    Ok(div as u32)
}

/// The rate an EUSART divider actually produces: `f / (OVS * (1 + DIV/32))`.
#[must_use]
pub fn eusart_actual_baud(clock_hz: u32, div: u32, oversample: u32) -> u32 {
    let denominator = 32u64 * oversample as u64 + oversample as u64 * div as u64;
    if denominator == 0 {
        return 0;
    }
    ((clock_hz as u64 * 32) / denominator) as u32
}

/// `IF.RXOF` — the receiver ran out of room and a byte was lost.
pub const IF_RXOF: u32 = 1 << 4;

/// `IF.PERR` — the byte arrived with a parity error.
pub const IF_PERR: u32 = 1 << 8;

/// `IF.FERR` — the byte arrived with a framing error: there was no stop bit where one belonged.
pub const IF_FERR: u32 = 1 << 9;

/// Why a byte at the front of the receiver is not data.
///
/// This exists because of a measurement, not a theory: with nothing driving the RX pad, `hal_usart_link` reported
/// `echo n=271` — 271 bytes that were never sent. A floating line produces start bits out of noise, and every one of them
/// sets `RXDATAV`, so a driver that trusts `RXDATAV` alone reads noise as a message and reports a working link.
///
/// This MCU has no per-byte error bit in `STATUS`; the only evidence a frame was broken is in `IF`, which is why the
/// driver has to look there before believing the byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RxProblem {
    /// `IF.RXOF`: the receiver overran -- at least one byte is gone, and this one may not be the one that was read.
    Overrun,
    /// `IF.FERR`: the frame had no stop bit. On a floating line this is the usual verdict.
    Framing,
    /// `IF.PERR`: the parity bit did not match.
    Parity,
}

impl RxProblem {
    /// The flag as it appears in a record.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            RxProblem::Overrun => "rxof",
            RxProblem::Framing => "ferr",
            RxProblem::Parity => "perr",
        }
    }
}

/// Whether the byte waiting in the receiver can be believed, given the `IF` register.
///
/// Returns `None` when the byte is data, and the reason when it is not. The order is deliberate: an overrun is reported
/// ahead of a framing error, because an overrun means bytes are missing and the count of what arrived is wrong, which
/// no later reading can repair.
#[must_use]
pub fn rx_problem(if_bits: u32) -> Option<RxProblem> {
    if if_bits & IF_RXOF != 0 {
        Some(RxProblem::Overrun)
    } else if if_bits & IF_FERR != 0 {
        Some(RxProblem::Framing)
    } else if if_bits & IF_PERR != 0 {
        Some(RxProblem::Parity)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use super::{rx_problem, RxProblem, IF_FERR, IF_PERR, IF_RXOF};

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

    /// The defect, as a test: a frame with no stop bit is not a byte of data. Before this existed, `read_byte` returned
    /// `Some(0xFF)` for it, and an instrument reported 271 bytes that were never sent.
    #[test]
    fn a_broken_frame_is_not_data() {
        assert_eq!(rx_problem(IF_FERR), Some(RxProblem::Framing));
        assert_eq!(rx_problem(IF_PERR), Some(RxProblem::Parity));
        assert_eq!(rx_problem(IF_RXOF), Some(RxProblem::Overrun));
    }

    /// The control that keeps the test above honest: a clean flag word means the byte *is* used. A predicate that
    /// refused everything would also pass "a broken frame is not data".
    #[test]
    fn a_clean_frame_is_data() {
        assert_eq!(rx_problem(0), None);
        // Other flags in the same register must not turn a good byte into a bad one: `TXC` (6) and `TXBL` (7) are set
        // for most of the driver's life.
        assert_eq!(rx_problem(1 << 6 | 1 << 7), None);
    }

    /// An overrun is reported ahead of a framing error: it means bytes are missing, and a count taken after it is wrong
    /// in a way a later reading cannot repair.
    #[test]
    fn an_overrun_outranks_the_other_reasons() {
        assert_eq!(
            rx_problem(IF_RXOF | IF_FERR | IF_PERR),
            Some(RxProblem::Overrun)
        );
        assert_eq!(rx_problem(IF_FERR | IF_PERR), Some(RxProblem::Framing));
    }

    /// The flag names are what a record prints, and a record with the wrong name sends a reader after the wrong cause.
    #[test]
    fn every_reason_has_its_register_name() {
        assert_eq!(RxProblem::Overrun.as_str(), "rxof");
        assert_eq!(RxProblem::Framing.as_str(), "ferr");
        assert_eq!(RxProblem::Parity.as_str(), "perr");
    }
}
