//! The clock sources the CMU can select and no driver owned: `FSRCO`, `HFRCOEM23` and `ULFRCO`.
//!
//! ## Why this exists
//!
//! `clock` configures the tree the core runs on, `pwm` and `adc` switch `EM01GRPACLK` to `HFRCOEM23` on the way past,
//! and `burtc`/`sysrtc` pick a low-frequency branch -- but nothing could say *what these oscillators measure on this
//! die*. The prose already carried three different answers for `HFRCOEM23` (18.5 MHz from a blink, 18.961 MHz from the
//! tick timer, "19.6 MHz" in a register note) and none of them was a claim the code could make. A frequency that is
//! only written in a comment is a number nobody can check.
//!
//! ## What is a register write and what is a datasheet number
//!
//! The selector writes here are the ones the part accepts: `EM01GRPACLK` has `HFRCODPLL`, `HFXO`, `FSRCO` and
//! `HFRCOEM23` (measured 2026-11-27 by counting each one with TIMER0), and `CAL.FREQRANGE` reads back the band the
//! oscillator was trimmed to. The limits below are **copied from the datasheet**, not measured, and each carries its
//! table so a reader can tell the two apart:
//!
//! - `FSRCO`: Table 4.28, 17.2 / 20 / 21.2 MHz.
//! - `ULFRCO`: Table 4.30, 0.944 / 1.0 / 1.095 kHz.
//! - `LFRCO`: Table 4.29, 32.768 kHz, normal mode +-3 %.
//! - `HFRCO` bands: Table 4.27 note 6, the `FREQRANGE` limits used for whichever band `CAL` reports.
//!
//! ## The one ordering that is not optional
//!
//! `CMU_CLKEN0.HFRCOEM23` is the bus gate: without it the first read of any `HFRCOEM23` register is a precise bus
//! fault, not a zero. `clock::start_hfxo` paid for the same lesson on `HFXO0` (`BFAR = 0x4001_000C`); this module sets
//! the gate before it touches the oscillator, and [`hfrcoem23_band`] waits for `STATUS.RDY` before it believes `CAL`.

use efr32mg2x_pac as pac;

/// `FSRCO` minimum, datasheet Rev 1.2 Table 4.28.
pub const FSRCO_MIN_HZ: u32 = 17_200_000;
/// `FSRCO` typical, datasheet Rev 1.2 Table 4.28.
pub const FSRCO_TYP_HZ: u32 = 20_000_000;
/// `FSRCO` maximum, datasheet Rev 1.2 Table 4.28.
pub const FSRCO_MAX_HZ: u32 = 21_200_000;

/// `ULFRCO` minimum, datasheet Rev 1.2 Table 4.30.
pub const ULFRCO_MIN_HZ: u32 = 944;
/// `ULFRCO` typical, datasheet Rev 1.2 Table 4.30.
pub const ULFRCO_TYP_HZ: u32 = 1_000;
/// `ULFRCO` maximum, datasheet Rev 1.2 Table 4.30.
pub const ULFRCO_MAX_HZ: u32 = 1_095;

/// `LFRCO` nominal, datasheet Rev 1.2 Table 4.29.
pub const LFRCO_TYP_HZ: u32 = 32_768;
/// `LFRCO` accuracy in normal mode, percent, datasheet Rev 1.2 Table 4.29.
pub const LFRCO_TOLERANCE_PER_CENT: u32 = 3;

/// A source `EM01GRPACLK` can be pointed at, as this part's SVD enumerates them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Em01Source {
    /// `HFRCODPLL`, the oscillator the core runs from at reset.
    HfrcoDpll,
    /// The 39 MHz crystal, once `clock::start_hfxo` has started it.
    Hfxo,
    /// The fast start-up RC oscillator, ~20 MHz, used to boot the flash.
    Fsrco,
    /// The energy-mode RC oscillator, the band `CAL.FREQRANGE` names.
    HfrcoEm23,
}

impl Em01Source {
    /// The value this source has in `EM01GRPACLKCTRL.CLKSEL`, which is also what reads back.
    #[must_use]
    pub const fn clksel(self) -> u8 {
        match self {
            Self::HfrcoDpll => 1,
            Self::Hfxo => 2,
            Self::Fsrco => 3,
            Self::HfrcoEm23 => 4,
        }
    }
}

/// Why a clock-source operation was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// `HFRCOEM23` never reported `STATUS.RDY`, so its `CAL` value would be a guess.
    HfrcoEm23NotReady,
    /// `CLKSEL` was written and read back a different value: the mux did not take the selection.
    NotSelected {
        /// The value written.
        wrote: u8,
        /// The value the register returned.
        read: u8,
    },
    /// `CAL.FREQRANGE` is outside the sixteen bands the datasheet gives limits for.
    BandOutOfTable {
        /// The value the field returned.
        range: u8,
    },
}

/// `EM01GRPACLKCTRL.CLKSEL` as it stands, without touching anything.
#[must_use]
pub fn em01_readback() -> u8 {
    // SAFETY: one CMU register, read only. `Board` owns the CMU for the life of the program and a read changes nothing.
    let cmu = unsafe { pac::CmuS::steal() };
    cmu.em01grpaclkctrl().read().clksel().bits()
}

