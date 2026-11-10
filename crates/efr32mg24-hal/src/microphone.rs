//! Microphone: the analog MEMS mic on the XIAO MG24 **Sense**, sampled by IADC0.
//!
//! ## Hardware facts, from Seeed's own MG24 backend
//!
//! `Seeed_Arduino_Mic/src/hardware/mg24_adc.{h,cpp}` is unambiguous: the mic is
//! **not** a PDM device on this module. It is an analog microphone on **PC9**,
//! powered through **PC8**, sampled by the **IADC** with `IADCCLK` from an
//! internal oscillator, 1.2 V reference and 12-bit right-aligned results.
//!
//! ## What the IADC demands (measured, each one cost a flash)
//!
//! 1. `CTRL.TIMEBASE` must be non-zero — `ceil(fsrc / 1 MHz) - 1`. With 0 the
//!    converter produces nothing at all and every status bit stays clear.
//! 2. Configuration writes are dropped while `STATUS.SYNCBUSY` or
//!    `EN.DISABLING` is set. `CTRL` can land while `SINGLE` silently does not.
//! 3. `SINGLE` has its own write handshake: wait for `STATUS.SINGLEWRITEPENDING`.
//! 4. A converter left with `EN.DISABLING` set **survives a core reset** and
//!    faults on the next `SINGLE` write (`BFAR = 0x49004098`). Only a power
//!    cycle clears it, so every entry point here refuses to write in that state
//!    and reports it instead of faulting.
//!
//! ## Why single conversions
//!
//! The vendor driver scans into a FIFO and lets the LDMA drain it. A scan needs
//! a trigger source configured; with the scan path and no trigger the converter
//! sat with `CONVERTING=1` and `SCANQUEUEPENDING=1` forever. One channel polled
//! with `SINGLESTART` needs no trigger, which is the right starting point for a
//! level meter. The DMA path belongs later, next to the streaming work.

use efr32mg24_pac as pac;

/// Result of one measurement window.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Level {
    /// Root-mean-square amplitude around mid-scale, in ADC counts.
    pub rms: u32,
    /// Largest deviation from mid-scale seen in the window.
    pub peak: u32,
    /// Samples actually collected — zero means the converter never delivered.
    pub samples: u32,
}

/// Errors reported when the converter cannot be configured.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MicError {
    /// The converter is stuck in `EN.DISABLING` (survives a core reset; needs a
    /// power cycle).
    StuckDisabling,
    /// Conversions never stopped, so registers cannot be safely rewritten.
    ConversionsWontStop,
    /// A configuration write did not publish (`*WRITEPENDING` never cleared).
    WriteNotPublished,
}

/// A configured single-channel microphone capture.
pub struct Microphone<'a> {
    adc: &'a pac::iadc0_s::RegisterBlock,
}

fn wait_bits_clear(mut read: impl FnMut() -> bool) -> bool {
    let mut spin = 0u32;
    while read() {
        spin += 1;
        if spin > 2_000_000 {
            return false;
        }
    }
    true
}

