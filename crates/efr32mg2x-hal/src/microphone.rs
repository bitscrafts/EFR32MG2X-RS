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

use efr32mg2x_pac as pac;

/// The converter, as the microphone driver needs it.
///
/// A handle and not a register block: the public API of this crate must never hand a caller a
/// `pac` type, because the moment it does, the caller has to own the PAC singleton to use a
/// driver — which is exactly how an example ends up writing `Peripherals::steal()`.
#[derive(Clone, Copy)]
pub struct Converter {
    reg: &'static pac::iadc0_s::RegisterBlock,
}

impl Converter {
    /// Wrap the converter's register block. Only `hal::board` calls this.
    pub(crate) const fn new(reg: &'static pac::iadc0_s::RegisterBlock) -> Self {
        Self { reg }
    }

    /// Start a microphone on this converter.
    ///
    /// # Errors
    ///
    /// [`MicError`] when the converter will not leave its disabled state.
    pub fn microphone(self) -> Result<Microphone<'static>, MicError> {
        Microphone::new(self.reg)
    }
}

/// Result of one measurement window.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Level {
    /// Root-mean-square amplitude **about the window's mean**, in ADC counts -- the signal, with the converter's bias
    /// removed. Before 2026-11-17 this was computed about the fixed mid-scale value 2048, which reported the bias
    /// itself: on this board `rms` was 343 for a signal of 8, because the bias sits at 2391.
    pub rms: u32,
    /// Largest deviation from the window's mean seen in the window.
    pub peak: u32,
    /// The window's mean, i.e. the converter's bias for this window. Report it next to `rms`: a level quoted without
    /// the bias it was measured against cannot be reproduced.
    pub mean: u32,
    /// Smallest and largest raw sample in the window. Reported because they bound every other number here: an `rms`
    /// larger than half the span, or a `peak` smaller than it, means the arithmetic and the samples disagree.
    pub min: u16,
    /// Largest raw sample in the window.
    pub max: u16,
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
    pub(crate) fn new(adc: &'a pac::iadc0_s::RegisterBlock) -> Result<Self, MicError> {
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

    /// Is the IADC's local timer running? `STATUS.TIMERACTIVE`, "The local timer is running".
    #[must_use]
    pub fn timer_active(&self) -> bool {
        self.adc.status().read().timeractive().bit_is_set()
    }

    /// Configure the single queue to trigger from the IADC's local timer, and load the period.
    ///
    /// `IADC_TIMER` counts **ADC_CLK cycles** — `em_iadc.h`, field `timerCycles`: "Number of ADC_CLK cycles per
    /// TIMER event" — so the caller passes the converter's own clock, not the bus clock. This driver divides
    /// `CLK_SRC_ADC` by two in `SCHED0.PRESCALE`, so with the 19 MHz HFRCOEM23 that is 9,5 MHz and 16 kHz is a
    /// period of 594 (15 993 Hz, 0,04 % low).
    ///
    /// Nothing here starts the timer: [`start_timer`](Self::start_timer) does, and it goes **after** the queue
    /// has been armed, which is the order the vendor driver leaves to the application
    /// (`iadcCmdEnableTimer` is an application command, not part of `IADC_Init`).
    ///
    /// # Errors
    ///
    /// [`MicError::StuckDisabling`] when the converter will not leave its enabled state: `TRIGGER` is
    /// write-protected while it runs, and writing it anyway raised a precise bus error at `0x4900_402C`
    /// (measured 2026-11-17, `CFSR=0x00008200`, `BFAR=0x4900402C`).
    pub fn pace_from_timer(&mut self, adc_clk_hz: u32, hz: u32) -> Result<u32, MicError> {
        if hz == 0 || adc_clk_hz == 0 {
            return Err(MicError::StuckDisabling);
        }
        self.adc.en().write(|w| w.en().clear_bit());
        let mut spin = 0u32;
        while self.adc.en().read().disabling().bit_is_set() {
            spin += 1;
            if spin > 1_000_000 {
                return Err(MicError::StuckDisabling);
            }
        }
        let cycles = adc_clk_hz / hz;
        self.adc
            .trigger()
            .modify(|_r, w| w.singletrigsel().timer().singletrigaction().once());
        self.adc
            .timer()
            .write(|w| unsafe { w.timer().bits(cycles as u16) });
        self.adc.en().write(|w| w.en().set_bit());
        Ok(cycles)
    }

    /// Set the timer period again, without touching the trigger configuration.
    ///
    /// The timer counts `CLK_SRC_ADC`, and that clock is not always what the datasheet's default suggests: the
    /// Bluetooth stack reconfigures the clock tree when it starts, and a period of 1187 that gives 16 kHz in a
    /// bare example gave 8 kHz with the stack running — measured 2026-11-17, a loop that read exactly twice
    /// real time. Rather than assume the clock, the caller can measure one window in core cycles and pass
    /// `cycles * measured / expected` here; the relationship is linear, so one iteration converges.
    ///
    /// # Errors
    ///
    /// [`MicError::StuckDisabling`] when the converter will not leave its enabled state to be reconfigured.
    pub fn retune_period(&mut self, cycles: u32) -> Result<u32, MicError> {
        self.adc.en().write(|w| w.en().clear_bit());
        let mut spin = 0u32;
        while self.adc.en().read().disabling().bit_is_set() {
            spin += 1;
            if spin > 1_000_000 {
                return Err(MicError::StuckDisabling);
            }
        }
        self.adc
            .timer()
            .write(|w| unsafe { w.timer().bits(cycles as u16) });
        self.adc.en().write(|w| w.en().set_bit());
        self.adc.cmd().write(|w| w.timerdis().set_bit());
        self.adc.cmd().write(|w| w.timeren().set_bit());
        Ok(cycles)
    }

    /// Arm the single queue once, so a trigger has a queue to start.
    pub fn arm(&mut self) {
        self.adc.cmd().write(|w| w.singlestart().set_bit());
    }

    /// Start the local timer. Call this **after** [`arm`](Self::arm).
    ///
    /// `TIMERDIS` first: "Disable the local timer and reset the counter to timer reload value" — the reset is
    /// what loads the `TIMER` register written by `pace_from_timer`, so the pair is DIS then EN.
    ///
    /// Both writes go through `CMD`, which faults while the converter is **disabled** (`BFAR=0x4900_400C`,
    /// measured 2026-11-17). That is the mirror of `TRIGGER`, which faults while it is **enabled**
    /// (`BFAR=0x4900_402C`): configuration goes in while disabled, commands while enabled, and there is no
    /// single moment when both are allowed.
    pub fn start_timer(&mut self) {
        self.adc.cmd().write(|w| w.timerdis().set_bit());
        self.adc.cmd().write(|w| w.timeren().set_bit());
    }

    /// Wait for the conversion the timer started, and read it — without issuing a start of its own.
    ///
    /// [`sample`](Self::sample) writes `CMD.SINGLESTART` on every call, which is what makes it free-running;
    /// a paced loop must not do that, or the rate is the CPU's, not the timer's (measured: 256 samples in
    /// 8 741 cycles under a 16 kHz trigger, i.e. 556 kHz).
    pub fn read_paced(&self) -> Option<u16> {
        // The budget is two sample periods, not a generous timeout. In sync, the valid flag is either already
        // set (the CPU was late) or arrives within one period of 1187 cycles; if it is not there in two, the
        // FIFO overran and waiting longer only wastes time. Measured 2026-11-17 with the BLE stack running: a
        // 200 000-cycle budget cost 200 000 cycles on every miss, and the application loop with BLE went to
        // 206 % of real time because of it — one missed read per window.
        const BUDGET: u32 = 2_500;
        let mut spin = 0u32;
        while !self.adc.status().read().singlefifodv().bit_is_set() {
            spin += 1;
            if spin > BUDGET {
                return None;
            }
        }
        // SINGLEFIFODATA, not SINGLEDATA: only the FIFO read *pops*, and a pop is what clears the valid flag
        // for the next conversion. Reading SINGLEDATA left the flag set and the paced loop returned the same
        // value instantly — 256 samples in 8 741 cycles under a 16 kHz timer, measured 2026-11-17.
        Some(self.adc.singlefifodata().read().data().bits() as u16)
    }

    /// Fill a whole window with paced samples, returning how many actually arrived.
    ///
    /// The paced read waits for the timer, so a full window spans `out.len() / hz` seconds of wall time — 16 ms
    /// for 256 samples at 16 kHz. Nothing between windows is buffered in hardware: the converter keeps
    /// converting whether or not the CPU reads, so a caller that spends too long computing features will find
    /// the buffer short. The count is returned rather than an `Err` because "how many arrived" is the number
    /// that tells a streaming loop whether it is keeping up.
    pub fn capture_window(&mut self, out: &mut [u16]) -> usize {
        let mut n = 0usize;
        for slot in out.iter_mut() {
            match self.read_paced() {
                Some(v) => {
                    *slot = v;
                    n += 1;
                }
                None => break,
            }
        }
        n
    }

    /// One conversion, or `None` if the converter did not deliver.
    ///
    /// Reads `SINGLEDATA`. A review on 2026-11-17 argued this is wrong — that `SINGLEDATA` returns the
    /// *latest* value and never pops the FIFO, so `SINGLEFIFODV` would stay set and the same value would come
    /// back at CPU speed. Reading `SINGLEFIFODATA` instead was tried and measured on the board the same night:
    /// the level went from `rms=593 pk=615` to a dead `rms=2048 pk=2048`, so in this configuration the FIFO
    /// path returns nothing and `SINGLEDATA` is the register that carries the conversion. The review's
    /// mechanism was not reproduced; see `crates/xiao-mg24-bsp/examples/module_mic_probe.rs`, which counts
    /// distinct values in a window so that "repeated data" is a measurement rather than an argument.
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

    /// RMS and peak amplitude over `window` samples, about the window's **own mean**.
    ///
    /// The mean, not mid-scale: this converter's bias is not 2048. Measured on the board 2026-11-17
    /// (`examples/module_mic_stats.rs`, one 63-sample window): `min=2371 max=2405 mean=2391`, so the bias sits 343 counts
    /// above mid-scale while the signal about that mean is 8 counts. Centring on 2048 therefore reported `rms=343` --
    /// the reference error -- for a signal of 8. Any caller that wants a sound level has to remove the bias, and the
    /// caller cannot know the bias without measuring it, which is what this function now does.
    ///
    /// Peak and RMS come from the same pass over the buffered samples, so the four numbers in `Level` describe one set
    /// of samples and can be checked against each other (`rms <= peak <= max - min`).
    pub fn level(&self, window: usize) -> Level {
        // The deviation is taken from the samples themselves, not from sums of squares. A closed form
        // (`E[x^2] - E[x]^2`) was tried first and produced numbers that violated their own bounds -- `rms=51` for a
        // window whose extreme deviation was 27, which Bhatia-Davis rules out for any distribution. Two passes over a
        // buffer cost 256 bytes of stack and cannot be wrong about which samples they are describing.
        let mut buf = [0u16; MAX_WINDOW];
        let window = window.min(MAX_WINDOW);
        let mut samples = 0usize;
        let mut sum: u64 = 0;
        while samples < window {
            match self.sample() {
                Some(raw) => {
                    buf[samples] = raw;
                    sum += raw as u64;
                    samples += 1;
                }
                None => break,
            }
        }
        if samples == 0 {
            return Level {
                rms: 0,
                peak: 0,
                mean: 0,
                min: 0,
                max: 0,
                samples: 0,
            };
        }
        let mean = (sum / samples as u64) as i32;
        let mut sum_sq: u64 = 0;
        let mut min = u16::MAX;
        let mut max = 0u16;
        let mut peak = 0u32;
        for raw in buf.iter().take(samples) {
            let deviation = (*raw as i32 - mean).unsigned_abs();
            sum_sq += (deviation as u64) * (deviation as u64);
            if deviation > peak {
                peak = deviation;
            }
            min = if *raw < min { *raw } else { min };
            max = if *raw > max { *raw } else { max };
        }
        let rms = isqrt((sum_sq / samples as u64) as u32);
        Level {
            rms,
            peak,
            mean: mean as u32,
            min,
            max,
            samples: samples as u32,
        }
    }
}

/// Largest window `level()` will measure, and the buffer it needs: 128 samples is 256 bytes of stack, and it is two
/// windows of the dosimeter's 63-sample second. Asking for more returns this many rather than overflowing.
pub const MAX_WINDOW: usize = 128;

/// Integer square root (no libm on a bare-metal target).
pub fn isqrt(value: u32) -> u32 {
    if value == 0 {
        return 0;
    }
    let mut x = value;
    let mut y = x.div_ceil(2);
    while y < x {
        x = y;
        y = (x + value / x) / 2;
    }
    x
}
