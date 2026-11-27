//! `ACMP0`: the analogue comparator, used here as a ladder that measures a voltage instead of a pin that compares one.
//!
//! # What this driver encodes, and which line of it was paid for with a fault
//!
//! 1. **The bus gate is `CMU_CLKEN1.ACMP0`, not `CLKEN0`.** Unlike `IADC0`, `LDMA` or `TIMER0`, the comparator's gate
//!    lives in the second bank. With the gate clear the block does not answer at all: the first register read faults
//!    precisely at `ACMP0_S + 0x04` (`CFSR = 0x0000_8200`, `BFAR = 0x4900_8004`, measured 2026-11-27). Nothing in the
//!    block's own registers says so, and a driver copied from another peripheral compiles and faults.
//! 2. **`INPUTCTRL` is not writable while `EN = 0`.** The same run measured a precise fault at `ACMP0_S + 0x14`
//!    (`BFAR = 0x4900_8014`) for exactly that write, so [`Acmp0::new`] enables the block first and
//!    [`Acmp0::compare`] is only reachable afterwards. `CFG` and `CTRL` are writable disabled -- that is what the enable
//!    sequence configures.
//! 3. **`SYNCBUSY.INPUTCTRL` is the handshake.** The input mux is in the analogue domain; the write is accepted and
//!    takes effect over several cycles, and `SYNCBUSY` is the only thing that says when. Every configuration here waits
//!    on it, bounded.
//! 4. **`VREFDIV` is a ladder tap, not a divisor of one.** `VREFOUT = VREFIN * VREFDIV / 63`, with `63` the pass-through
//!    tap; `0` is zero volts. The arithmetic is in [`ladder_mv`], which is host-tested, so a sweeping instrument does
//!    not have to trust the driver's own mental arithmetic.
//! 5. **The output follows `CTRL.NOTRDYVAL` while the block is not ready.** Measured with the block disabled: with
//!    `NOTRDYVAL = 0` the status bit reads 0, with `NOTRDYVAL = 1` it reads 1, with the inputs left where they were. So
//!    a disabled comparator is not "a comparator that says low", and an instrument that never checks
//!    [`Acmp0::is_ready`] cannot tell a measurement from a configured default.
//!
//! # What this driver does not do
//!
//! The interrupt path (`IF`/`IEN`, rise/fall), the capacitive-sense resistor bank, hibernate mode, the GPIO output
//! route, `ACMP1`, and any input on a pin are not exposed: no instrument for them exists in this crate, and a driver
//! without one is a claim. The measured record and what each run rules out are in
//! `docs/04-reference/ACMP-LADDER-2026-11-27.md`.

use efr32mg2x_pac as pac;

/// The ladder arithmetic, re-exported so an instrument does not reach into the private logic layer.
pub use crate::cores::acmp::{ladder_mv, overlap, window_mv, LADDER_FULL};

/// How many polls a synchronisation wait will make before giving up.
const SYNC_POLLS: u32 = 2_000_000;

/// Why an `ACMP0` operation was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// `EN` was written and `STATUS.ACMPRDY` never came up.
    NeverReady,
    /// `EN` was cleared and `EN.DISABLING` never went away.
    NeverDisabled,
    /// `SWRST` was written and `SWRST.RESETTING` never went away.
    NeverReset,
    /// The input mux did not take the requested source: the register reads back different from what was written.
    InputsRefused {
        /// The positive input asked for.
        pos: u8,
        /// The negative input asked for.
        neg: u8,
        /// The ladder tap asked for.
        div: u8,
        /// What `INPUTCTRL` returned instead.
        read: u32,
    },
    /// The requested port/pin is not an input this part has (ports A-D, pins 0-15).
    NoSuchPin {
        /// Port index, 0 = A.
        port: u8,
        /// Pin index.
        pin: u8,
    },
}

/// One side of the comparator's input mux.
///
/// The numbers are the PAC's own field values (`Possel`/`Negsel`), not literals chosen here: `0` is ground, `16..=21`
/// are the reference ladder taps, `32..=35` are the two internal sense nodes divided by four, and `128..=191` are the
/// package pins in port-major order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Ground.
    Vss,
    /// `AVDD` through the ladder.
    DividedAvdd,
    /// `AVDD` through the ladder, low-power bias.
    DividedAvddLp,
    /// The 1.25 V reference through the ladder.
    Divided1v25,
    /// The 1.25 V reference through the ladder, low-power bias.
    Divided1v25Lp,
    /// The 2.5 V reference through the ladder.
    Divided2v5,
    /// The 2.5 V reference through the ladder, low-power bias.
    Divided2v5Lp,
    /// Internal sense node `VSENSE0`, divided by four.
    Sense0Div4,
    /// Internal sense node `VSENSE0` divided by four, low-power bias.
    Sense0Div4Lp,
    /// Internal sense node `VSENSE1`, divided by four.
    Sense1Div4,
    /// Internal sense node `VSENSE1` divided by four, low-power bias.
    Sense1Div4Lp,
    /// A package pin, `0x80 + port * 16 + pin`.
    Pin {
        /// Port index, 0 = A.
        port: u8,
        /// Pin index.
        pin: u8,
    },
}