impl<'a> Microphone<'a> {
    /// Configure IADC0 for one channel on PC9.
    ///
    /// The caller must have powered the microphone (PC8) and enabled the IADC0
    /// bus clock; both are board-level concerns handled by
    /// `board::SensorPower` and `board::Board`.
    pub fn new(adc: &'a pac::iadc0_s::RegisterBlock) -> Result<Self, MicError> {
        // Refuse to touch a converter that cannot leave DISABLING: this is the
        // state that faults on write and survives a core reset.
        if adc.en().read().disabling().bit_is_set() {
            return Err(MicError::StuckDisabling);
        }
        adc.en().write(|w| w.en().set_bit());

        // Stop conversions before rewriting anything.
        adc.cmd().write(|w| {
            w.singlestop().set_bit().scanstop().set_bit();
            w.timerdis().set_bit()
        });
        let stopped = wait_bits_clear(|| {
            adc.status().read().converting().bit_is_set()
                || adc.status().read().singlequeuepending().bit_is_set()
                || adc.status().read().scanqueuepending().bit_is_set()
                || adc.status().read().timeractive().bit_is_set()
        });
        if !stopped {
            return Err(MicError::ConversionsWontStop);
        }

        // Reset the two handshake registers and drain the FIFO while enabled,
        // exactly as the vendor's IADC_reset() does. Without this the first
        // samples come from whatever an earlier run left in the FIFO.
        adc.maskreq().write(|w| unsafe { w.bits(0) });
        adc.single().write(|w| unsafe { w.bits(0) });
        if !wait_bits_clear(|| {
            adc.status().read().maskreqwritepending().bit_is_set()
                || adc.status().read().singlewritepending().bit_is_set()
        }) {
            return Err(MicError::WriteNotPublished);
        }
        let mut drained = 0u32;
        while adc.status().read().singlefifodv().bit_is_set()
            || adc.singlefifostat().read().bits() > 0
        {
            let _ = adc.singlefifodata().read().bits();
            drained += 1;
            if drained > 4096 {
                break;
            }
        }

        // Disable and wait for the synchroniser to settle, then configure.
        adc.en().write(|w| w.en().clear_bit());
        if !wait_bits_clear(|| {
            adc.status().read().syncbusy().bit_is_set() || adc.en().read().disabling().bit_is_set()
        }) {
            return Err(MicError::StuckDisabling);
        }

        adc.ctrl().write(|w| {
            w.warmupmode().normal();
            w.adcclksuspend0().prswudis();
            w.adcclksuspend1().prswudis();
            unsafe {
                // ceil(fsrc / 1 MHz) - 1; a larger value only lengthens the
                // internal timing windows, so 19 is safe for any MHz-scale clock.
                w.timebase().bits(19);
                w.hsclkrate().bits(0);
            }
            w
        });
        adc.cfg0().write(|w| {
            w.adcmode()
                .normal()
                .osrhs()
                .hispd2()
                .analoggain()
                .anagain1()
                .refsel()
                .vbgr()
        });
        adc.sched0().write(|w| unsafe { w.prescale().bits(1) });
        adc.singlefifocfg().write(|w| {
            w.alignment().right12();
            w.showid().clear_bit();
            w.dvl().valid1()
        });
        // Enable before writing the channel: SINGLE is a WSYNC register and the
        // vendor's driver says so in a comment — "IADCn->SINGLE has WSYNC type
        // and can only be written while enabled". Writing it disabled is a bus
        // fault (BFAR=0x49004098) and leaves the converter in DISABLING.
        adc.en().write(|w| w.en().set_bit());
        if !wait_bits_clear(|| {
            adc.status().read().syncbusy().bit_is_set() || adc.en().read().disabling().bit_is_set()
        }) {
            return Err(MicError::StuckDisabling);
        }

        adc.single().write(|w| {
            w.portpos().portc();
            unsafe {
                w.pinpos().bits(9); // PC9, the microphone output
                w.pinneg().bits(0);
            }
            w.cmp().clear_bit();
            w.portneg().gnd();
            w.cfg().config0()
        });
        if !wait_bits_clear(|| adc.status().read().singlewritepending().bit_is_set()) {
            return Err(MicError::WriteNotPublished);
        }
        // Let the microphone bias settle before the first conversion. Measured:
        // sampling immediately after configuration returned a railed input
        // (RMS 2028, peak 2048 of a 12-bit range) where the PAC twin, which does
        // its reset sequence first, read a 98/115 noise floor.
        cortex_m::asm::delay(300_000);
        Ok(Self { adc })
    }

    /// One conversion, or `None` if the converter did not deliver.
    pub fn sample(&self) -> Option<u16> {
        self.adc.cmd().write(|w| w.singlestart().set_bit());
        let mut spin = 0u32;
        while !self.adc.status().read().singlefifodv().bit_is_set() {
            spin += 1;
            if spin > 20_000 {
                return None;
            }
        }
        Some(self.adc.singledata().read().data().bits() as u16)
    }

    /// RMS and peak amplitude over `window` samples, around mid-scale.
    pub fn level(&self, window: usize) -> Level {
        let mut sum_sq: u64 = 0;
        let mut peak = 0u32;
        let mut samples = 0u32;
        for _ in 0..window {
            match self.sample() {
                Some(raw) => {
                    let centred = if raw >= 2048 {
                        (raw - 2048) as u32
                    } else {
                        (2048 - raw) as u32
                    };
                    sum_sq += (centred as u64) * (centred as u64);
                    if centred > peak {
                        peak = centred;
                    }
                    samples += 1;
                }
                None => break,
            }
        }
        let rms = if samples > 0 {
            isqrt((sum_sq / samples as u64) as u32)
        } else {
            0
        };
        Level { rms, peak, samples }
    }
}

/// Integer square root (no libm on a bare-metal target).
pub fn isqrt(value: u32) -> u32 {
    if value == 0 {
        return 0;
    }
    let mut x = value;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + value / x) / 2;
    }
    x
}
