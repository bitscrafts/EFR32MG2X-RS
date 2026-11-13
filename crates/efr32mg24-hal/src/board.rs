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

use efr32mg24_pac as pac;

/// Everything an application needs from the XIAO MG24 Sense + Expansion Base.
pub struct Board {
    /// Held so `cortex_m::Peripherals::take()` cannot succeed a second time. SysTick stays
    /// inside it; `delay()` borrows it back.
    _cp: cortex_m::Peripherals,
    dp: pac::Peripherals,
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
        Some(Self { dp, _cp: cp })
    }

    /// The tick timer, shared by every protocol stack that may run on this radio.
    ///
    /// SYSRTC is one peripheral, so it is handed out here rather than taken by a stack: a second consumer
    /// (Zigbee, Thread, or the application's own scheduler) shares this counter instead of starting another.
    pub fn sysrtc(&self) -> crate::sysrtc::Sysrtc {
        crate::sysrtc::Sysrtc::take(&self.dp.cmu_ns, &self.dp.sysrtc0_ns)
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
        crate::pwm::Timer0::take(&self.dp, port, pin)
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
        crate::buzzer::Buzzer::new(&self.dp, port, pin)
    }

    /// IADC0 clocked from HFRCOEM23, ready for the microphone driver.
    ///
    /// HFRCOEM23 rather than FSRCO: measured, with IADCCLK from FSRCO the
    /// converter's synchroniser never finished disabling and the following
    /// configuration write faulted. HFRCOEM23 is the oscillator already proven
    /// to clock TIMER0 here, so it is forced on and waited for before the IADC
    /// clock is pointed at it.
    pub fn converter(&self) -> crate::microphone::Converter {
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
    // GPIO base from the device memory map.
    const GPIO_BASE: usize = 0x5003_C000;
    unsafe { &*(GPIO_BASE as *const pac::gpio_s::RegisterBlock) }
}
