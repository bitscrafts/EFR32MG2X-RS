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
    /// Which peripherals have been handed out. See [`Handout`] for what this can and cannot enforce.
    slots: core::cell::Cell<crate::cores::slots::Slots>,
}

/// Why a peripheral was not handed out.
///
/// Two different facts, kept apart on purpose: the board may have already given this peripheral away, or the
/// peripheral may be available and have refused its configuration. A single "error" would make a caller read the
/// first as the second.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handout<E> {
    /// The board already handed this peripheral out. One `steal()` per program is the rule; this is the refusal.
    Taken(crate::cores::slots::Taken),
    /// The peripheral was available and refused what it was asked to do.
    Driver(E),
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
            slots: core::cell::Cell::new(crate::cores::slots::Slots::new()),
        })
    }

    /// Claim a peripheral for this program, or refuse because it is already out.
    ///
    /// Every hand-out below goes through here. The claim is a table of booleans rather than a move because the PAC's
    /// `steal()` hands back a zero-sized handle to a fixed address: two of them are indistinguishable to the compiler,
    /// so the only place the second one can be caught is here, at run time.
    fn claim(&self, slot: crate::cores::slots::Slot) -> Result<(), crate::cores::slots::Taken> {
        let mut table = self.slots.get();
        table.claim(slot)?;
        self.slots.set(table);
        Ok(())
    }

    /// The hardware I2C0, with its clock enabled and its two lines still unrouted.
    ///
    /// `hfclk` is passed in rather than read here: the `Board` holds the CMU inside `Peripherals`
    /// and does not own a `FrozenClocks`, so it would have to guess the frequency. The caller knows
    /// it -- `clocks.hfclk()` -- and a wrong frequency here is a wrong bus rate, which shows up as a
    /// device that never acknowledges.
    ///
    /// Route the pins before using it: [`Board::route_i2c0`].
    pub fn i2c0(
        &self,
        config: crate::i2c::Config,
        hfclk: crate::clock::Hertz,
    ) -> Result<crate::i2c::I2c0, Handout<()>> {
        self.claim(crate::cores::slots::Slot::I2c)
            .map_err(Handout::Taken)?;
        self.dp.cmu_s.clken0().modify(|_, w| w.i2c0().set_bit());
        // SAFETY: `Board` took `Peripherals` once and holds every singleton for the life of the program. Handing I2C0
        // out here is the only path a caller has to it, and no second `steal()` can happen while the board owns the tree.
        Ok(crate::i2c::I2c0::new(
            unsafe { pac::I2c0Ns::steal() },
            config,
            hfclk,
        ))
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
            // SAFETY: `bits()` touches one 4-bit `MODE` field. The mask keeps the other seven pins of the register and
            // the value is one of the numbers the manual gives (`route::PinMode`), never something computed.
            self.dp.gpio_ns.portb_model().modify(|r, w| unsafe {
                let limpo = r.bits() & !(0xF << desloc);
                w.bits(limpo | (u32::from(mode.value()) << desloc))
            });
        }

        // Then the route, then the enable.
        self.dp
            .gpio_ns
            // SAFETY: the value comes from `route::route_value`, which bounds-checks port and pin and places them in
            // the fields the manual defines (`PORT` [1:0], `PIN` [19:16]); no other bit of the register is set.
            .i2c0_sclroute()
            .write(|w| unsafe { w.bits(scl_valor) });
        self.dp
            .gpio_ns
            // SAFETY: as above -- the same bounds-checked `route_value`, for the SDA field.
            .i2c0_sdaroute()
            .write(|w| unsafe { w.bits(sda_valor) });
        self.dp
            .gpio_ns
            .i2c0_routeen()
            .modify(|_, w| w.sclpen().set_bit().sdapen().set_bit());
        Ok(())
    }

    /// USART0 with its clock enabled, ready for the pins to be routed.
    ///
    /// Route first ([`Board::route_usart0`]) and then build the driver, and note that the other order is not
    /// equivalent: a driver enabled with its pins unrouted drives nothing while looking perfectly alive.
    ///
    /// The clock is passed as a number because `FrozenClocks` owns the CMU and this board owns the CMU — the same
    /// reason [`Board::i2c0`] takes a `Hertz`. PCLK is HFCLK here, and the board is what tracks it.
    pub fn usart0(
        &self,
        config: crate::usart::Config,
    ) -> Result<crate::usart::Usart0, Handout<crate::usart::Error>> {
        self.claim(crate::cores::slots::Slot::Usart0)
            .map_err(Handout::Taken)?;
        self.dp.cmu_s.clken0().modify(|_, w| w.usart0().set_bit());
        // SAFETY: as with I2C0 -- the board owns the PAC tree, and this is the single hand-out of USART0.
        crate::usart::Usart0::new_at(
            unsafe { pac::Usart0S::steal() },
            config,
            self.sysclk_hz.get(),
        )
        .map_err(Handout::Driver)
    }

    /// Bring up the branch clock the `TIMER` block counts from, the way `pwm::Timer0::take` learned to do it.
    ///
    /// Measured 2026-11-27, and it cost a flash to find out: gating `CLKEN0.TIMEn` alone is not enough. The first write
    /// to `TIMER1`'s `EN` with the branch clock not running was a **precise bus fault at `0x4004c004`** —
    /// `Escalated BusFault (Precise data access error)` in the fault trace. `EM01GRPACLK` has to be pointed at a
    /// running oscillator, and the oscillator has to be forced on: a selector aimed at an oscillator that nobody turned
    /// on gives a timer that does not count, with no error anywhere.
    ///
    /// Returns `false` if `HFRCOEM23` never reports ready, which is the one failure here that is not a bus fault.
    fn open_timer_branch(&self) -> bool {
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
                return false;
            }
        }
        self.dp
            .cmu_s
            .em01grpaclkctrl()
            .modify(|_r, w| w.clksel().hfrcoem23());
        true
    }

    /// `TIMER1`, handed out with its own claim so two owners cannot pace the same counter.
    ///
    /// This is the first instance `timer::TimerN` can be built for at all. Until now every `Board` hand-out built its
    /// driver from a clock in hertz, and the timer driver's only constructor wanted a `FrozenClocks` the board does not
    /// keep — so `TIMER1`–`TIMER4` compiled, were documented, and were reachable from nowhere. See
    /// [`crate::timer::Timer1::new_at`], which is the same constructor without the clock handle.
    ///
    /// No pin is routed: a timer that only counts and compares needs no pad, and routing one would claim a pin the
    /// caller did not ask for.
    pub fn timer1(
        &self,
        config: crate::timer::Config,
    ) -> Result<crate::timer::Timer1, Handout<()>> {
        self.claim(crate::cores::slots::Slot::Timer1)
            .map_err(Handout::Taken)?;
        self.dp.cmu_s.clken0().modify(|_, w| w.timer1().set_bit());
        if !self.open_timer_branch() {
            return Err(Handout::Driver(()));
        }
        // SAFETY: the board owns the PAC tree, and this is the single hand-out of TIMER1.
        Ok(crate::timer::Timer1::new_at(
            unsafe { pac::Timer1S::steal() },
            config,
            self.sysclk_hz.get(),
        ))
    }

    /// `TIMER2` — the same driver and the same claim discipline as [`Board::timer1`].
    pub fn timer2(
        &self,
        config: crate::timer::Config,
    ) -> Result<crate::timer::Timer2, Handout<()>> {
        self.claim(crate::cores::slots::Slot::Timer2)
            .map_err(Handout::Taken)?;
        self.dp.cmu_s.clken0().modify(|_, w| w.timer2().set_bit());
        if !self.open_timer_branch() {
            return Err(Handout::Driver(()));
        }
        // SAFETY: the board owns the PAC tree, and this is the single hand-out of TIMER2.
        Ok(crate::timer::Timer2::new_at(
            unsafe { pac::Timer2S::steal() },
            config,
            self.sysclk_hz.get(),
        ))
    }

    /// `TIMER3` — the same driver and the same claim discipline as [`Board::timer1`].
    pub fn timer3(
        &self,
        config: crate::timer::Config,
    ) -> Result<crate::timer::Timer3, Handout<()>> {
        self.claim(crate::cores::slots::Slot::Timer3)
            .map_err(Handout::Taken)?;
        self.dp.cmu_s.clken0().modify(|_, w| w.timer3().set_bit());
        if !self.open_timer_branch() {
            return Err(Handout::Driver(()));
        }
        // SAFETY: the board owns the PAC tree, and this is the single hand-out of TIMER3.
        Ok(crate::timer::Timer3::new_at(
            unsafe { pac::Timer3S::steal() },
            config,
            self.sysclk_hz.get(),
        ))
    }

    /// `TIMER4` — the same driver and the same claim discipline as [`Board::timer1`].
    pub fn timer4(
        &self,
        config: crate::timer::Config,
    ) -> Result<crate::timer::Timer4, Handout<()>> {
        self.claim(crate::cores::slots::Slot::Timer4)
            .map_err(Handout::Taken)?;
        self.dp.cmu_s.clken0().modify(|_, w| w.timer4().set_bit());
        if !self.open_timer_branch() {
            return Err(Handout::Driver(()));
        }
        // SAFETY: the board owns the PAC tree, and this is the single hand-out of TIMER4.
        Ok(crate::timer::Timer4::new_at(
            unsafe { pac::Timer4S::steal() },
            config,
            self.sysclk_hz.get(),
        ))
    }

    /// EUSART0 as an SPI master on three routed pins: the board's own SPI pads.
    ///
    /// D10 = PA5 is MOSI, D9 = PA4 is MISO, D8 = PA3 is SCLK — the same three pads Seeed's board support
    /// routes for SPI. The clock is enabled here, the pins are routed here, and the driver is built last.
    pub fn spi_on(
        &self,
        mosi: (crate::pin::Port, u8),
        miso: (crate::pin::Port, u8),
        sclk: (crate::pin::Port, u8),
        config: crate::spi::Config,
    ) -> Result<crate::spi::Spi1, Handout<crate::route::RouteError>> {
        self.claim(crate::cores::slots::Slot::Spi)
            .map_err(Handout::Taken)?;
        self.dp.cmu_s.clken1().modify(|_, w| w.eusart0().set_bit());
        self.route_eusart0_spi(mosi, miso, sclk)
            .map_err(Handout::Driver)?;
        // SAFETY: as with I2C0 -- the board owns the PAC tree, and this is the single hand-out of EUSART0.
        Ok(crate::spi::Spi1::new_at(
            unsafe { pac::Eusart0S::steal() },
            config,
            self.sysclk_hz.get(),
        ))
    }

    /// Route USART0's transmitter and receiver to two pins.
    ///
    /// This is the board's **console** UART: on the XIAO MG24 the pads PA8 (TX) and PA9 (RX) are the ones
    /// wired to the SAMD11 next to the MCU, and the SAMD11 presents them to the host as a serial port. That
    /// makes USART0 the one peripheral in this crate that can be watched from *outside* the board without a
    /// second wire — the host either sees the bytes or it does not.
    ///
    /// The order is the one `route_i2c0` uses, and for the same reason: the pin mode first, then the route,
    /// then the enable. A signal enabled before its pin is driven puts a floating line on the wire.
    ///
    /// The transmitter pin is push-pull and the receiver pin is a plain input; there is no transceiver
    /// direction bit on this part, so telling the two apart is the pin mode's job.
    pub fn route_usart0(
        &self,
        tx: (crate::pin::Port, u8),
        rx: (crate::pin::Port, u8),
    ) -> Result<(), crate::route::RouteError> {
        use crate::route::{port_number, route_value};
        let tx_valor = route_value(port_number(tx.0), tx.1)?;
        let rx_valor = route_value(port_number(rx.0), rx.1)?;

        crate::pin::set_mode(&self.dp.gpio_s, tx.0, tx.1, crate::pin::mode::PUSH_PULL);
        crate::pin::set_mode(&self.dp.gpio_s, rx.0, rx.1, crate::pin::mode::INPUT);

        self.dp
            .gpio_ns
            // SAFETY: `route_value` bounds-checks the pair, so port and pin land in their fields and nothing else is
            // written. The pin mode was set above, which is the half a route register cannot do.
            .usart0_txroute()
            .write(|w| unsafe { w.bits(tx_valor) });
        self.dp
            .gpio_ns
            // SAFETY: as above, for RX.
            .usart0_rxroute()
            .write(|w| unsafe { w.bits(rx_valor) });
        self.dp
            .gpio_ns
            .usart0_routeen()
            .modify(|_, w| w.txpen().set_bit().rxpen().set_bit());
        Ok(())
    }

    /// Route EUSART0 in synchronous mode: MOSI, MISO and the clock.
    ///
    /// On the XIAO MG24 these are the pads the board itself documents as SPI — D10 = PA5 (MOSI),
    /// D9 = PA4 (MISO), D8 = PA3 (SCLK) — and the same three pins Seeed's board support routes.
    ///
    /// MOSI and SCLK are push-pull outputs; MISO is an input, because in master mode the data comes
    /// back on it. Getting that one wrong is silent: a MISO configured as an output reads back the
    /// value the pin is driving, which on a loopback looks exactly like a working bus.
    pub fn route_eusart0_spi(
        &self,
        mosi: (crate::pin::Port, u8),
        miso: (crate::pin::Port, u8),
        sclk: (crate::pin::Port, u8),
    ) -> Result<(), crate::route::RouteError> {
        use crate::route::{port_number, route_value};
        let mosi_valor = route_value(port_number(mosi.0), mosi.1)?;
        let miso_valor = route_value(port_number(miso.0), miso.1)?;
        let sclk_valor = route_value(port_number(sclk.0), sclk.1)?;

        crate::pin::set_mode(&self.dp.gpio_s, mosi.0, mosi.1, crate::pin::mode::PUSH_PULL);
        crate::pin::set_mode(&self.dp.gpio_s, sclk.0, sclk.1, crate::pin::mode::PUSH_PULL);
        crate::pin::set_mode(&self.dp.gpio_s, miso.0, miso.1, crate::pin::mode::INPUT);

        self.dp
            .gpio_ns
            // SAFETY: `route_value` bounds-checks the pair before it gets here, so the fields hold exactly the pads the
            // caller named and no reserved bit is touched.
            .eusart0_txroute()
            .write(|w| unsafe { w.bits(mosi_valor) });
        self.dp
            .gpio_ns
            // SAFETY: as above, for MISO.
            .eusart0_rxroute()
            .write(|w| unsafe { w.bits(miso_valor) });
        self.dp
            .gpio_ns
            // SAFETY: as above, for the clock line.
            .eusart0_sclkroute()
            .write(|w| unsafe { w.bits(sclk_valor) });
        self.dp
            .gpio_ns
            .eusart0_routeen()
            .modify(|_, w| w.txpen().set_bit().rxpen().set_bit().sclkpen().set_bit());
        Ok(())
    }

    /// EUSART0 as an **asynchronous** serial port, with its clock branch and its two pads.
    ///
    /// The EUSART is the same block `spi_on` hands out, in the other mode, so it takes the same slot: a program cannot
    /// have it as SPI and as UART at once, and the claim table is what says so.
    ///
    /// Three things happen here that a caller cannot do for itself without naming the PAC:
    ///
    /// 1. `CMU_CLKEN1.EUSART0`, the bus gate.
    /// 2. `CMU_EUSART0CLKCTRL.CLKSEL = EM01GRPCCLK`, written rather than inherited. The reset value is already this
    ///    branch, but a value nobody set is a value a later change can move without the driver noticing; the baud
    ///    divider is computed against this branch and nothing else.
    /// 3. Nothing about the pads: [`route_eusart0_async`](Self::route_eusart0_async) does that, and it is a separate
    ///    call for the same reason `route_usart0` is -- a driver built before its pins are routed looks alive and drives
    ///    nothing, so the order is the caller's to get right and the doc says which one it is: route first, then build.
    ///
    /// The clock passed to the driver is the board's own `sysclk_hz`: on this part `EM01GRPCCLK` resets to
    /// `HFRCODPLL`, which is the core clock, and `boost` moves the two together.
    pub fn eusart0_async(
        &self,
        config: crate::usart::Config,
    ) -> Result<crate::eusart::Eusart0, Handout<crate::usart::Error>> {
        self.claim(crate::cores::slots::Slot::Spi)
            .map_err(Handout::Taken)?;
        self.dp.cmu_s.clken1().modify(|_, w| w.eusart0().set_bit());
        self.dp
            .cmu_s
            .eusart0clkctrl()
            .modify(|_, w| w.clksel().em01grpcclk());
        // SAFETY: the board owns the PAC tree and this is the single hand-out of EUSART0, through the claim table.
        crate::eusart::Eusart0::new_at(
            unsafe { pac::Eusart0S::steal() },
            config,
            self.sysclk_hz.get(),
        )
        .map_err(Handout::Driver)
    }

    /// Route EUSART0's transmitter and receiver to two pads, for the asynchronous mode.
    ///
    /// Same order as every other route in this file: pin mode, then route value, then enable. TX is a push-pull
    /// output; RX is an input with the internal pull-up selected (`INPUT_PULL` plus `DOUT = 1`), so an idle line is a
    /// real high level rather than whatever the pad's capacitance last held.
    pub fn route_eusart0_async(
        &self,
        tx: (crate::pin::Port, u8),
        rx: (crate::pin::Port, u8),
    ) -> Result<(), crate::route::RouteError> {
        use crate::route::{port_number, route_value};
        let tx_valor = route_value(port_number(tx.0), tx.1)?;
        let rx_valor = route_value(port_number(rx.0), rx.1)?;

        crate::pin::set_mode(&self.dp.gpio_s, tx.0, tx.1, crate::pin::mode::PUSH_PULL);
        crate::pin::set_mode(&self.dp.gpio_s, rx.0, rx.1, crate::pin::mode::INPUT_PULL);
        // The pull-up is selected by the output latch, not by the mode alone.
        crate::pin::write_dout(
            &self.dp.gpio_s,
            rx.0,
            crate::pin::read_dout(&self.dp.gpio_s, rx.0) | (1 << rx.1),
        );

        self.dp
            .gpio_ns
            // SAFETY: `route_value` bounds-checks the pair, so each field holds exactly the pad the caller named.
            .eusart0_txroute()
            .write(|w| unsafe { w.bits(tx_valor) });
        self.dp
            .gpio_ns
            // SAFETY: as above, for RX.
            .eusart0_rxroute()
            .write(|w| unsafe { w.bits(rx_valor) });
        self.dp
            .gpio_ns
            .eusart0_routeen()
            .modify(|_, w| w.txpen().set_bit().rxpen().set_bit());
        Ok(())
    }

    /// Gate the SYSRTC's clocks without starting it. The state the Bluetooth stack needs before it configures the
    /// radio; see `sysrtc::Sysrtc::enable_clocks` for the two faults that bracket this state.
    pub fn prepare_sysrtc_clocks(&self) {
        crate::sysrtc::Sysrtc::enable_clocks(&self.dp.cmu_ns);
    }

    /// The energy management unit, handed out through the board like every other peripheral: an example outside this
    /// crate must not name a PAC type, and the gate enforces that.
    pub fn emu(&self) -> Result<crate::emu::Emu, Handout<()>> {
        self.claim(crate::cores::slots::Slot::Emu)
            .map_err(Handout::Taken)?;
        // SAFETY: `Board` holds the PAC singletons for the whole program; this hands the EMU out through the board, once,
        // instead of letting a caller `steal()` it behind the board's back.
        Ok(crate::emu::Emu::new(unsafe { pac::EmuS::steal() }))
    }

    /// The watchdog, configured at construction. Same reason as `emu`: no PAC type crosses the crate boundary.
    pub fn wdog(
        &self,
        config: crate::wdog::WdogConfig,
    ) -> Result<crate::wdog::Wdog, Handout<crate::wdog::Error>> {
        self.claim(crate::cores::slots::Slot::Wdog)
            .map_err(Handout::Taken)?;
        // SAFETY: same as `emu`: the board owns the peripherals and gives this one out exactly once.
        crate::wdog::Wdog::new(unsafe { pac::Wdog0Ns::steal() }, config).map_err(Handout::Driver)
    }

    /// The CRC accelerator, for checking an image, a configuration blob or a frame.
    ///
    /// One engine, so one owner: two callers interleaving bytes into `INPUTDATABYTE` would produce one nonsense CRC and
    /// two confident callers.
    pub fn gpcrc(&self, poly: crate::gpcrc::CRCPoly) -> Result<crate::gpcrc::Gpcrc, Handout<()>> {
        self.claim(crate::cores::slots::Slot::Gpcrc)
            .map_err(Handout::Taken)?;
        // The bus clock, before anything touches a register in it: measured 2026-11-25, a GPCRC write with its clock
        // gated is a bus fault, which on this part is a hard fault and a board that never writes a record at all.
        self.dp.cmu_s.clken0().modify(|_, w| w.gpcrc().set_bit());
        // SAFETY: the board owns the peripherals and gives this one out exactly once, through the claim table.
        Ok(unsafe { crate::gpcrc::Gpcrc::new(pac::GpcrcNs::steal(), poly) })
    }

    /// The tick timer, shared by every protocol stack that may run on this radio.
    ///
    /// SYSRTC is one peripheral, so it is handed out here rather than taken by a stack: a second consumer
    /// (Zigbee, Thread, or the application's own scheduler) shares this counter instead of starting another.
    pub fn sysrtc(&self) -> Result<crate::sysrtc::Sysrtc, Handout<()>> {
        self.claim(crate::cores::slots::Slot::Sysrtc)
            .map_err(Handout::Taken)?;
        Ok(crate::sysrtc::Sysrtc::take(
            &self.dp.cmu_ns,
            &self.dp.sysrtc0_ns,
        ))
    }

    /// The low-energy timer: 24-bit counter, two compare channels, and the only timer here that runs in EM2.
    ///
    /// The clock is part of the hand-out because it is part of the measurement: `CLKEN0.LETIMER0` alone is not enough,
    /// the EM23 group branch has no source at reset, and a module without one answers on the bus and never counts. The
    /// branch is pinned to LFRCO here, which is the source the record was taken with.
    ///
    /// Left stopped and with `CTRL.CNTTOPEN` set, so the first `TOP` the caller writes is the top.
    pub fn letimer(&self) -> Result<crate::letimer::Letimer, Handout<()>> {
        self.claim(crate::cores::slots::Slot::Letimer)
            .map_err(Handout::Taken)?;
        self.dp.cmu_s.clken0().modify(|_, w| w.letimer0().set_bit());
        // 32.768 kHz LFRCO: the low-frequency domain the module is designed to run from, and the branch the 2026-11-26
        // measurement used. Selecting a source the part does not have running is the defect this line exists to avoid.
        self.dp
            .cmu_s
            .em23grpaclkctrl()
            .modify(|_, w| w.clksel().lfrco());
        // SAFETY: `Board` holds the PAC singletons for the whole program; this hands LETIMER0 out through the claim table,
        // so no second driver can reach the same registers.
        Ok(unsafe { crate::letimer::Letimer::new(pac::Letimer0Ns::steal()) })
    }

    /// The backup counter, on the LFRCO branch and counting from the first tick after this call.
    ///
    /// Three clocks are part of the hand-out because all three are needed and none of them is on at reset: the bus gate
    /// (`CLKEN0.BURTC`), the source of `LFRCO`'s own gate, and `EM4GRPACLK`'s select. A module with the gate and no
    /// source answers on the bus and never counts -- see `burtc::remove_source`, which is that state on purpose.
    ///
    /// `prescaler` is the power of two the counting clock is divided by: `0` counts at 32768 Hz, `3` at 4096 Hz.
    pub fn burtc(
        &self,
        prescaler: u8,
    ) -> Result<crate::burtc::Burtc, Handout<crate::burtc::Error>> {
        self.claim(crate::cores::slots::Slot::Burtc)
            .map_err(Handout::Taken)?;
        self.dp.cmu_s.clken0().modify(|_, w| {
            w.burtc()
                .set_bit()
                // LFRCO's own gate, the same one SYSRTC's take sets: selecting a source that is not running gives a
                // mux that points nowhere.
                .lfrco()
                .set_bit()
        });
        crate::burtc::select_lfrco();
        // SAFETY: `Board` holds the PAC singletons for the whole program; this hands BURTC out through the claim table,
        // so no second driver can reach the same registers.
        unsafe { crate::burtc::Burtc::new(pac::BurtcNs::steal(), prescaler) }
            .map_err(Handout::Driver)
    }

    /// The analogue comparator, with its bus gate already set.
    ///
    /// `ACMP0` is not in `CMU_CLKEN0` with the other analogue blocks: its gate is `CLKEN1.ACMP0`, and with it clear the
    /// first register access is a precise bus fault (`ACMP0_S + 0x04`, measured 2026-11-27). The driver sets it, and the
    /// hand-out exists so the claim table can refuse a second owner of the same input mux.
    pub fn acmp(&self) -> Result<crate::acmp::Acmp0, Handout<crate::acmp::Error>> {
        self.claim(crate::cores::slots::Slot::Acmp0)
            .map_err(Handout::Taken)?;
        // SAFETY: `Board` holds the PAC singletons for the whole program; this hands ACMP0 out through the claim table,
        // so no second driver can reach the same registers.
        crate::acmp::Acmp0::new(unsafe { pac::Acmp0S::steal() }).map_err(Handout::Driver)
    }

    /// The `RADIOAES` crypto block, with its bus gate on.
    ///
    /// The gate is `CMU_CLKEN0.RADIOAES` (bit 2) -- the first bank, unlike `ACMP0`'s. Nothing here touches
    /// `CMU_RADIOCLKCTRL`: the AES engine is in the EM1 power domain with its own bus clock and answers without the
    /// transceiver's clock, which the measurement in `docs/04-reference/AES-KAT-2026-11-27.md` records.
    pub fn aes(&self) -> Result<crate::aes::Aes, Handout<crate::aes::Error>> {
        self.claim(crate::cores::slots::Slot::Aes)
            .map_err(Handout::Taken)?;
        self.dp.cmu_s.clken0().modify(|_, w| w.radioaes().set_bit());
        // SAFETY: `Board` holds the PAC singletons for the whole program; this hands RADIOAES out through the claim
        // table, so no second driver can reach the same registers.
        crate::aes::Aes::new(unsafe { pac::RadioaesS::steal() }).map_err(Handout::Driver)
    }

    /// Entry and exit of EM2, carrying the `EMU` it drives.
    ///
    /// Handed out here for the same reason the timer is: EM2 is where the core stops running from the
    /// high-frequency clock, and the decision to do that belongs to whoever owns the board's clocks.
    pub fn sleep(&self) -> crate::sleep::Sleep<'_> {
        crate::sleep::Sleep::new(&self.dp.emu_ns)
    }

    /// Give the odd CD bus to `ADC0`, which is what an odd pin needs to reach the converter.
    ///
    /// Measured twice, in two places: 2026-11-17 without this write the microphone's converter delivers a flat zero
    /// (`Microphone::level` reports `rms=2048`, mid-scale, because the zero is centred), and 2026-11-26 an unallocated
    /// input still "converted" — it returned 0 and raised `IADC.IF.PORTALLOCERR` (bit 13) — which is a wrong answer
    /// nobody would notice without reading the flag.
    ///
    /// This is one bus and not a per-pin table: it is what this board's odd analogue input (PC9) needs, and the HAL's
    /// `adc` driver refuses to guess about any other pin (`cores::adc::bus_for`).
    pub fn allocate_odd_cd_bus(&self) {
        self.dp
            .gpio_s
            .cdbusalloc()
            // SAFETY: `CDODD0` is a one-bit allocate field; 1 is the only value written, and giving the odd CD bus to
            // IADC0 is what PC9 needs.
            .modify(|_r, w| unsafe { w.cdodd0().bits(1) });
    }

    /// The PRS block, with its bus clock open, for a caller that drives its own trigger.
    ///
    /// Measured 2026-11-26: the IADC converts on a positive edge of an asynchronous PRS channel, and the pulse is a
    /// write to `ASYNC_SWPULSE` that needs no channel configuration. What this hands out is the block and its clock;
    /// [`crate::prs::Prs`] says what it does and does not drive.
    pub fn prs(&self) -> Result<crate::prs::Prs, Handout<()>> {
        self.claim(crate::cores::slots::Slot::Prs)
            .map_err(Handout::Taken)?;
        // The consumer words live in PRS, and a block without its bus clock drops writes: the ADC's triggered path
        // writes `CONSUMER_IADC0_SINGLETRIGGER`, so the gate is opened here, once, for both users.
        self.dp.cmu_s.clken0().modify(|_r, w| w.prs().set_bit());
        // SAFETY: the board owns `Peripherals` and hands PRS out through the claim table, so no second `Prs` exists.
        Ok(unsafe { crate::prs::Prs::new(pac::PrsS::steal()) })
    }

    /// The pulse counter, with its clock open, counting nothing yet.
    ///
    /// `CLKEN1.PCNT0` gates the block and `PCNT0CLKCTRL.CLKSEL` (reset value 1) sends it the `EM23GRPACLK` branch;
    /// that branch has no source of its own at reset, so it is pinned to LFRCO here for the same reason the LETIMER
    /// hand-out pins it — a module with an unsourced branch answers on the bus and never counts. The counted edges
    /// themselves come from S0IN ([`crate::pcnt::Pcnt::use_prs_input`]), not from this branch.
    pub fn pcnt(&self) -> Result<crate::pcnt::Pcnt<'_>, Handout<()>> {
        self.claim(crate::cores::slots::Slot::Pcnt)
            .map_err(Handout::Taken)?;
        self.dp.cmu_s.clken1().modify(|_r, w| w.pcnt0().set_bit());
        self.dp
            .cmu_s
            .pcnt0clkctrl()
            .modify(|_r, w| w.clksel().em23grpaclk());
        self.dp
            .cmu_s
            .em23grpaclkctrl()
            .modify(|_r, w| w.clksel().lfrco());
        // SAFETY: `Board` holds the PAC singletons and hands PCNT0 out through the claim table, so no second driver
        // can reach the same registers. The CMU block is lent, not moved, for the clock-select write the driver owns.
        Ok(unsafe { crate::pcnt::Pcnt::new(pac::Pcnt0Ns::steal(), &self.dp.cmu_s) })
    }

    /// The matrix/vector processor, enabled and ready.
    ///
    /// The board hands it out for the same reason it hands out the others: an example outside the PAC must
    /// not name a PAC type, and `Mvp::take` needs both the CMU and the peripheral.
    ///
    /// The non-secure window is used deliberately. In the secure alias the configuration registers read back
    /// as zero from a non-secure image — measured on this board, in `pac_mvp_add`.
    pub fn mvp(&self) -> Result<crate::mvp::Mvp, Handout<crate::mvp::Error>> {
        self.claim(crate::cores::slots::Slot::Mvp)
            .map_err(Handout::Taken)?;
        // SAFETY: the board owns `Peripherals`, so MVP cannot be taken twice; `Mvp::take` still checks that the block
        // answers before it returns, which is what catches a die where this peripheral is absent.
        crate::mvp::Mvp::take(&self.dp.cmu_ns, unsafe { pac::MvpNs::steal() })
            .map_err(Handout::Driver)
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
            // SAFETY: `LOCKKEY` is a 16-bit field and the value is the unlock sequence from the reference manual; a
            // wrong sequence is refused by the hardware, it does not unlock something else.
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

        // SAFETY: `CAL` is a 7-bit field and `cal` is one of the constants the table above selects by target
        // frequency -- never a computed value, so it cannot exceed the field.
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
            // SAFETY: the timer's `LOCKKEY` is a 16-bit field and `0xCE80` is the unlock sequence the reference manual
            // gives for TIMER; the same constant `pwm::Timer0::take` uses.
            .write(|w| unsafe { w.lockkey().bits(0xCE80) });
        self.dp.timer0_s.en().write(|w| w.en().clear_bit());
        // `CFG` may not be written while the timer is enabled — measured 2026-11-17, a precise bus error at
        // `TIMER0 + 0x04` — and clearing `EN` is not instantaneous: `EN.DISABLING` reports when the block has actually
        // stopped. Without this wait a *second* pacing (a caller re-pacing the same timer) faults on the `CFG` write
        // below, which is exactly what the PCNT measurement hit on 2026-11-26, at `BFAR = 0x4004_8004`.
        cortex_m::asm::delay(16);
        let mut settling = 0u32;
        while self.dp.timer0_s.en().read().disabling().bit_is_set() {
            settling += 1;
            if settling > 2_000_000 {
                break;
            }
        }
        self.dp
            .timer0_s
            .cfg()
            .write(|w| w.mode().up().clksel().prescem01grpaclk().presc().div1());
        // TOP is written with the timer ENABLED, which is the order `pwm::Timer0::take` proves works. Written
        // with EN=0 it raised a precise bus error at 0x4004_801C (TIMER0 + TOP): measured 2026-11-17 from
        // CFSR=0x00008200 (PRECISERR|BFARVALID), HFSR=0x40000000, BFAR=0x4004801C.
        let period = u32::from(self.clock_hz()) / hz;
        // A period above 16 bits is a rate this counter cannot produce, and `bits()` would truncate it silently -- the
        // same class of defect as the baud divider that wrapped at 19 MHz. Refused, not programmed. A period of 0 is the
        // clock itself, which this counter cannot divide down to either.
        if period == 0 || period > 0x1_0000 {
            return None;
        }
        self.dp.timer0_s.en().write(|w| w.en().set_bit());
        // SAFETY: `TOP` is a 16-bit field and the bound above has just refused anything that does not fit it; the
        // `saturating_sub` keeps a period of 1 from underflowing.
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

    /// TIMER0 as a free-running 16-bit counter in the `EM01GRPACLK` group, with that group's source left alone.
    ///
    /// `pace_microphone` is the board's other door to a free-running TIMER0 and, on the way, it points
    /// `EM01GRPACLK` at `HFRCOEM23`. That is right for the IADC's trigger and wrong for an instrument that wants to
    /// count *other* members of the group: the source has to be the caller's choice, so this configures the timer and
    /// nothing else. `clock::sources::em01_select` decides what is being counted.
    ///
    /// Returns `true` when `CNT` moved, so an oscillator that never arrives is a `false` rather than a plausible
    /// count of zeros. The probe is a bounded wait of about 50 ms at 19 MHz, long enough for the slowest source the
    /// group has and short enough that a stopped timer reports instead of hanging.
    ///
    /// # Errors
    ///
    /// [`Handout::Taken`] when another owner already has TIMER0 (`pwm_on`, `buzzer_on`, `pace_microphone`).
    pub fn timer0_free_running(&self, top: u16) -> Result<bool, Handout<()>> {
        self.timer0_free_running_prescaled(top, 0)
    }

    /// The same counter with its prescaler chosen by the caller, `divide_log2` being the number of halvings.
    ///
    /// `0` is the counter undivided, which wraps every 1.7 ms at the crystal and is therefore useless for timing
    /// anything a blocking call does: the caller cannot poll while it is blocked, and a 200 ms delay is 116 wraps it
    /// would have to count. `10` divides the 39 MHz crystal by 1024, which puts one count at 26.3 us and a full
    /// 16-bit wrap at 1.72 s -- one reading, no ambiguity. Values above 10 are clamped to `DIV1024`, the largest the
    /// register has.
    ///
    /// # Errors
    ///
    /// [`Handout::Taken`] when another owner already has TIMER0, exactly as [`Board::timer0_free_running`].
    pub fn timer0_free_running_prescaled(
        &self,
        top: u16,
        divide_log2: u8,
    ) -> Result<bool, Handout<()>> {
        use efr32mg2x_pac::timer0_s::cfg::Presc;
        let presc = match divide_log2 {
            0 => Presc::Div1,
            1 => Presc::Div2,
            2 => Presc::Div4,
            3 => Presc::Div8,
            4 => Presc::Div16,
            5 => Presc::Div32,
            6 => Presc::Div64,
            7 => Presc::Div128,
            8 => Presc::Div256,
            9 => Presc::Div512,
            _ => Presc::Div1024,
        };
        self.claim(crate::cores::slots::Slot::Timer0)
            .map_err(Handout::Taken)?;
        self.dp.cmu_s.clken0().modify(|_r, w| w.timer0().set_bit());
        // Same order `pace_microphone` uses and paid for: the lock key, `EN` down, wait out `EN.DISABLING`, `CFG`,
        // `EN` up, `TOP` (which is only writable with the timer enabled -- measured 2026-11-17, `BFAR = 0x4004_801C`),
        // then `CMD.START`.
        self.dp
            .timer0_s
            .lock()
            // SAFETY: `LOCKKEY` is a 16-bit field and `0xCE80` is the unlock sequence the reference manual gives.
            .write(|w| unsafe { w.lockkey().bits(0xCE80) });
        self.dp.timer0_s.en().write(|w| w.en().clear_bit());
        cortex_m::asm::delay(16);
        let mut settling = 0u32;
        while self.dp.timer0_s.en().read().disabling().bit_is_set() {
            settling += 1;
            if settling > 2_000_000 {
                break;
            }
        }
        self.dp.timer0_s.cfg().write(|w| {
            w.mode()
                .up()
                .clksel()
                .prescem01grpaclk()
                .presc()
                .variant(presc)
        });
        self.dp.timer0_s.en().write(|w| w.en().set_bit());
        self.dp
            .timer0_s
            .top()
            // SAFETY: `top` is a `u16` and `TOP` is a 16-bit field, so the write cannot exceed it.
            .write(|w| unsafe { w.top().bits(u32::from(top)) });
        self.dp.timer0_s.cmd().write(|w| w.start().set_bit());

        let first = self.timer0_counter() as u16;
        for _ in 0..200_000u32 {
            if (self.timer0_counter() as u16) != first {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Run the core from the crystal and report the frequency in force.
    ///
    /// The sequence lives in `clock::start_hfxo` and is the same one `Clocks::new` uses; this is the door that was
    /// missing. Without it a `Board` user could not select the crystal at all: `Clocks::new` asks for the CMU by
    /// value, and the board does not give that up. The only documented way to reach the crystal was to steal the
    /// PAC in an example -- which is the defect the gate refuses.
    ///
    /// On failure the board keeps whatever clock it had: `HfxoTimeout` means the crystal did not stabilise within
    /// the ceiling, and a core running from a clock that never started would be worse than the slow one.
    pub fn select_hfxo(
        &self,
        frequency: u32,
    ) -> Result<crate::clock::Hertz, crate::clock::ClockError> {
        crate::clock::start_hfxo(&self.dp.cmu_s, crate::clock::Hertz(frequency))?;
        self.sysclk_hz.set(frequency);
        Ok(crate::clock::Hertz(frequency))
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
        // The clock in force, not the one the part resets to. It was `DEFAULT_SYSCLK_HZ`, which made the delay
        // wrong by exactly the ratio of any boost or crystal change -- and silent, because a delay that returns
        // early looks the same as a delay that was right. Measured 2026-11-21 with the crystal running: five
        // `delay_ms(1000)` calls took 2,46 s of wall clock, and this is the line that explains the missing half.
        crate::delay::Delay::with_sysclk(
            // SAFETY: `Board` took `cortex_m::Peripherals` once at construction and keeps it; this borrows SysTick back
            // for the delay, and nothing else in this crate touches it.
            unsafe { cortex_m::Peripherals::steal() }.SYST,
            self.clock_hz().0,
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

    /// Point an external-interrupt channel at `port`/`pin` and have it react to the edges named.
    ///
    /// The channel number is the pin number: on this part an external interrupt can only watch within its own group
    /// of four pins (`(channel & ~3) == (pin & ~3)`, the constraint emlib encodes in `GPIO_INTNO_PIN_VALID`), so
    /// binding `channel = pin` is the one choice that is always legal. The returned handle reports and clears the
    /// channel's sticky flag; the channel number is also the `SIGSEL` the PRS needs for `Source::GpioPin`.
    ///
    /// The pin keeps whatever mode it was configured with. `input_on` + `set_pull_up` is enough for a probe with no
    /// jumper wire, because the pull resistor slews the pad on a pin with nothing attached.
    pub fn watch_edges(
        &self,
        port: crate::pin::Port,
        pin: u8,
        rising: bool,
        falling: bool,
    ) -> crate::pin::EdgeWatch<'_> {
        crate::pin::EdgeWatch::bind(&self.dp.gpio_s, port, pin, rising, falling)
    }

    /// The same channel as [`Self::watch_edges`], with its edges counted in the GPIO interrupt handler.
    ///
    /// The caller installs `GPIO_ODD_IRQHandler` and `GPIO_EVEN_IRQHandler`, each one calling
    /// [`crate::extint::service`] with its parity, and arms the returned counter. Nothing polls the flag: the count is
    /// what the vector delivered. See [`crate::extint`] for the two-handler contract.
    pub fn count_edges(
        &self,
        port: crate::pin::Port,
        pin: u8,
        rising: bool,
        falling: bool,
    ) -> crate::extint::EdgeCounter<'_> {
        crate::extint::EdgeCounter::new(crate::pin::EdgeWatch::bind(
            &self.dp.gpio_s,
            port,
            pin,
            rising,
            falling,
        ))
    }

    /// TIMER0 on the pin the caller names, calibrated but undecided about what it means.
    ///
    /// `None` when the counter does not move — a clock that is not running.
    pub fn timer0_on(
        &self,
        port: crate::pin::Port,
        pin: u8,
    ) -> Result<crate::pwm::Timer0<'_>, Handout<()>> {
        self.claim(crate::cores::slots::Slot::Timer0)
            .map_err(Handout::Taken)?;
        crate::pwm::Timer0::take(&self.dp, port, pin, self.sysclk_hz.get())
            .ok_or(Handout::Driver(()))
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
    ) -> Result<crate::pwm::Pwm<'_>, Handout<()>> {
        let timer = self.timer0_on(port, pin)?;
        Ok(crate::pwm::Pwm::new(timer, freq_hz))
    }

    /// A TIMER0 PWM buzzer on the pin the caller names.
    ///
    /// `None` if the timer never reports a running counter — a missing clock, not a missing
    /// buzzer.
    pub fn buzzer_on(
        &self,
        port: crate::pin::Port,
        pin: u8,
    ) -> Result<crate::buzzer::Buzzer<'_>, Handout<()>> {
        // The buzzer takes TIMER0 through `Buzzer::new`, so the slot is claimed here rather than inside the driver: the
        // board is the only thing that knows what else has been handed out.
        self.claim(crate::cores::slots::Slot::Timer0)
            .map_err(Handout::Taken)?;
        crate::buzzer::Buzzer::new(&self.dp, port, pin, self.sysclk_hz.get())
            .ok_or(Handout::Driver(()))
    }

    /// IADC0 clocked from HFRCOEM23, ready for the microphone driver.
    ///
    /// HFRCOEM23 rather than FSRCO: measured, with IADCCLK from FSRCO the
    /// converter's synchroniser never finished disabling and the following
    /// configuration write faulted. HFRCOEM23 is the oscillator already proven
    /// to clock TIMER0 here, so it is forced on and waited for before the IADC
    /// clock is pointed at it.
    pub fn converter(&self) -> Result<crate::microphone::Converter, Handout<()>> {
        self.claim(crate::cores::slots::Slot::Converter)
            .map_err(Handout::Taken)?;
        // The analog bus comes first: PC9 is an odd-numbered pin, and an odd pin does not reach ADC0 without it.
        self.allocate_odd_cd_bus();

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
        // SAFETY: the converter lives at a fixed address for the life of the program, so a `'static` view is literally
        // true -- the same argument `gpio_static` makes. The address is the one the PAC gives for IADC0.
        const IADC0_BASE: usize = 0x4900_4000;
        let reg = unsafe { &*(IADC0_BASE as *const pac::iadc0_s::RegisterBlock) };
        Ok(crate::microphone::Converter::new(reg))
    }

    /// The general-purpose ADC, distinct from the microphone's converter.
    ///
    /// `Board::converter()` returns the IADC as the microphone uses it; this one hands out the
    /// channel-oriented driver, so an example outside the PAC never has to name a PAC type.
    ///
    /// The oscillator is forced on here for the same reason it is in `converter()`: measured
    /// 2026-11-15, an IADC whose `IADCCLK` points at a stopped HFRCOEM23 faults on the first
    /// `CTRL` write, at `0x49004008`.
    pub fn adc(&self, config: crate::adc::Config) -> Result<crate::adc::Adc, Handout<()>> {
        self.claim(crate::cores::slots::Slot::Adc)
            .map_err(Handout::Taken)?;
        self.run_hfrcoem23();
        // SAFETY: the board owns `Peripherals`; the ADC is handed out through it, so no caller can reach it another way.
        Ok(crate::adc::Adc::new(
            unsafe { pac::Iadc0S::steal() },
            config,
            &self.dp.cmu_s,
        ))
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
    // SAFETY: the GPIO block lives at a fixed address for the whole run, and the pointer comes from the PAC rather
    // than from a literal.
    //
    // The type is the **secure** GPIO block (`gpio_s`), and `0x5003_C000` is `GPIO_NS_BASE`
    // (`efr32mg24b220f1536im48.h:588` against `:537` for `GPIO_S_BASE`). With TrustZone off the two alias the
    // same registers, which is why this never showed up as a fault -- but a pointer whose address and type
    // name different blocks is a trap for whoever enables secure mode. Take the address from the PAC.
    unsafe { &*pac::GpioS::ptr() }
}
