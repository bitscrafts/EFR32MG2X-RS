//! Piezo buzzer on TIMER0 compare output — the Expansion Base's `D3` (PC03).
//!
//! ## Why a timer and not a delay loop
//!
//! A passive buzzer needs a square wave at the wanted pitch. Producing it by
//! toggling a pin in a NOP loop ties the CPU up and makes the pitch a function of
//! whatever else the program is doing. TIMER0's compare channel generates it in
//! hardware: the CPU sets a frequency and a duty, and is free.
//!
//! ## The sequence this die demands
//!
//! Measured, and in this order — every deviation is a bus fault or a silent timer:
//!
//! ```text
//! LOCK = 0xCE80            unlock the timer's configuration registers
//! EN   = 0                 configuration is only writable with the timer disabled
//! CFG  = up, prescem01grpaclk, div1
//! CC0_CFG = output compare  (also needs EN = 0)
//! EN   = 1
//! TOP  = period            (this one needs EN = 1)
//! CC0_OC = half period     (so the output is a 50 % square)
//! CC0_CTRL = toggle on compare
//! ```
//!
//! ## Why the frequency is calibrated instead of assumed
//!
//! `HFRCOEM23` does not start in the band the datasheet suggests: measured on this
//! board it runs at roughly 5 MHz at reset and settles near 19 MHz, so a `TOP`
//! computed from 20 MHz gave a tone 5 % flat. The driver therefore counts timer
//! ticks against a known CPU delay once, then computes `TOP` from what it measured.
//! See `docs/04-reference/PAC-CLOCK-AND-TIMER-NOTES.md`.
//!
//! ## Silence
//!
//! Stopping the timer stops `CNT` too, which breaks any timing that waits on it.
//! Silence is `CC0_CTRL.CMOA = none` with the timer still counting — measured
//! failure of the obvious approach.

use efr32mg24_pac as pac;

/// Which pin the base's buzzer is on: D3 = PC03, driven by TIMER0 CC0.
const BUZZER_PIN: u32 = 1 << 3;

/// The piezo on this board is loudest near 2.7 kHz; the base's own demo uses it.
pub const DEFAULT_TONE_HZ: u32 = 2700;

/// A calibrated buzzer.
pub struct Buzzer<'a> {
    timer: &'a pac::timer0_s::RegisterBlock,
    /// Measured TIMER0 tick rate in Hz (ticks per CPU cycle × SYSCLK).
    tick_hz: u32,
    /// The SYSCLK this rate was derived from.
    sysclk_hz: u32,
    top: u32,
}

