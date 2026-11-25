//! The general-purpose CRC accelerator, driven so that its answer can be compared with an independent one.
//!
//! ## What this is for
//!
//! A CRC is the cheapest way for firmware to notice that an image, a configuration blob or an over-the-air frame arrived
//! corrupted, and this device has an accelerator for it: `GPCRC`.
//!
//! ## Which CRC-32 this actually is — measured, not read
//!
//! The reference manual describes the accelerator as MSB-first with polynomial `0x04C11DB7`, and this module's first draft
//! said so. The part disagreed: `hal_gpcrc` read `0x340BC6D9` for `b"123456789"` with seed `0xFFFFFFFF`, and
//! `!0x340BC6D9` is exactly `0xCBF43926` — the published check value of the **reflected** CRC-32, the one `zlib`, PNG and
//! Ethernet use. So the engine computes the reflected convention and stops one step short: the final exclusive-or is the
//! caller's.
//!
//! Recorded in the README as `PASS gpcrc: poly=crc32 raw=340bc6d9 zlib=cbf43926 want=cbf43926 diff=1 seed=1`.
//!
//! ## The order the part expects
//!
//! Measured 2026-11-25 on the XIAO MG24 (`hal_gpcrc`, record in the README): enable, then the polynomial and byte mode,
//! then the seed through `INIT` followed by `CMD.INIT` — the write to `INIT` alone does not load the engine, and a CRC
//! seeded with the previous seed is a CRC of the wrong message. Bytes then go to `INPUTDATABYTE`, one per call, and the
//! result is read from `DATA`.
//!
//! ## What it does not do
//!
//! No DMA, no interrupt, no 16-bit polynomial. Those are in the PAC and in the reference manual; adding them here without
//! an instrument that measures them would be the kind of code this crate's README calls "compiled and unmeasured".

use efr32mg2x_pac as pac;

use crate::cores::gpcrc::seed_matters;
pub use crate::cores::gpcrc::CRCPoly;

/// The CRC accelerator.
pub struct Gpcrc {
    regs: pac::GpcrcNs,
    poly: CRCPoly,
}

impl Gpcrc {
    /// Take the accelerator and configure it for one polynomial.
    ///
    /// # Safety
    ///
    /// The caller must own `GPCRC` for the life of this value. `Board::gpcrc` is the only caller that may exist: two
    /// values writing `INPUTDATABYTE` interleave into one nonsense CRC.
    pub(crate) unsafe fn new(regs: pac::GpcrcNs, poly: CRCPoly) -> Self {
        // Enable first: a configuration written to a disabled accelerator is not latched, which is exactly the kind of
        // silent no-op this crate has been bitten by before.
        regs.en().write(|w| w.en().set_bit());

        regs.ctrl().modify(|_, w| {
            match poly {
                CRCPoly::Crc32 => w.polysel().crc32(),
                CRCPoly::Crc16 => w.polysel().crc16(),
            };
            // Byte mode: the input register takes one byte per write, which is what `update` assumes.
            w.bytemode().set_bit();
            // Normal bit and byte order -- the MSB-first convention, spelled out rather than left at reset.
            w.bitreverse().normal();
            w.bytereverse().normal();
            // Auto-init off: the seed is set explicitly below, so a caller cannot inherit the last message's state.
            w.autoinit().clear_bit();
            w
        });

        Self { regs, poly }
    }

    /// Which polynomial this accelerator is configured for.
    #[must_use]
    pub fn poly(&self) -> CRCPoly {
        self.poly
    }

    /// Load a seed into the engine.
    ///
    /// Both writes are required: `INIT` holds the value and `CMD.INIT` moves it into the CRC. Measured: with only the
    /// first, the result is the previous message's CRC continued, which is a plausible-looking number computed over the
    /// wrong bytes.
    pub fn reset(&mut self, seed: u32) {
        // SAFETY: `INIT` is a full-width seed register; every 32-bit value is a valid seed, so there is no invalid
        // pattern to exclude and no other invariant for the caller to hold.
        self.regs.init().write(|w| unsafe { w.init().bits(seed) });
        self.regs.cmd().write(|w| w.init().set_bit());
    }

    /// Feed bytes through the engine.
    pub fn update(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            // SAFETY: `INPUTDATABYTE` is eight bits wide and `byte` is a `u8`, so the value cannot exceed the field.
            self.regs
                .inputdatabyte()
                .write(|w| unsafe { w.inputdatabyte().bits(byte) });
        }
    }

    /// The CRC of everything fed since the last [`Gpcrc::reset`].
    #[must_use]
    pub fn result(&self) -> u32 {
        self.regs.data().read().data().bits()
    }

    /// Reset, feed and read, which is what every caller actually wants.
    pub fn crc(&mut self, seed: u32, bytes: &[u8]) -> u32 {
        self.reset(seed);
        self.update(bytes);
        self.result()
    }

    /// Whether a seed distinction this driver relies on actually reaches the hardware.
    ///
    /// Kept as a method rather than a comment because the first question anyone asks of a CRC driver is "does the seed
    /// work?", and the answer has to come from the part.
    #[must_use]
    pub fn seed_changes_the_result(&mut self) -> bool {
        seed_matters(|seed, data| self.crc(seed, data))
    }
}

impl Drop for Gpcrc {
    fn drop(&mut self) {
        self.regs.en().write(|w| w.en().clear_bit());
    }
}