impl Source {
    /// The `POSSEL`/`NEGSEL` field value, or `None` for a pin this part does not have.
    #[must_use]
    pub fn code(self) -> Option<u8> {
        use pac::acmp0_s::inputctrl::Possel;
        Some(match self {
            Source::Vss => Possel::Vss as u8,
            Source::DividedAvdd => Possel::Vrefdivavdd as u8,
            Source::DividedAvddLp => Possel::Vrefdivavddlp as u8,
            Source::Divided1v25 => Possel::Vrefdiv1v25 as u8,
            Source::Divided1v25Lp => Possel::Vrefdiv1v25lp as u8,
            Source::Divided2v5 => Possel::Vrefdiv2v5 as u8,
            Source::Divided2v5Lp => Possel::Vrefdiv2v5lp as u8,
            Source::Sense0Div4 => Possel::Vsense01div4 as u8,
            Source::Sense0Div4Lp => Possel::Vsense01div4lp as u8,
            Source::Sense1Div4 => Possel::Vsense11div4 as u8,
            Source::Sense1Div4Lp => Possel::Vsense11div4lp as u8,
            Source::Pin { port, pin } => {
                if port > 3 || pin > 15 {
                    return None;
                }
                0x80 + port * 16 + pin
            }
        })
    }
}

/// The comparator, owned by one caller.
pub struct Acmp0 {
    regs: pac::Acmp0S,
}

impl Acmp0 {
    /// Take `ACMP0`, gate its bus clock, reset it, and leave it enabled and configured with the reset bias.
    ///
    /// The gate is set here rather than left to the caller because it is the difference between a block that answers
    /// and a bus fault on the first read; the module documentation carries the measurement.
    pub fn new(regs: pac::Acmp0S) -> Result<Self, Error> {
        // SAFETY: `CMU_CLKEN1` is one word of a block the board owns; setting one gate bit changes nothing else, and
        // the alternative -- a caller that forgets it -- is a precise bus fault on the first ACMP0 read.
        let cmu = unsafe { pac::CmuS::steal() };
        cmu.clken1().modify(|_r, w| w.acmp0().set_bit());

        let acmp = Acmp0 { regs };

        // A reset of a block that was already running is the state that leaves `EN.DISABLING` behind (the BURTC
        // measurement, 2026-11-27): drop `EN` and wait it out before touching the configuration.
        acmp.regs.en().write(|w| w.en().clear_bit());
        let mut spins = 0u32;
        while acmp.regs.en().read().disabling().bit_is_set() {
            spins += 1;
            if spins > SYNC_POLLS {
                return Err(Error::NeverDisabled);
            }
        }

        acmp.regs.swrst().write(|w| w.swrst().set_bit());
        let mut spins = 0u32;
        while acmp.regs.swrst().read().resetting().bit_is_set() {
            spins += 1;
            if spins > SYNC_POLLS {
                return Err(Error::NeverReset);
            }
        }

        // `CFG` and `CTRL` are writable disabled; `INPUTCTRL` is not (measured, `+0x14`).
        //
        // SAFETY: `CFG`'s reset word is `0x0000_0004` (`BIAS = 4`, everything else 0, `_ACMP_CFG_RESETVALUE` in the
        // vendor header), so this write restores exactly the fields the block comes up with and sets no undefined bit.
        acmp.regs.cfg().write(|w| unsafe { w.bits(0x0000_0004) });
        // SAFETY: `CTRL = 0` is "not inverted, not-ready value low, GPIO output not inverted" -- the reset word, written
        // field by field by a zero, so no defined field is left at a value nobody chose.
        acmp.regs.ctrl().write(|w| unsafe { w.bits(0) });

        acmp.regs.en().write(|w| w.en().set_bit());
        let mut spins = 0u32;
        while !acmp.regs.status().read().acmprdy().bit_is_set() {
            spins += 1;
            if spins > SYNC_POLLS {
                return Err(Error::NeverReady);
            }
        }
        Ok(acmp)
    }