/// Point `EM01GRPACLK` -- the clock group TIMER0-4 and USART0 run from -- at `source`, and check it took.
///
/// The read-back is the point. A mux that accepts the write and keeps the old source is indistinguishable from a
/// source that is slow or stopped, and both look like a bug in whatever is being timed.
///
/// # Errors
///
/// [`Error::NotSelected`] when `CLKSEL` reads back as something other than what was written.
pub fn em01_select(source: Em01Source) -> Result<u8, Error> {
    // SAFETY: `EM01GRPACLKCTRL` is a CMU register whose `CLKSEL` this module owns; `Board` has handed the CMU out and
    // this is the same write `board::pace_microphone` makes. Only `CLKSEL` is replaced.
    let cmu = unsafe { pac::CmuS::steal() };
    cmu.em01grpaclkctrl().modify(|_, w| match source {
        Em01Source::HfrcoDpll => w.clksel().hfrcodpll(),
        Em01Source::Hfxo => w.clksel().hfxo(),
        Em01Source::Fsrco => w.clksel().fsrco(),
        Em01Source::HfrcoEm23 => w.clksel().hfrcoem23(),
    });
    let read = em01_readback();
    if read == source.clksel() {
        Ok(read)
    } else {
        Err(Error::NotSelected {
            wrote: source.clksel(),
            read,
        })
    }
}

/// The datasheet band limits of an `HFRCO` `FREQRANGE`, in hertz: Table 4.27 note 6, Rev 1.2.
///
/// These are the limits each band can reach over the operating range, so they are the band a die trimmed to that
/// `FREQRANGE` must sit inside. They are not the +-3 % accuracy of the production-calibrated frequencies (also Table
/// 4.27): a band is wider than that on purpose, and the difference matters -- see
/// `docs/04-reference/CMU-MISSING-CLOCKS-2026-11-27.md`.
#[must_use]
pub const fn hfrco_band_limits(range: u8) -> Option<(u32, u32)> {
    let limits = match range {
        0 => (3_710_000, 5_240_000),
        1 => (4_390_000, 6_260_000),
        2 => (5_250_000, 7_550_000),
        3 => (6_220_000, 9_010_000),
        4 => (7_880_000, 11_600_000),
        5 => (9_900_000, 14_600_000),
        6 => (11_500_000, 17_000_000),
        7 => (14_100_000, 20_900_000),
        8 => (16_400_000, 24_700_000),
        9 => (19_800_000, 30_400_000),
        10 => (22_700_000, 34_900_000),
        11 => (28_600_000, 44_400_000),
        12 => (33_000_000, 51_000_000),
        13 => (42_200_000, 64_600_000),
        14 => (48_800_000, 74_800_000),
        15 => (57_600_000, 87_400_000),
        _ => return None,
    };
    Some(limits)
}

/// Force `HFRCOEM23` on, wait for `STATUS.RDY`, and return the band `CAL.FREQRANGE` reports.
///
/// The gate comes first: without `CLKEN0.HFRCOEM23` the first access to any register of this block is a precise bus
/// fault. `FORCEEN` comes next because the oscillator is demand-driven -- a selector pointed at an oscillator that was
/// never forced on gives a timer that does not count, with no error anywhere.
///
/// # Errors
///
/// [`Error::HfrcoEm23NotReady`] if `RDY` never comes up, [`Error::BandOutOfTable`] if `CAL.FREQRANGE` names a band the
/// datasheet has no limits for (which would make any check against a limit meaningless).
pub fn hfrcoem23_band() -> Result<u8, Error> {
    // SAFETY: the CMU gate and the oscillator block are the ones `Board` owns; this is the same sequence
    // `board::pace_microphone` runs, with the band read added.
    let cmu = unsafe { pac::CmuS::steal() };
    let osc = unsafe { pac::Hfrcoem23S::steal() };
    cmu.clken0().modify(|_r, w| w.hfrcoem23().set_bit());
    osc.ctrl().modify(|_r, w| w.forceen().set_bit());
    let mut spin = 0u32;
    while !osc.status().read().rdy().bit_is_set() {
        spin += 1;
        if spin > 1_000_000 {
            return Err(Error::HfrcoEm23NotReady);
        }
    }
    let range = osc.cal().read().freqrange().bits();
    if hfrco_band_limits(range).is_none() {
        return Err(Error::BandOutOfTable { range });
    }
    Ok(range)
}

/// `HFRCOEM23.STATUS` as it stands, for a record that has to tell "never forced on" from "forced and not ready".
#[must_use]
pub fn hfrcoem23_status() -> u32 {
    // SAFETY: one read of a block whose bus gate `hfrcoem23_band` or a `Board` call has already opened. `HFRCOEM23`'s
    // registers are not owned by any driver in this crate.
    let osc = unsafe { pac::Hfrcoem23S::steal() };
    osc.status().read().bits()
}

/// `HFRCOEM23.CAL` as it stands: the trim the oscillator is actually running on.
///
/// The reset value is a band and a mid-scale tuning (`FREQRANGE = 8`, `TUNING = 0x7F` in this part's RM), **not** a
/// production-calibrated pair: the reference manual is explicit that using a factory-calibrated frequency means reading
/// `DEVINFO` and writing it into `CAL`, and nothing in this crate does that. A record that carries this word says which
/// trim the frequency beside it belongs to.
#[must_use]
pub fn hfrcoem23_cal() -> u32 {
    // SAFETY: one read of the block's calibration register, which is only written by a trim this crate never applies.
    let osc = unsafe { pac::Hfrcoem23S::steal() };
    osc.cal().read().bits()
}