impl<'a> Buzzer<'a> {
    /// Take TIMER0 and PC03, calibrate against the CPU, and stay silent.
    ///
    /// Returns `None` if the timer never reports a running counter, which is what a
    /// missing or mis-selected clock looks like.
    pub fn new(dp: &'a pac::Peripherals) -> Option<Self> {
        // TIMER0, GPIO and HFRCOEM23 must all be clocked before any register is
        // touched: a missing CMU_CLKEN0 bit is a precise bus fault on first access.
        dp.cmu_s
            .clken0()
            .modify(|_r, w| w.timer0().set_bit().gpio().set_bit().hfrcoem23().set_bit());
        dp.hfrcoem23_s.ctrl().modify(|_r, w| w.forceen().set_bit());
        let mut spin = 0u32;
        while !dp.hfrcoem23_s.status().read().rdy().bit_is_set() {
            spin += 1;
            if spin > 1_000_000 {
                return None;
            }
        }
        // TIMER0 clocked from HFRCOEM23: it keeps running in low energy modes, so a
        // beep pattern survives an EM2 sleep.
        dp.cmu_s
            .em01grpaclkctrl()
            .modify(|_r, w| w.clksel().hfrcoem23());

        // Route CC0 to PC03: the route register picks the pin, the route-enable
        // register opens the connection. Both, or the timer toggles a pin nobody is
        // looking at.
        dp.gpio_s.portc_model().modify(|_r, w| w.mode3().pushpull());
        dp.gpio_s
            .timer0_cc0route()
            .write(|w| unsafe { w.port().bits(2).pin().bits(3) });
        dp.gpio_s
            .timer0_routeen()
            .modify(|_r, w| w.cc0pen().set_bit());

        let timer = &dp.timer0_s;
        timer.lock().write(|w| unsafe { w.lockkey().bits(0xCE80) });
        timer.en().write(|w| w.en().clear_bit());
        timer
            .cfg()
            .write(|w| w.mode().up().clksel().prescem01grpaclk().presc().div1());
        timer.cc0_cfg().write(|w| w.mode().outputcompare());
        timer.en().write(|w| w.en().set_bit());

        // Calibrate: run the counter with the largest TOP and see how far it gets in
        // a CPU delay. `asm::delay` is a NOP count, so the ratio is stable even
        // though neither clock is exactly its nominal value.
        //
        // The counter wraps at TOP, and 0xFFFF ticks is only ~3.4 ms at 19 MHz: a
        // 10 ms measurement reads the remainder, not the count. Measured the hard
        // way — the first version reported 988 kHz and a "correct" 2.7 kHz tone
        // derived from a wrong rate. The window is halved until no wrap is seen.
        timer.top().write(|w| unsafe { w.top().bits(0xFFFF) });
        timer.cmd().write(|w| w.start().set_bit());

        // Measure timer ticks against CPU cycles, not against `asm::delay`
        // iterations: `delay(n)` costs about three cycles per iteration, so counting
        // iterations inflated the rate by that factor (measured: 3.0 exactly, which
        // turned 19 MHz into 57 MHz).
        let cycles = &unsafe { cortex_m::Peripherals::steal() }.DWT;
        let mut window: u32 = 4_000;
        let mut ticks: u32 = 0;
        let mut elapsed_cycles: u32 = 0;
        for _ in 0..6 {
            let ticks_before = u32::from(timer.cnt().read().cnt().bits());
            let cycles_before = cycles.cyccnt.read();
            cortex_m::asm::delay(window);
            let ticks_after = u32::from(timer.cnt().read().cnt().bits());
            let cycles_after = cycles.cyccnt.read();
            if ticks_after >= ticks_before {
                ticks = ticks_after - ticks_before;
                elapsed_cycles = cycles_after.wrapping_sub(cycles_before);
                if ticks >= 500 && elapsed_cycles > 0 {
                    break;
                }
            }
            window /= 2; // wrapped, or too few ticks to measure: try a smaller window
        }
        if ticks < 500 || elapsed_cycles == 0 {
            return None; // counting nothing usable: wrong clock, do not pretend otherwise
        }
        // Ticks per CPU cycle, scaled by 1000 to keep it integral.
        let ticks_per_1000_cycles = (u64::from(ticks) * 1000) / u64::from(elapsed_cycles);

        // Turning ticks-per-cycle into Hz needs SYSCLK, which this driver cannot
        // measure on its own. The value below is the figure this board produced when
        // the 1 Hz blink was calibrated against a stopwatch (docs/04-reference/
        // PAC-CLOCK-AND-TIMER-NOTES.md); a caller that knows better can override it
        // with `set_sysclk_hz`.
        const SYSCLK_HZ: u64 = 19_000_000;
        let tick_hz = (ticks_per_1000_cycles * SYSCLK_HZ / 1000) as u32;

        let mut buzzer = Self {
            timer,
            tick_hz,
            sysclk_hz: SYSCLK_HZ as u32,
            top: 0,
        };
        buzzer.set_tone(DEFAULT_TONE_HZ);
        buzzer.silence();
        Some(buzzer)
    }

    /// Override the assumed SYSCLK, in Hz.
    ///
    /// Only affects the absolute pitch: the timer is calibrated against CPU cycles,
    /// so a caller that measures SYSCLK precisely (against the 32.768 kHz reference,
    /// for instance) can remove the last few percent of error here.
    pub fn set_sysclk_hz(&mut self, hz: u32) {
        if hz > 1_000_000 {
            let ratio = (self.tick_hz as u64 * 1000) / self.sysclk_hz as u64;
            self.sysclk_hz = hz;
            self.tick_hz = ((ratio * u64::from(hz)) / 1000) as u32;
        }
    }

    /// The assumed SYSCLK, in Hz.
    #[must_use]
    pub fn sysclk_hz(&self) -> u32 {
        self.sysclk_hz
    }

    /// The measured TIMER0 tick rate, in Hz.
    #[must_use]
    pub fn tick_hz(&self) -> u32 {
        self.tick_hz
    }

    /// Set the pitch. Values between roughly 200 Hz and 8 kHz are usable.
    pub fn set_tone(&mut self, hz: u32) {
        if hz == 0 || self.tick_hz == 0 {
            return;
        }
        // One CC0 toggle per timer period, so the pin changes at tick/TOP and the
        // square wave — which is what a piezo reproduces — is half that.
        let period = (self.tick_hz / (hz * 2)).clamp(2, 0xFFFF);
        self.top = period;
        // Buffered registers: both of these need the timer enabled.
        self.timer.top().write(|w| unsafe { w.top().bits(period) });
        self.timer
            .cc0_oc()
            .write(|w| unsafe { w.oc().bits(period / 2) });
        self.timer
            .cc0_ctrl()
            .write(|w| w.cmoa().toggle().outinv().clear_bit());
    }

    /// Sound the tone.
    pub fn sound(&mut self) {
        self.timer
            .cc0_ctrl()
            .write(|w| w.cmoa().toggle().outinv().clear_bit());
    }

    /// Silence without stopping the counter, so timings that wait on `CNT` keep
    /// working.
    pub fn silence(&mut self) {
        self.timer.cc0_ctrl().write(|w| w.cmoa().none());
    }

    /// The configured period in timer ticks, for tests and reports.
    #[must_use]
    pub fn period_ticks(&self) -> u32 {
        self.top
    }

    /// Give the peripherals back.
    pub fn free(self) {
        self.timer.cmd().write(|w| w.stop().set_bit());
    }
}
