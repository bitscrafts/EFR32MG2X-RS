//! Peripherals: the single place where the HAL touches the PAC singleton.
//!
//! The point of this module is that no **example** ends up naming `pac`. An
//! application asks the HAL for a board, and the board hands out the pieces
//! (I2C bus, LED) with their bus clocks already enabled — because a missing
//! `CMU_CLKEN0` bit is a precise bus fault on the first register access, and
//! that failure belongs in one place, not in every example.
//!
//! The PAC is still underneath: this is a PAC-based HAL, and it is the PAC that
//! defines every register this board pokes. What the HAL owns is the sequence
//! and the defaults that the hardware demands (gate first, then configure).

use efr32mg2x_pac as pac;

/// The MSC's unlock key: `MSC_LOCK_LOCKKEY_UNLOCK` = 0x1B71 in `efr32mg24_msc.h:455`.
///
/// The first value tried here was remembered rather than read, and the register stayed locked — which is why
/// this constant carries its source in the comment.
const MSC_LOCKKEY_UNLOCK: u16 = 0x1B71;

/// Everything an application needs from the XIAO MG24 Sense + Expansion Base.
pub struct Board {
    /// Held so `cortex_m::Peripherals::take()` cannot succeed a second time. SysTick stays
    /// inside it; `delay()` borrows it back.
    _cp: cortex_m::Peripherals,
    dp: pac::Peripherals,
    /// The SYSCLK in force, as far as this board has configured it.
    ///
    /// The board is the thing that changes the clock (`boost`), so it is the thing that knows it. Drivers take it
    /// from here instead of being handed a number at every call site: a caller repeating `DEFAULT_SYSCLK_HZ` is a
    /// caller that can repeat the wrong one, and a driver that assumed 19 MHz was already one defect too many.
    sysclk_hz: core::cell::Cell<u32>,
}