    /// Point the comparator at two sources and one ladder tap.
    ///
    /// `div` is the `VREFDIV` tap, `0..=63`, where `63` passes the reference through undivided; it applies to whichever
    /// side selects a ladder tap. The write is checked: `INPUTCTRL` is read back and a mux that did not take the request
    /// is an error, not a measurement taken on something else.
    pub fn compare(&mut self, pos: Source, neg: Source, div: u8) -> Result<(), Error> {
        let pos_code = pos.code().ok_or(Error::NoSuchPin {
            port: match pos {
                Source::Pin { port, .. } => port,
                _ => 0,
            },
            pin: match pos {
                Source::Pin { pin, .. } => pin,
                _ => 0,
            },
        })?;
        let neg_code = neg.code().ok_or(Error::NoSuchPin {
            port: match neg {
                Source::Pin { port, .. } => port,
                _ => 0,
            },
            pin: match neg {
                Source::Pin { pin, .. } => pin,
                _ => 0,
            },
        })?;

        let div = if div > 63 { 63 } else { div };
        let bits = u32::from(pos_code) | (u32::from(neg_code) << 8) | (u32::from(div) << 16);
        // SAFETY: the word is the three fields `INPUTCTRL` defines, each already range-checked here -- `pos_code` and
        // `neg_code` come from the PAC's own `Possel` values or the pin formula, and `div` is clamped to six bits -- so
        // the write cannot set a bit outside a defined field. The register is reached through the claim table, so no
        // other owner is configuring the same mux.
        self.regs.inputctrl().write(|w| unsafe { w.bits(bits) });
        self.wait_sync()?;

        let read = self.regs.inputctrl().read();
        let back_pos = read.possel().bits();
        let back_neg = read.negsel().bits();
        let back_div = read.vrefdiv().bits();
        if back_pos != pos_code || back_neg != neg_code || back_div != div {
            return Err(Error::InputsRefused {
                pos: pos_code,
                neg: neg_code,
                div,
                read: read.bits(),
            });
        }
        Ok(())
    }

    /// What the comparator currently says: `true` when the positive input is above the negative one.
    ///
    /// Meaningful only while [`Acmp0::is_ready`]; a comparison taken before the block is ready is `CTRL.NOTRDYVAL`.
    #[must_use]
    pub fn output(&self) -> bool {
        self.regs.status().read().acmpout().bit_is_set()
    }

    /// `STATUS.ACMPRDY`: the block has settled and its output is a comparison rather than the not-ready value.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.regs.status().read().acmprdy().bit_is_set()
    }

    /// `STATUS`: output, ready, input conflict and port allocation error, read as one word.
    #[must_use]
    pub fn status(&self) -> u32 {
        self.regs.status().read().bits()
    }

    /// `IF`: rise, fall, ready, input-conflict and port-allocation flags, read as one word.
    #[must_use]
    pub fn flags(&self) -> u32 {
        self.regs.if_().read().bits()
    }

    /// What the output reads while the block is not ready.
    pub fn set_not_ready_value(&mut self, high: bool) {
        if high {
            self.regs.ctrl().modify(|_r, w| w.notrdyval().high());
        } else {
            self.regs.ctrl().modify(|_r, w| w.notrdyval().low());
        }
    }

    /// The inputs and ladder tap as they read back, `(possel, negsel, vrefdiv)`.
    #[must_use]
    pub fn inputs(&self) -> (u8, u8, u8) {
        let read = self.regs.inputctrl().read();
        (
            read.possel().bits(),
            read.negsel().bits(),
            read.vrefdiv().bits(),
        )
    }

    /// Drop `EN` and wait for `EN.DISABLING`. The output then reads the not-ready value.
    pub fn disable(&mut self) -> Result<(), Error> {
        self.regs.en().write(|w| w.en().clear_bit());
        let mut spins = 0u32;
        while self.regs.en().read().disabling().bit_is_set() {
            spins += 1;
            if spins > SYNC_POLLS {
                return Err(Error::NeverDisabled);
            }
        }
        Ok(())
    }

    /// Raise `EN` and wait for `STATUS.ACMPRDY`.
    pub fn enable(&mut self) -> Result<(), Error> {
        self.regs.en().write(|w| w.en().set_bit());
        let mut spins = 0u32;
        while !self.regs.status().read().acmprdy().bit_is_set() {
            spins += 1;
            if spins > SYNC_POLLS {
                return Err(Error::NeverReady);
            }
        }
        Ok(())
    }

    /// Wait for the input mux to accept the last write.
    fn wait_sync(&self) -> Result<(), Error> {
        let mut spins = 0u32;
        while self.regs.syncbusy().read().inputctrl().bit_is_set() {
            spins += 1;
            if spins > SYNC_POLLS {
                // The only synchronisation this driver waits on is the input mux's; a stuck bit is a mux that never
                // took the write, which is the same defect `InputsRefused` reports.
                return Err(Error::InputsRefused {
                    pos: 0,
                    neg: 0,
                    div: 0,
                    read: self.regs.inputctrl().read().bits(),
                });
            }
        }
        Ok(())
    }
}
