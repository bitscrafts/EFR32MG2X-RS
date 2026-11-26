//! TIMER0 as a PWM source on any pin.
//!
//! This is the module behind "the louder you shout, the brighter the light" — the demo Seeed
//! ships in the flash of every XIAO MG24 **Sense**. In Arduino that is `analogWrite(pin, 0..255)`;
//! here it is a compare channel with a duty cycle, because a GPIO has exactly two brightnesses
//! and a PWM has 1024.
//!
//! ## What this module owns, and what it does not
//!
//! It owns the timer: the clock gate, the oscillator choice, the route to one pin, and the
//! calibration of the tick rate against the CPU cycle counter. It does **not** know which pin
//! the caller wants, nor what is attached to it — an LED and a piezo are the same waveform at
//! different frequencies.
//!
//! ## The two facts that cost time here
//!
//! - **HFRCOEM23, not FSRCO.** The IADC and TIMER0 share the requirement; the oscillator is
//!   forced on and waited for, because a timer clocked by a stopped source reports a running
//!   counter and counts nothing.
//! - **`asm::delay(n)` is not `n` cycles.** It is ~3 cycles per iteration on this part, which
//!   turned a 19 MHz measurement into 57 MHz and a 2.7 kHz tone into something else entirely.
//!   The rate below is measured with `DWT->CYCCNT`, and the tick/cycle ratio is what is stored.
//!
//! ## One owner per timer
//!
//! `TIMER0` has one counter and one route per channel, so a Pwm and a Buzzer cannot be alive at
//! the same time. [`Timer0::take`] returns `None` the second time, which makes that a compile-time
//! discipline rather than a runtime surprise.

use efr32mg2x_pac as pac;

use crate::pin::{mode, set_mode, Port};

/// The timer, configured and calibrated, before it is given a meaning.
pub struct Timer0<'a> {
    reg: &'a pac::timer0_s::RegisterBlock,
    /// Measured ticks per second of the timer's own clock.
    tick_hz: u32,
    /// The SYSCLK the measurement was scaled by.
    sysclk_hz: u32,
}