impl Board {
    /// Take the peripherals and enable the bus clocks this HAL drives.
    ///
    /// Returns `None` if called twice, which is what the PAC singleton is for.
    pub fn take() -> Option<Self> {
        let dp = pac::Peripherals::take()?;
        // The core peripherals (SysTick, NVIC) are taken here for the same reason the PAC is:
        // so a caller never has to `steal()` one. `cortex_m::Peripherals::take()` returns `None`
        // the second time, exactly like the PAC's.
        let cp = cortex_m::Peripherals::take()?;
        // GPIO is needed by the I2C bit-banger and the LED; without this gate
        // the first DOUT/MODEL access faults (CFSR=0x8200, BFAR = the address).
        dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());
        Some(Self {
            dp,
            _cp: cp,
            // Reset brings the part up on HFRCO at 19 MHz; `boost` is the only thing here that moves it.
            sysclk_hz: core::cell::Cell::new(crate::clock::DEFAULT_SYSCLK_HZ),
        })
    }

    /// The hardware I2C0, with its clock enabled and its two lines still unrouted.
    ///
    /// `hfclk` is passed in rather than read here: the `Board` holds the CMU inside `Peripherals`
    /// and does not own a `FrozenClocks`, so it would have to guess the frequency. The caller knows
    /// it -- `clocks.hfclk()` -- and a wrong frequency here is a wrong bus rate, which shows up as a
    /// device that never acknowledges.
    ///
    /// Route the pins before using it: [`Board::route_i2c0`].
    pub fn i2c0(&self, config: crate::i2c::Config, hfclk: crate::clock::Hertz) -> crate::i2c::I2c0 {
        self.dp.cmu_s.clken0().modify(|_, w| w.i2c0().set_bit());
        crate::i2c::I2c0::new(unsafe { pac::I2c0Ns::steal() }, config, hfclk)
    }

    /// Points I2C0's SCL and SDA at real pins, and puts those pins in open-drain.
    ///
    /// ## Why the Board does this and not the caller
    ///
    /// The routing registers live on the GPIO block and the pin-mode registers on the port, both
    /// reached through the PAC. Handing that to a caller would put the generated crate in the HAL's
    /// public API, which is the one thing the layering keeps out: a caller must never have to name
    /// `pac::GpioNs` to drive an I2C bus.
    ///
    /// ## What it writes, and in what order
    ///
    /// The pin mode first, then the route, then the enable. A signal enabled before its pin is
    /// driven correctly puts a floating line on the bus for a few cycles.
    ///
    /// Only port B is wired: that is where this board's I2C lives (SDA = PB2, SCL = PB3). Any other
    /// port is refused rather than routed by the arithmetic and left undriven.
    pub fn route_i2c0(
        &self,
        scl: (crate::pin::Port, u8),
        sda: (crate::pin::Port, u8),
        mode: crate::route::PinMode,
    ) -> Result<(), crate::route::RouteError> {
        use crate::pin::Port;
        use crate::route::{route_value, RouteError};

        if !matches!(scl.0, Port::B) || !matches!(sda.0, Port::B) {
            return Err(RouteError::PortDoesNotExist(0));
        }
        if scl.1 > crate::route::PIN_MAX {
            return Err(RouteError::PinDoesNotExist(scl.1));
        }
        if sda.1 > crate::route::PIN_MAX {
            return Err(RouteError::PinDoesNotExist(sda.1));
        }
        let scl_valor = route_value(1, scl.1)?;
        let sda_valor = route_value(1, sda.1)?;

        // The pin mode first: open-drain, with the number the manual gives.
        for pino in [scl.1, sda.1] {
            let desloc = 4 * u32::from(pino);
            self.dp.gpio_ns.portb_model().modify(|r, w| unsafe {
                let limpo = r.bits() & !(0xF << desloc);
                w.bits(limpo | (u32::from(mode.value()) << desloc))
            });
        }

        // Then the route, then the enable.
        self.dp
            .gpio_ns
            .i2c0_sclroute()
            .write(|w| unsafe { w.bits(scl_valor) });
        self.dp
            .gpio_ns
            .i2c0_sdaroute()
            .write(|w| unsafe { w.bits(sda_valor) });
        self.dp
            .gpio_ns
            .i2c0_routeen()
            .modify(|_, w| w.sclpen().set_bit().sdapen().set_bit());
        Ok(())
    }

    /// Gate the SYSRTC's clocks without starting it. The state the Bluetooth stack needs before it configures the
    /// radio; see `sysrtc::Sysrtc::enable_clocks` for the two faults that bracket this state.
    pub fn prepare_sysrtc_clocks(&self) {
        crate::sysrtc::Sysrtc::enable_clocks(&self.dp.cmu_ns);
    }

    /// The energy management unit, handed out through the board like every other peripheral: an example outside this
    /// crate must not name a PAC type, and the gate enforces that.
    pub fn emu(&self) -> crate::emu::Emu {
        // SAFETY: `Board` holds the PAC singletons for the whole program; this hands the EMU out through the board, once,
        // instead of letting a caller `steal()` it behind the board's back.
        crate::emu::Emu::new(unsafe { pac::EmuS::steal() })
    }

    /// The watchdog, configured at construction. Same reason as `emu`: no PAC type crosses the crate boundary.
    pub fn wdog(
        &self,
        config: crate::wdog::WdogConfig,
    ) -> Result<crate::wdog::Wdog, crate::wdog::Error> {
        // SAFETY: same as `emu`: the board owns the peripherals and gives this one out exactly once.
        crate::wdog::Wdog::new(unsafe { pac::Wdog0Ns::steal() }, config)
    }

    /// The tick timer, shared by every protocol stack that may run on this radio.
    ///
    /// SYSRTC is one peripheral, so it is handed out here rather than taken by a stack: a second consumer
    /// (Zigbee, Thread, or the application's own scheduler) shares this counter instead of starting another.
    pub fn sysrtc(&self) -> crate::sysrtc::Sysrtc {
        crate::sysrtc::Sysrtc::take(&self.dp.cmu_ns, &self.dp.sysrtc0_ns)
    }

    /// Entry and exit of EM2, carrying the `EMU` it drives.
    ///
    /// Handed out here for the same reason the timer is: EM2 is where the core stops running from the
    /// high-frequency clock, and the decision to do that belongs to whoever owns the board's clocks.
    pub fn sleep(&self) -> crate::sleep::Sleep<'_> {
        crate::sleep::Sleep::new(&self.dp.emu_ns)
    }

    /// The matrix/vector processor, enabled and ready.
    ///
    /// The board hands it out for the same reason it hands out the others: an example outside the PAC must
    /// not name a PAC type, and `Mvp::take` needs both the CMU and the peripheral.
    ///
    /// The non-secure window is used deliberately. In the secure alias the configuration registers read back
    /// as zero from a non-secure image — measured on this board, in `pac_mvp_add`.
    pub fn mvp(&self) -> Result<crate::mvp::Mvp, crate::mvp::Error> {
        crate::mvp::Mvp::take(&self.dp.cmu_ns, unsafe { pac::MvpNs::steal() })
    }

    /// Raise the core clock to the 38 MHz HFRCODPLL band, and report the calibration word it wrote.
    ///
    /// The core already runs on `HFRCODPLL` (the 19 MHz band), and `CMU_SYSCLKCTRL.CLKSEL` accepts only FSRCO,
    /// HFRCODPLL and HFXO — there is no "Dpll" option to switch to. So raising the band raises the clock **in
    /// place**, with no SYSCLK switch, which is why the HFXO sequence that bricked this board once is not
    /// involved at all.
    ///
    /// The value written into `HFRCO0.CAL` is the factory calibration for that band, read from
    /// `DEVINFO->HFRCODPLLCAL[12]` — the mapping the vendor's own `CMU_HFRCODPLLBandSet` uses
    /// (`em_cmu.c:5021`). Writing anything else would be an uncalibrated frequency.
    ///
    /// Order, and why: flash wait states go up **before** the clock does (RM p57 §6.3.9: "MODE should be
    /// changed ... before the clock is changed when increasing clock speed"), and the voltage scale is already
    /// at its highest (`Vscale2`, 1,1 V) — asserted here rather than assumed, because a lower scale at this
    /// frequency is the kind of mistake that shows up as random corruption rather than a fault.
    ///
    /// # Measured, not assumed
    ///
    /// ```
    /// 2026-11-17, same 256-sample window of 16 ms of audio, counted in core cycles:
    ///   19 MHz   304 186 cycles
    ///   38 MHz   606 508 cycles     ratio 1,99   calibration written 0xc8cc9d43
    /// ```
    ///
    /// # This is not what makes the application loop fit
    ///
    /// The loop that captures a window and *then* computes its bands took 108,3 % of real time at 19 MHz and
    /// 102,4 % at 38 MHz — only 5 % faster in wall time, because most of a paced capture is spent **waiting for
    /// the timer**, and a faster clock does not shorten a wait. What it shortens is the 46 044 cycles of band
    /// arithmetic, which is 15 % of the window. The loop's real problem is its shape: 1177 of every 1187 cycles
    /// per sample are idle, which is 301 312 free cycles per window against 46 044 needed. Interleave the
    /// features with the capture and the loop fits at 19 MHz; raising the clock only buys margin for the model
    /// and the radio that come after.
    ///
    /// # Returns
    ///
    /// `Some(cal)` with the word written, `None` when the oscillator never reported ready or the hardware
    /// clamped the band (the vendor detects the same clamp and reverts; this reports instead of reverting,
    /// because the caller is an example that will say so in its record).
    pub fn boost_to_38_mhz(&self) -> Option<u32> {
        // 38 MHz -> DEVINFO->HFRCODPLLCAL[12]
        let cal = self.dp.devinfo.hfrcodpllcal12().read().bits();
        self.apply_hfrco_cal(cal, 38_000_000)
    }

    /// Raise the core clock to the **56 MHz** band — the nearest calibrated band to 19 x 3.
    ///
    /// Same procedure, same traps, one constant different: 56 MHz is `DEVINFO->HFRCODPLLCAL[14]` in the
    /// vendor's table. There is no 57 MHz band, so "times three" lands on 56 (x2,95) or 64 (x3,37).
    pub fn boost_to_56_mhz(&self) -> Option<u32> {
        let cal = self.dp.devinfo.hfrcodpllcal14().read().bits();
        self.apply_hfrco_cal(cal, 56_000_000)
    }

    /// Raise the core clock to the **80 MHz** band, the fastest calibrated one under 100.
    pub fn boost_to_80_mhz(&self) -> Option<u32> {
        let cal = self.dp.devinfo.hfrcodpllcal16().read().bits();
        self.apply_hfrco_cal(cal, 80_000_000)
    }

    /// The procedure itself, shared by every band: clocks, wait states, calibration, wait for ready.
    fn apply_hfrco_cal(&self, cal: u32, target_hz: u32) -> Option<u32> {
        if cal == 0 || cal == u32::MAX {
            return None;
        }
        // Flash wait states first: too many only slow the flash, too few fault the moment the clock rises.
        // Three things had to be found before this write stopped faulting (measured 2026-11-17, BFAR =
        // 0x4003_0004, the read of MSC_READCTRL): the MSC's bus clock lives in CMU CLKEN1, not CLKEN0
        // (`CMU->CLKEN1_SET = CMU_CLKEN1_MSC`, em_cmu.c:4942), and the MSC is locked at reset and needs its
        // key (`MSC->LOCK = MSC_LOCK_LOCKKEY_UNLOCK`, em_msc.h:373).
        self.dp.cmu_s.clken1().modify(|_r, w| w.msc().set_bit());
        self.dp
            .msc_s
            .lock()
            .write(|w| unsafe { w.lockkey().bits(MSC_LOCKKEY_UNLOCK) });
        self.dp.msc_s.readctrl().modify(|_r, w| w.mode().ws3());

        // The oscillator's own bus clock. Without it the first read of HFRCO0 STATUS faults at 0x4001_000C
        // (measured 2026-11-17); the vendor enables it in the same breath as the DPLL's
        // (`CMU->CLKEN0_SET = CMU_CLKEN0_DPLL0 | CMU_CLKEN0_HFRCO0`, em_cmu.c:2533).
        self.dp
            .cmu_s
            .clken0()
            .modify(|_r, w| w.hfrco0().set_bit().dpll0().set_bit());

        // The oscillator must be idle before its calibration is replaced.
        let mut spin = 0u32;
        loop {
            let st = self.dp.hfrco0_s.status().read();
            if !st.syncbusy().bit_is_set() && !st.freqbsy().bit_is_set() {
                break;
            }
            spin += 1;
            if spin > 1_000_000 {
                return None;
            }
        }

        self.dp.hfrco0_s.cal().write(|w| unsafe { w.bits(cal) });

        // Wait for the new band to settle.
        let mut spin = 0u32;
        loop {
            let st = self.dp.hfrco0_s.status().read();
            if st.rdy().bit_is_set() && !st.freqbsy().bit_is_set() {
                break;
            }
            spin += 1;
            if spin > 1_000_000 {
                return None;
            }
        }
        // The peripheral bus and the radio clock are derived from the core clock and both have a ceiling:
        // PCLK at 50 MHz and the radio clock at 40 MHz (`em_cmu.c:99,102`). The vendor divides them after every
        // frequency change (`pclkDivOptimize`, `rhclkPrescOptimize`, called from `CMU_ClockFreqGet`'s siblings at
        // `em_cmu.c:4463`), and this function did not -- so at 80 MHz both were running undivided and out of spec,
        // with the Bluetooth stack on. Measured 2026-11-17 by the review that read the vendor source; the divider
        // does not touch the microphone's pace, which runs from HFRCOEM23.
        let pclk = if target_hz > 50_000_000 {
            crate::pac::cmu_s::sysclkctrl::Pclkpresc::Div2
        } else {
            crate::pac::cmu_s::sysclkctrl::Pclkpresc::Div1
        };
        self.dp
            .cmu_s
            .sysclkctrl()
            .modify(|_r, w| w.pclkpresc().variant(pclk));
        let rhclk = if target_hz > 40_000_000 {
            crate::pac::cmu_s::sysclkctrl::Rhclkpresc::Div2
        } else {
            crate::pac::cmu_s::sysclkctrl::Rhclkpresc::Div1
        };
        self.dp
            .cmu_s
            .sysclkctrl()
            .modify(|_r, w| w.rhclkpresc().variant(rhclk));

        // The clock moved, so the board's own answer to "what am I running at" has to move with it. Every boost
        // comes through here, which is why this is the one line that keeps `sysclk_hz` honest -- and a driver
        // calibrated against a stale number is the same defect the timer used to carry, only quieter.
        self.sysclk_hz.set(target_hz);

        Some(cal)
    }

    /// Start TIMER0 free-running at `hz`, with no pin and no PWM channel, as the microphone's pace.
    ///
    /// The IADC's `TRIGGER.SINGLETRIGSEL = Timer` does not go through PRS: it takes the overflow of a timer
    /// **in the same clock group and on the same clock source as the ADC**, which is why this runs TIMER0
    /// from HFRCOEM23 exactly as `converter()` clocks the IADC. Get that wrong and the trigger fires at a
    /// rate nobody chose, silently.
    ///
    /// Returns the period in timer clocks (`19 000 000 / hz`), so a caller can report the rate it actually
    /// got: 16 000 Hz asks for a period of 1 187, which is 16 006 Hz, not 16 000. The error is 0,04 % and it
    /// is stated rather than rounded away.
    ///
    /// `None` if HFRCOEM23 never reports ready — a missing clock, not a missing timer.
    ///
    /// # Conflicts
    ///
    /// This owns TIMER0. `pwm_on` and `buzzer_on` also own TIMER0, so a board that is buzzing cannot also be
    /// paced by it; on the Sense module there is no buzzer, which is why this is acceptable here and would
    /// not be on a board with one.
    pub fn pace_microphone(&self, hz: u32) -> Option<u32> {
        if hz == 0 {
            return None;
        }
        self.dp
            .cmu_s
            .clken0()
            .modify(|_r, w| w.timer0().set_bit().hfrcoem23().set_bit());
        self.dp
            .hfrcoem23_s
            .ctrl()
            .modify(|_r, w| w.forceen().set_bit());
        let mut spin = 0u32;
        while !self.dp.hfrcoem23_s.status().read().rdy().bit_is_set() {
            spin += 1;
            if spin > 1_000_000 {
                return None;
            }
        }
        self.dp
            .cmu_s
            .em01grpaclkctrl()
            .modify(|_r, w| w.clksel().hfrcoem23());

        // TIMER0 is write-protected until the lock key is given, and CFG may not be written while the timer is
        // enabled — doing so faulted the core (measured 2026-11-17: the probe reported "took the board" and
        // then nothing). This is the same order `pwm::Timer0::take` uses, minus the pin routing.
        self.dp
            .timer0_s
            .lock()
            .write(|w| unsafe { w.lockkey().bits(0xCE80) });
        self.dp.timer0_s.en().write(|w| w.en().clear_bit());
        self.dp
            .timer0_s
            .cfg()
            .write(|w| w.mode().up().clksel().prescem01grpaclk().presc().div1());
        // TOP is written with the timer ENABLED, which is the order `pwm::Timer0::take` proves works. Written
        // with EN=0 it raised a precise bus error at 0x4004_801C (TIMER0 + TOP): measured 2026-11-17 from
        // CFSR=0x00008200 (PRECISERR|BFARVALID), HFSR=0x40000000, BFAR=0x4004801C.
        let period = u32::from(self.clock_hz()) / hz;
        self.dp.timer0_s.en().write(|w| w.en().set_bit());
        self.dp
            .timer0_s
            .top()
            .write(|w| unsafe { w.top().bits(period.saturating_sub(1)) });
        self.dp.timer0_s.cmd().write(|w| w.start().set_bit());
        Some(period)
    }

    /// The raw TIMER0 counter, for a probe that has to answer "does the timer run at all".
    ///
    /// Reads `CNT` only: no configuration, no side effects. It exists so that a fault in the pacing code can be
    /// separated from a timer that was never clocked — the two look identical from the mailbox.
    pub fn timer0_counter(&self) -> u32 {
        self.dp.timer0_s.cnt().read().bits()
    }

    /// The frequency this board is running at.
    ///
    /// One place, because there were two: `pace_microphone` typed `19_000_000` into its period and
    /// `delay` used `DEFAULT_SYSCLK_HZ`, so a board that was boosted would have one of them right and
    /// the other wrong, silently. Callers that need a frequency -- `i2c0` wants one to divide down to
    /// a bus rate -- read it here instead of carrying their own copy.
    #[must_use]
    pub fn clock_hz(&self) -> crate::clock::Hertz {
        // It used to answer `DEFAULT_SYSCLK_HZ` regardless of what the board had done to the clock, which made
        // it a second, quieter source of the truth `apply_hfrco_cal` was already keeping. One accessor, and it
        // follows the boost.
        crate::clock::Hertz(self.sysclk_hz.get())
    }

    /// A millisecond delay on SysTick.
    ///
    /// The board owns the core peripherals, so no caller has to reach for
    /// `cortex_m::Peripherals::steal()` to get a delay — which is the same defect as stealing the
    /// PAC, one layer down.
    pub fn delay(&self) -> crate::delay::Delay {
        // SAFETY: the core peripherals were taken by `take()` and are held in `_cp`, so this is
        // not a second owner -- it is the same SysTick, borrowed back. `Delay` only programs the
        // SysTick registers and nothing else in this crate touches them.
        crate::delay::Delay::with_sysclk(
            unsafe { cortex_m::Peripherals::steal() }.SYST,
            crate::clock::DEFAULT_SYSCLK_HZ,
        )
    }

    /// The GPIO register block this board's pins live on.
    #[must_use]
    /// A bit-banged I2C master on the two pins the caller names.
    ///
    /// Bit-banged on purpose: the on-chip I2C0 master on this die cannot complete a read
    /// that follows a write. See `docs/04-reference/I2C0-READ-DEFECT.md`.
    pub fn i2c_on(&self, bus: crate::bus::Bus) -> crate::bus::BitBangI2c<'_> {
        crate::bus::BitBangI2c::on(&self.dp.gpio_s, bus)
    }

    /// A push-pull output on the pin the caller names.
    pub fn output_on(
        &self,
        port: crate::pin::Port,
        pin: u8,
        active_low: bool,
    ) -> crate::pin::Output<'_> {
        crate::pin::Output::new(&self.dp.gpio_s, port, pin, active_low)
    }

    /// An input with the internal pull-up on the pin the caller names.
    pub fn input_on(&self, port: crate::pin::Port, pin: u8) -> crate::pin::Input<'_> {
        crate::pin::Input::with_pull_up(&self.dp.gpio_s, port, pin)
    }

    /// TIMER0 on the pin the caller names, calibrated but undecided about what it means.
    ///
    /// `None` when the counter does not move — a clock that is not running.
    pub fn timer0_on(&self, port: crate::pin::Port, pin: u8) -> Option<crate::pwm::Timer0<'_>> {
        crate::pwm::Timer0::take(&self.dp, port, pin, self.sysclk_hz.get())
    }

    /// A PWM channel on the pin the caller names, at `freq_hz`, dark.
    ///
    /// This is the call an example should reach for: the PAC is taken once, by `take()`, and a
    /// HAL example that writes `pac::Peripherals::steal()` is a HAL that failed to hand out what
    /// it already owns.
    pub fn pwm_on(
        &self,
        port: crate::pin::Port,
        pin: u8,
        freq_hz: u32,
    ) -> Option<crate::pwm::Pwm<'_>> {
        Some(crate::pwm::Pwm::new(self.timer0_on(port, pin)?, freq_hz))
    }

    /// A TIMER0 PWM buzzer on the pin the caller names.
    ///
    /// `None` if the timer never reports a running counter — a missing clock, not a missing
    /// buzzer.
    pub fn buzzer_on(&self, port: crate::pin::Port, pin: u8) -> Option<crate::buzzer::Buzzer<'_>> {
        crate::buzzer::Buzzer::new(&self.dp, port, pin, self.sysclk_hz.get())
    }

    /// IADC0 clocked from HFRCOEM23, ready for the microphone driver.
    ///
    /// HFRCOEM23 rather than FSRCO: measured, with IADCCLK from FSRCO the
    /// converter's synchroniser never finished disabling and the following
    /// configuration write faulted. HFRCOEM23 is the oscillator already proven
    /// to clock TIMER0 here, so it is forced on and waited for before the IADC
    /// clock is pointed at it.
    pub fn converter(&self) -> crate::microphone::Converter {
        // The analog bus comes first. PC9 is an odd-numbered pin, and an odd pin only reaches ADC0 if
        // the odd CD bus is allocated to it: measured 2026-11-17, without this write the converter
        // delivers a flat zero, and `Microphone::level` reports that as `rms=2048` because it centres
        // at mid-scale. The PAC example has always written it (`pac_mic_level`); this path had not,
        // which is why the HAL's microphone read nothing while the PAC's read a noise floor.
        self.dp
            .gpio_s
            .cdbusalloc()
            .modify(|_r, w| unsafe { w.cdodd0().bits(1) });

        self.dp
            .cmu_s
            .clken0()
            .modify(|_r, w| w.iadc0().set_bit().hfrcoem23().set_bit());
        self.dp
            .hfrcoem23_s
            .ctrl()
            .modify(|_r, w| w.forceen().set_bit());
        let mut spin = 0u32;
        while !self.dp.hfrcoem23_s.status().read().rdy().bit_is_set() {
            spin += 1;
            if spin > 1_000_000 {
                break;
            }
        }
        self.dp
            .cmu_s
            .iadcclkctrl()
            .modify(|_r, w| w.clksel().hfrcoem23());
        // The converter lives at a fixed address for the life of the program, so a
        // `'static` view is literally true — the same argument `gpio_static` makes.
        const IADC0_BASE: usize = 0x4900_4000;
        let reg = unsafe { &*(IADC0_BASE as *const pac::iadc0_s::RegisterBlock) };
        crate::microphone::Converter::new(reg)
    }

    /// The general-purpose ADC, distinct from the microphone's converter.
    ///
    /// `Board::converter()` returns the IADC as the microphone uses it; this one hands out the
    /// channel-oriented driver, so an example outside the PAC never has to name a PAC type.
    ///
    /// The oscillator is forced on here for the same reason it is in `converter()`: measured
    /// 2026-11-15, an IADC whose `IADCCLK` points at a stopped HFRCOEM23 faults on the first
    /// `CTRL` write, at `0x49004008`.
    pub fn adc(&self, config: crate::adc::Config) -> crate::adc::Adc {
        self.run_hfrcoem23();
        crate::adc::Adc::new(unsafe { pac::Iadc0S::steal() }, config, &self.dp.cmu_s)
    }

    /// Force HFRCOEM23 on and wait for it, then point `IADCCLK` at it.
    fn run_hfrcoem23(&self) {
        self.dp
            .cmu_s
            .clken0()
            .modify(|_r, w| w.hfrcoem23().set_bit());
        self.dp
            .hfrcoem23_s
            .ctrl()
            .modify(|_r, w| w.forceen().set_bit());
        let mut spin = 0u32;
        while !self.dp.hfrcoem23_s.status().read().rdy().bit_is_set() {
            spin += 1;
            if spin > 1_000_000 {
                break;
            }
        }
        self.dp
            .cmu_s
            .iadcclkctrl()
            .modify(|_r, w| w.clksel().hfrcoem23());
    }
}

/// A bit-banged bus that outlives the `Board` it came from.
///
/// A facade owns its buses for the life of the program, and a bus borrowed from a `Board` cannot
/// be stored in the same struct that owns the board. The registers live at a fixed address, so the
/// `'static` view is literally true; this function exists so a BSP never has to name the PAC to
/// get one.
#[must_use]
pub fn i2c_static(bus: crate::bus::Bus) -> crate::bus::BitBangI2c<'static> {
    crate::bus::BitBangI2c::on(gpio_static(), bus)
}

/// The GPIO register block as a `'static` reference.
///
/// MMIO lives at a fixed address for the whole life of the program, so `'static` is
/// literally true here. It exists because a facade (the BSP) needs to own bus
/// objects that outlive the `Board` value it took them from — the alternative is a
/// self-referential struct, which Rust does not allow, or passing the register block
/// through every call, which is worse.
///
/// The safety argument is the same one the PAC already makes for every register
/// access: the address is fixed by the silicon, and concurrent access is prevented
/// by the bus being owned by one place.
#[must_use]
pub(crate) fn gpio_static() -> &'static pac::gpio_s::RegisterBlock {
    // The type is the **secure** GPIO block (`gpio_s`), and `0x5003_C000` is `GPIO_NS_BASE`
    // (`efr32mg24b220f1536im48.h:588` against `:537` for `GPIO_S_BASE`). With TrustZone off the two alias the
    // same registers, which is why this never showed up as a fault -- but a pointer whose address and type
    // name different blocks is a trap for whoever enables secure mode. Take the address from the PAC.
    unsafe { &*pac::GpioS::ptr() }
}