impl<'a> Timer0<'a> {
    /// Take TIMER0, route channel 0 to one pin, and measure the tick rate.
    ///
    /// Returns `None` if the counter does not move — a clock that is not running, which is not
    /// the same thing as a pin that is not connected.
    pub(crate) fn take(
        dp: &'a pac::Peripherals,
        port: Port,
        pin: u8,
        sysclk_hz: u32,
    ) -> Option<Self> {
        if sysclk_hz == 0 {
            return None;
        }
        // Gate first: a register access with the bus clock off is a precise bus fault.
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
        dp.cmu_s
            .em01grpaclkctrl()
            .modify(|_r, w| w.clksel().hfrcoem23());

        // The pin, and the route that connects the channel to it. Both are needed: the route
        // register picks the pin, the route-enable register opens the connection.
        set_mode(&dp.gpio_s, port, pin, mode::PUSH_PULL);
        let port_index = match port {
            Port::A => 0,
            Port::B => 1,
            Port::C => 2,
            Port::D => 3,
        };
        // A pin above 15 does not fit `PIN` and would be truncated into a different pad, so it is refused here the
        // way the `TIMER` period is refused below when it does not fit `TOP`.
        if pin >= 16 {
            return None;
        }
        // SAFETY: `PORT` is 2 bits and `PIN` is 4. `port_index` comes from the `Port` match above (0..3) and `pin` was
        // just refused if it does not fit, so both fields hold exactly the pad the caller named.
        dp.gpio_s
            .timer0_cc0route()
            .write(|w| unsafe { w.port().bits(port_index).pin().bits(pin) });
        dp.gpio_s
            .timer0_routeen()
            .modify(|_r, w| w.cc0pen().set_bit());

        let reg = &dp.timer0_s;
        // SAFETY: the timer's `LOCKKEY` is a 16-bit field and `0xCE80` is the unlock sequence the reference manual
        // gives for TIMER; a wrong sequence is refused by the hardware.
        reg.lock().write(|w| unsafe { w.lockkey().bits(0xCE80) });
        reg.en().write(|w| w.en().clear_bit());
        // `CFG` faults while the timer is enabled (measured 2026-11-17) and clearing `EN` is not instantaneous; on a
        // timer that was already running the write below lands before the block has stopped. `EN.DISABLING` is the
        // handshake that says it has (measured 2026-11-26 on the PCNT measurement's second pacing).
        cortex_m::asm::delay(16);
        let mut settling = 0u32;
        while reg.en().read().disabling().bit_is_set() {
            settling += 1;
            if settling > 2_000_000 {
                break;
            }
        }
        reg.cfg()
            .write(|w| w.mode().up().clksel().prescem01grpaclk().presc().div1());
        reg.cc0_cfg().write(|w| w.mode().pwm());
        reg.en().write(|w| w.en().set_bit());

        // Calibrate: let the counter run with the largest TOP and see how far it gets while the
        // CPU counts its own cycles. The window is halved when the counter wrapped, because
        // 0xFFFF ticks is only ~3.4 ms at 19 MHz and a longer window reads the remainder.
        // SAFETY: `0xFFFF` is the 16-bit `TOP` field's own maximum -- the calibration wants the counter to run as long
        // as it can, and this is literally the largest value the field can hold.
        reg.top().write(|w| unsafe { w.top().bits(0xFFFF) });
        reg.cmd().write(|w| w.start().set_bit());

        // The counter has to be enabled before it can be read, and nothing here used to do that: `CYCCNT` stayed at
        // zero, the calibration measured zero cycles and `take` returned `None`, so the buzzer reported "timer never
        // started" while the timer was running perfectly. Measured 2026-11-25. `dwt::start` also proves the counter
        // moves, which is the difference between "no cycles elapsed" and "this instrument is not running here".
        // SAFETY: `dwt::start` asks the caller to be the only owner of the DWT. Nothing else in this crate
        // configures it, and the calibration below only reads `CYCCNT`.
        if unsafe { crate::dwt::start() }.is_err() {
            return None;
        }
        // SAFETY: `Board` took `cortex_m::Peripherals` once and keeps SysTick inside it; this borrows the DWT for the
        // calibration only, and `dwt::start` above has just proved the counter is running.
        let dwt = &unsafe { cortex_m::Peripherals::steal() }.DWT;
        let mut window: u32 = 4_000;
        let mut ticks: u32 = 0;
        let mut elapsed_cycles: u32 = 0;
        for _ in 0..6 {
            let ticks_before = reg.cnt().read().cnt().bits();
            let cycles_before = dwt.cyccnt.read();
            cortex_m::asm::delay(window);
            let ticks_after = reg.cnt().read().cnt().bits();
            let cycles_after = dwt.cyccnt.read();
            if ticks_after >= ticks_before {
                ticks = ticks_after - ticks_before;
                elapsed_cycles = cycles_after.wrapping_sub(cycles_before);
                if ticks >= 500 && elapsed_cycles > 0 {
                    break;
                }
            }
            window /= 2;
        }
        if ticks < 500 || elapsed_cycles == 0 {
            return None;
        }
        let ticks_per_1000_cycles = (u64::from(ticks) * 1000) / u64::from(elapsed_cycles);

        // `sysclk_hz` is what the caller configured, not what this driver would like it to be.
        //
        // It used to be typed in here as 19 MHz. A driver that assumes its own bus frequency turns a
        // wrong clock into a wrong period with no error anywhere: the timer accepts the divider, the
        // waveform comes out at the wrong pitch, and nothing in the chain complains. The board knows
        // the clock it asked for, so it says it (`Board::pwm_on`, `Board::buzzer_on`), and a caller
        // that changes the clock afterwards still has `set_sysclk_hz` for the correction.
        let tick_hz = (ticks_per_1000_cycles * u64::from(sysclk_hz) / 1000) as u32;

        Some(Self {
            reg,
            tick_hz,
            sysclk_hz,
        })
    }

    /// The measured tick rate of the timer's clock, in Hz.
    #[must_use]
    pub fn tick_hz(&self) -> u32 {
        self.tick_hz
    }

    /// The SYSCLK the rate was scaled by.
    #[must_use]
    pub fn sysclk_hz(&self) -> u32 {
        self.sysclk_hz
    }

    /// Tell the timer what SYSCLK actually is, keeping the measured tick/cycle ratio.
    pub fn set_sysclk_hz(&mut self, hz: u32) {
        if hz > 1_000_000 {
            let ratio = (self.tick_hz as u64 * 1000) / u64::from(self.sysclk_hz);
            self.sysclk_hz = hz;
            self.tick_hz = ((ratio * u64::from(hz)) / 1000) as u32;
        }
    }
}

/// A PWM channel: a frequency, and a duty cycle between 0 % and 100 %.
pub struct Pwm<'a> {
    timer: Timer0<'a>,
    top: u32,
}

impl<'a> Pwm<'a> {
    /// Start a PWM at `freq_hz`, dark (0 % duty).
    ///
    /// Frequencies between about 100 Hz and 20 kHz are usable for a visible LED; above that the
    /// timer's 16-bit TOP runs out at the top of the range.
    pub fn new(timer: Timer0<'a>, freq_hz: u32) -> Self {
        let mut pwm = Self { timer, top: 0 };
        pwm.set_frequency(freq_hz);
        pwm.set_duty(0);
        pwm
    }

    /// Change the PWM frequency, keeping the current duty.
    pub fn set_frequency(&mut self, freq_hz: u32) {
        if freq_hz == 0 || self.timer.tick_hz == 0 {
            return;
        }
        let top = (self.timer.tick_hz / freq_hz).clamp(2, 0xFFFF);
        self.top = top;
        // `CC0_CFG` is not writable while the counter is enabled: measured 2026-11-17, a precise bus fault at
        // TIMER0 + CC0_CFG. The order is the one `Timer0::take` uses -- counter down, configure, counter up, and only
        // then `TOP`, which has to be written with the counter enabled (also measured, at TIMER0 + TOP).
        self.timer.reg.en().write(|w| w.en().clear_bit());
        // `EN` crosses into the timer's own clock domain, and `CC0_CFG` written before it lands faults the core --
        // measured 2026-11-25 at `TIMER0 + 0x60`, precisely, with the counter running. This block exposes no
        // `SYNCBUSY` in the PAC to poll, so the wait is a handful of cycles, which is all the crossing takes.
        cortex_m::asm::delay(16);
        self.timer.reg.cc0_cfg().write(|w| w.mode().pwm());
        self.timer.reg.en().write(|w| w.en().set_bit());
        // SAFETY: `top` is clamped to `2..=0xFFFF` above, which is exactly the range the 16-bit field holds; the
        // clamp is what keeps `bits()` from truncating.
        self.timer.reg.top().write(|w| unsafe { w.top().bits(top) });
    }

    /// The frequency the channel is running at, in Hz.
    #[must_use]
    pub fn frequency_hz(&self) -> u32 {
        self.timer.tick_hz.checked_div(self.top).unwrap_or(0)
    }

    /// Set the duty cycle, 0..=100. Out-of-range values are clamped, not wrapped: a caller that
    /// asks for 200 % wanted "as bright as it goes", not 71 %.
    ///
    /// In `CC0_CFG.MODE = PWM` the waveform is made by the comparison against `CC0_OC`, not by
    /// the `CMOA` action field — that field only has None/Toggle/Clear/Set and belongs to compare
    /// mode. Writing the compare value is therefore the whole of setting the duty, and because it
    /// is buffered the change lands on the next period instead of tearing the current one.
    pub fn set_duty(&mut self, percent: u32) {
        let percent = percent.min(100);
        let ticks = (u64::from(self.top) * u64::from(percent) / 100) as u32;
        // SAFETY: `OC` is the 16-bit compare field and `ticks` is `top * percent / 100` with `percent <= 100`, so it
        // can never exceed the `top` the field already holds.
        self.timer
            .reg
            .cc0_oc()
            .write(|w| unsafe { w.oc().bits(ticks) });
    }

    /// Turn the output off. The timer keeps running, so the next `set_duty` is immediate.
    pub fn off(&mut self) {
        self.timer.reg.cc0_ctrl().write(|w| w.cmoa().none());
    }

    /// The timer back, for a caller that wants to reuse it (a buzzer, for example).
    #[must_use]
    pub fn free(mut self) -> Timer0<'a> {
        self.off();
        self.timer
    }
}

impl<'a> Timer0<'a> {
    /// Drive channel 0 as a square wave at `hz`, half period high and half low.
    ///
    /// This is what a piezo wants: it reproduces the fundamental of a square wave, so a 2.7 kHz
    /// square sounds like a 2.7 kHz tone. Toggle mode changes the pin once per period, which is
    /// why the period is computed as `tick / (2 * hz)`.
    pub fn square_wave(&mut self, hz: u32) -> u32 {
        if hz == 0 || self.tick_hz == 0 {
            return 0;
        }
        let period = (self.tick_hz / (hz * 2)).clamp(2, 0xFFFF);
        // The same order `set_frequency` uses, for the same measured reason: `CC0_CFG` written with the counter running
        // faults the core, and `TOP`/`OC` want it running. This is the bug the buzzer path had until 2026-11-25 -- it
        // never reached this line, because the calibration above returned `None` first.
        self.reg.en().write(|w| w.en().clear_bit());
        // Same crossing as `set_frequency`: `EN` has to land before `CC0_CFG` is written.
        cortex_m::asm::delay(16);
        self.reg.cc0_cfg().write(|w| w.mode().outputcompare());
        self.reg.en().write(|w| w.en().set_bit());
        // SAFETY: `period` is clamped to `2..=0xFFFF` above, the range the 16-bit `TOP` field holds.
        self.reg.top().write(|w| unsafe { w.top().bits(period) });
        // SAFETY: half of a value that already fits `TOP` also fits `OC`, both fields being 16 bits.
        self.reg
            .cc0_oc()
            .write(|w| unsafe { w.oc().bits(period / 2) });
        self.reg
            .cc0_ctrl()
            .write(|w| w.cmoa().toggle().outinv().clear_bit());
        period
    }

    /// Drive the pin with the square wave `square_wave` configured.
    pub fn sound(&mut self) {
        self.reg
            .cc0_ctrl()
            .write(|w| w.cmoa().toggle().outinv().clear_bit());
    }

    /// The period `square_wave` last set, in timer ticks.
    #[must_use]
    pub fn period_ticks(&self) -> u32 {
        self.reg.top().read().top().bits()
    }

    /// Stop driving the pin without stopping the counter.
    pub fn silence(&mut self) {
        self.reg.cc0_ctrl().write(|w| w.cmoa().none());
    }
}
